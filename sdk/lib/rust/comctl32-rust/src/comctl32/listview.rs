use crate::comctl32::comctl32undoc::Alloc;
use crate::comctl32::commctrl::COMCTL32_CreateToolTip;
use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::comctl32::commctrl::COMCTL32_RefreshSysColors;
use crate::comctl32::dpa::DPA_Create;
use crate::comctl32::dpa::DPA_DeleteAllPtrs;
use crate::comctl32::dpa::DPA_DeletePtr;
use crate::comctl32::dpa::DPA_Destroy;
use crate::comctl32::dpa::DPA_GetPtr;
use crate::comctl32::dpa::DPA_GetPtrIndex;
use crate::comctl32::dpa::DPA_InsertPtr;
use crate::comctl32::dpa::DPA_Search;
use crate::comctl32::dpa::DPA_SetPtr;
use crate::comctl32::dpa::DPA_Sort;
use crate::comctl32::commctrl::DrawShadowText;
use crate::comctl32::comctl32undoc::Free;
use crate::comctl32::imagelist::ImageList_Add;
use crate::comctl32::imagelist::ImageList_Create;
use crate::comctl32::imagelist::ImageList_Destroy;
use crate::comctl32::imagelist::ImageList_Draw;
use crate::comctl32::imagelist::ImageList_DrawEx;
use crate::comctl32::imagelist::ImageList_GetIconSize;
use crate::comctl32::imagelist::ImageList_SetBkColor;
use crate::comctl32::comctl32undoc::Str_SetPtrWtoA;
use crate::comctl32::commctrl::_TrackMouseEvent;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_listview);
extern "system" {
    fn SendMessageW(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const WC_LISTVIEWW: [::core::ffi::c_ushort; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_ushort; 14]>(
        *b"S\0y\0s\0L\0i\0s\0t\0V\0i\0e\0w\x003\x002\0\0\0",
    )
};

static mut themeClass: [WCHAR; 9] = [
    'L' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'V' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'w' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[repr(C)]
pub struct COLUMN_INFO {
    pub rcHeader: RECT,
    pub fmt: INT,
    pub cxMin: INT,
}

#[repr(C)]
pub struct ITEMHDR {
    pub pszText: LPWSTR,
    pub iImage: INT,
}
pub type LPITEMHDR = *mut ITEMHDR;

#[repr(C)]
pub struct SUBITEM_INFO {
    pub hdr: ITEMHDR,
    pub iSubItem: INT,
}

#[repr(C)]
pub struct ITEM_INFO {
    pub hdr: ITEMHDR,
    pub state: UINT,
    pub lParam: LPARAM,
    pub iIndent: INT,
    pub id: *mut ITEM_ID,
}

#[repr(C)]
pub struct ITEM_ID {
    pub id: UINT,
    pub item: HDPA,
}

#[repr(C)]
pub struct RANGE {
    pub lower: INT,
    pub upper: INT,
}

#[repr(C)]
pub struct tagRANGES {
    pub hdpa: HDPA,
}
pub type RANGES = *mut tagRANGES;

#[repr(C)]
pub struct ITERATOR {
    pub nItem: INT,
    pub nSpecial: INT,
    pub range: RANGE,
    pub ranges: RANGES,
    pub index: INT,
}

#[repr(C)]
pub struct DELAYED_ITEM_EDIT {
    pub fEnabled: BOOL,
    pub iItem: INT,
}

pub const NOTIFY_MASK_ITEM_CHANGE: u32 = 0x1;
pub const NOTIFY_MASK_END_LABEL_EDIT: u32 = 0x2;
pub const NOTIFY_MASK_UNMASK_ALL: u32 = 0xffffffff;

#[repr(C)]
pub struct LISTVIEW_INFO {
    /* control window */
    pub hwndSelf: HWND,
    pub rcList: RECT,

    /* notification window */
    pub notifyFormat: SHORT,
    pub hwndNotify: HWND,
    pub notify_mask: DWORD,
    pub uCallbackMask: UINT,

    /* tooltips */
    pub hwndToolTip: HWND,

    /* items */
    pub nItemCount: INT,
    pub hdpaItems: HDPA,
    pub hdpaItemIds: HDPA,
    pub hdpaPosX: HDPA,
    pub hdpaPosY: HDPA,
    pub selectionRanges: RANGES,
    pub nSelectionMark: INT,
    pub nHotItem: INT,

    /* columns */
    pub hdpaColumns: HDPA,
    pub colRectsDirty: BOOL,

    /* item metrics */
    pub bNoItemMetrics: BOOL,
    pub nItemHeight: INT,
    pub nItemWidth: INT,

    /* sorting */
    pub pfnCompare: PFNLVCOMPARE,
    pub lParamSort: LPARAM,

    /* style */
    pub dwStyle: DWORD,
    pub dwLvExStyle: DWORD,
    pub uView: DWORD,

    /* edit item */
    pub hwndEdit: HWND,
    pub EditWndProc: WNDPROC,
    pub nEditLabelItem: INT,
    pub itemEdit: DELAYED_ITEM_EDIT,

    /* icons */
    pub himlNormal: HIMAGELIST,
    pub himlSmall: HIMAGELIST,
    pub himlState: HIMAGELIST,
    pub iconSize: SIZE,
    pub autoSpacing: BOOL,
    pub iconSpacing: SIZE,
    pub iconStateSize: SIZE,
    pub currIconPos: POINT,

    /* header */
    pub hwndHeader: HWND,
    pub xTrackLine: INT,

    /* marquee selection */
    pub bMarqueeSelect: BOOL,
    pub bScrolling: BOOL,
    pub marqueeRect: RECT,
    pub marqueeDrawRect: RECT,
    pub marqueeOrigin: POINT,

    /* focus drawing */
    pub bFocus: BOOL,
    pub nFocusedItem: INT,
    pub rcFocus: RECT,

    /* colors */
    pub hBkBrush: HBRUSH,
    pub clrBk: COLORREF,
    pub clrText: COLORREF,
    pub clrTextBk: COLORREF,
    pub bDefaultBkColor: BOOL,

    /* font */
    pub hDefaultFont: HFONT,
    pub hFont: HFONT,
    pub ntmHeight: INT,
    pub ntmMaxCharWidth: INT,
    pub nEllipsisWidth: INT,

    /* mouse operation */
    pub bLButtonDown: BOOL,
    pub bDragging: BOOL,
    pub ptClickPos: POINT,
    pub nLButtonDownItem: INT,
    pub dwHoverTime: DWORD,
    pub hHotCursor: HCURSOR,
    pub cWheelRemainder: INT,

    /* keyboard operation */
    pub lastKeyPressTimestamp: DWORD,
    pub charCode: WPARAM,
    pub nSearchParamLength: INT,
    pub szSearchParam: [WCHAR; MAX_PATH as usize],

    /* painting */
    pub bIsDrawing: BOOL,
    pub nMeasureItemHeight: INT,
    pub redraw: BOOL,

    /* misc */
    pub iVersion: DWORD,
}

pub const DEBUG_BUFFERS: i32 = 20;

pub const DEBUG_BUFFER_SIZE: i32 = 256;

pub const SB_INTERNAL: i32 = -1;

pub const DISP_TEXT_SIZE: i32 = 260;

pub const WIDTH_PADDING: i32 = 12;

pub const HEIGHT_PADDING: i32 = 1;

pub const REPORT_MARGINX: i32 = 2;

pub const ICON_TOP_PADDING_NOTHITABLE: i32 = 2;

pub const ICON_TOP_PADDING_HITABLE: i32 = 2;

pub const ICON_TOP_PADDING: i32 = ICON_TOP_PADDING_NOTHITABLE + ICON_TOP_PADDING_HITABLE;

pub const ICON_BOTTOM_PADDING: i32 = 4;

pub const LABEL_HOR_PADDING: i32 = 5;

pub const LABEL_VERT_PADDING: i32 = 7;

pub const ICON_LR_PADDING: i32 = 16;

pub const ICON_LR_HALF: i32 = ICON_LR_PADDING / 2;

pub const DEFAULT_LABEL_WIDTH: i32 = 40;

pub const MAX_EMPTYTEXT_SELECT_WIDTH: i32 = 80;

pub const DEFAULT_COLUMN_WIDTH: i32 = 128;

pub const LISTVIEW_SCROLL_ICON_LINE_SIZE: i32 = 37;

pub const IMAGE_PADDING: i32 = 2;

pub const TRAILING_LABEL_PADDING: i32 = 12;

pub const TRAILING_HEADER_PADDING: i32 = 11;

pub const CAPTION_BORDER: i32 = 2;

pub const LV_ML_DT_FLAGS: u32 = DT_TOP | DT_NOPREFIX | DT_EDITCONTROL | DT_CENTER | DT_WORDBREAK | DT_WORD_ELLIPSIS | DT_END_ELLIPSIS;

pub const LV_FL_DT_FLAGS: u32 = DT_TOP | DT_NOPREFIX | DT_EDITCONTROL | DT_CENTER | DT_WORDBREAK | DT_NOCLIP;

pub const LV_SL_DT_FLAGS: u32 = DT_VCENTER | DT_NOPREFIX | DT_EDITCONTROL | DT_SINGLELINE | DT_WORD_ELLIPSIS | DT_END_ELLIPSIS;

pub const KEY_DELAY: i32 = 450;

pub const SHIFT_KEY: u32 = 0x1;
pub const CTRL_KEY: u32 = 0x2;
pub const SPACE_KEY: u32 = 0x4;

pub const SCROLL_LEFT: i32 = 0x1;

pub const SCROLL_RIGHT: i32 = 0x2;

pub const SCROLL_UP: i32 = 0x4;

pub const SCROLL_DOWN: i32 = 0x8;

#[allow(non_upper_case_globals)]
const __WINE_DBCL_TRACE: __wine_debug_class = 3;
unsafe extern "C" fn is_text(text: LPCWSTR) -> BOOL {
    return (!text.is_null() && text != LPSTR_TEXTCALLBACKW as LPCWSTR) as BOOL;
}

unsafe extern "C" fn textlenT(text: LPCWSTR, isW: BOOL) -> ::core::ffi::c_int {
    return if is_text(text) == 0 {
        0
    } else if isW != 0 {
        lstrlenW(text)
    } else {
        lstrlenA(text as LPCSTR)
    };
}

unsafe extern "C" fn textcpynT(dest: LPWSTR, isDestW: BOOL, src: LPCWSTR, isSrcW: BOOL, max: INT) {
    if isDestW != 0 {
        if isSrcW != 0 {
            lstrcpynW(dest, src, max);
        } else {
            MultiByteToWideChar(
                CP_ACP,
                0,
                src as LPCSTR,
                -1,
                dest,
                max,
            );
        }
    } else {
        if isSrcW != 0 {
            WideCharToMultiByte(
                CP_ACP,
                0,
                src,
                -1,
                dest as LPSTR,
                max,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            );
        } else {
            lstrcpynA(dest as LPSTR, src as LPCSTR, max);
        }
    }
}

unsafe extern "C" fn textdupTtoW(text: LPCWSTR, isW: BOOL) -> LPWSTR {
    let mut wstr: LPWSTR = text as LPWSTR;
    if isW == 0 && is_text(text) != 0 {
        let mut len: INT = MultiByteToWideChar(
            CP_ACP,
            0,
            text as LPCSTR,
            -1,
            core::ptr::null_mut(),
            0,
        ) as INT;
        wstr = Alloc((len as usize * core::mem::size_of::<WCHAR>()) as DWORD) as LPWSTR;
        if !wstr.is_null() {
            MultiByteToWideChar(
                CP_ACP,
                0,
                text as LPCSTR,
                -1,
                wstr,
                len,
            );
        }
    }
    return wstr;
}

unsafe extern "C" fn textfreeT(wstr: LPWSTR, isW: BOOL) {
    if isW == 0 && is_text(wstr as LPCWSTR) != 0 {
        Free(wstr as LPVOID);
    }
}

unsafe extern "C" fn textsetptrT(dest: *mut LPWSTR, src: LPCWSTR, isW: BOOL) -> BOOL {
    let mut bResult: BOOL = TRUE;
    if src == LPSTR_TEXTCALLBACKW as LPCWSTR {
        if is_text(*dest as LPCWSTR) != 0 {
            Free(*dest as LPVOID);
        }
        *dest = LPSTR_TEXTCALLBACKW;
    } else {
        let mut pszText: LPWSTR = textdupTtoW(src, isW);
        if *dest == LPSTR_TEXTCALLBACKW {
            *dest = ::core::ptr::null_mut::<WCHAR>();
        }
        bResult = Str_SetPtrW(dest, pszText as LPCWSTR);
        textfreeT(pszText, isW);
    }
    return bResult;
}

unsafe extern "C" fn textcmpWT(aw: LPCWSTR, bt: LPCWSTR, isW: BOOL) -> ::core::ffi::c_int {
    if aw.is_null() {
        return if !bt.is_null() {
            -(1 as ::core::ffi::c_int)
        } else {
            0 as ::core::ffi::c_int
        };
    }
    if bt.is_null() {
        return 1 as ::core::ffi::c_int;
    }
    if aw == LPSTR_TEXTCALLBACKW as LPCWSTR {
        return if bt == LPSTR_TEXTCALLBACKW as LPCWSTR {
            1 as ::core::ffi::c_int
        } else {
            -(1 as ::core::ffi::c_int)
        };
    }
    if bt != LPSTR_TEXTCALLBACKW as LPCWSTR {
        let mut bw: LPWSTR = textdupTtoW(bt, isW);
        let mut r: ::core::ffi::c_int = if !bw.is_null() {
            lstrcmpW(aw, bw as LPCWSTR)
        } else {
            1 as ::core::ffi::c_int
        };
        textfreeT(bw, isW);
        return r;
    }

    return 1 as ::core::ffi::c_int;
}

unsafe extern "C" fn debugtext_t(text: LPCWSTR, isW: BOOL) -> LPCSTR {
    if text == LPSTR_TEXTCALLBACKW as LPCWSTR {
        return b"(callback)\0".as_ptr() as LPCSTR;
    }
    if isW != 0 {
        debugstr_w(text as *const WCHAR) as LPCSTR
    } else {
        debugstr_a(text as *const i8) as LPCSTR
    }
}

unsafe extern "C" fn debugtext_tn(text: LPCWSTR, isW: BOOL, mut n: INT) -> LPCSTR {
    if text == LPSTR_TEXTCALLBACKW as LPCWSTR {
        return b"(callback)\0".as_ptr() as LPCSTR;
    }
    n = min(textlenT(text, isW), n);
    return if isW != 0 {
        debugstr_wn(text as *const WCHAR, n) as LPCSTR
    } else {
        debugstr_an(text as *const ::core::ffi::c_char, n) as LPCSTR
    };
}

unsafe extern "C" fn debug_getbuf() -> *mut ::core::ffi::c_char {
    static mut index: ::core::ffi::c_int = 0;
    static mut buffers: [[::core::ffi::c_char; (DEBUG_BUFFER_SIZE) as usize]; (DEBUG_BUFFERS as usize)] =
        [[0; (DEBUG_BUFFER_SIZE) as usize]; (DEBUG_BUFFERS as usize)];
    let fresh15 = index;
    index = index + 1;
    return (&raw mut buffers as *mut [::core::ffi::c_char; (DEBUG_BUFFER_SIZE) as usize])
        .offset((fresh15 % DEBUG_BUFFERS) as isize) as *mut ::core::ffi::c_char;
}

unsafe extern "C" fn debugrange(lprng: *const RANGE) -> *const ::core::ffi::c_char {
    if lprng.is_null() {
        return b"(null)\0".as_ptr() as *const ::core::ffi::c_char;
    }
    return wine_dbg_sprintf(
        b"[%d, %d]\0".as_ptr() as *const ::core::ffi::c_char,
        (*lprng).lower,
        (*lprng).upper,
    );
}

unsafe extern "C" fn debugscrollinfo(pScrollInfo: *const SCROLLINFO) -> *const ::core::ffi::c_char {
    let mut buf: *mut ::core::ffi::c_char = debug_getbuf();
    let mut text: *mut ::core::ffi::c_char = buf;
    let mut len: ::core::ffi::c_int = 0;
    let mut size: ::core::ffi::c_int = DEBUG_BUFFER_SIZE;
    if pScrollInfo.is_null() {
        return b"(null)\0".as_ptr() as *const ::core::ffi::c_char;
    }
    let mut at_end: bool = false;
    'body: {
        len = snprintf(
            buf,
            size as usize,
            b"{cbSize=%u, \0".as_ptr() as *const ::core::ffi::c_char,
            (*pScrollInfo).cbSize,
        );
        if len == -1 {
            at_end = true;
            break 'body;
        }
        buf = buf.offset(len as isize);
        size -= len;
        if ((*pScrollInfo).fMask & SIF_RANGE) != 0 {
            len = snprintf(
                buf,
                size as usize,
                b"nMin=%d, nMax=%d, \0".as_ptr() as *const ::core::ffi::c_char,
                (*pScrollInfo).nMin,
                (*pScrollInfo).nMax,
            );
        } else {
            len = 0;
        }
        if len == -1 {
            at_end = true;
            break 'body;
        }
        buf = buf.offset(len as isize);
        size -= len;
        if ((*pScrollInfo).fMask & SIF_PAGE) != 0 {
            len = snprintf(
                buf,
                size as usize,
                b"nPage=%u, \0".as_ptr() as *const ::core::ffi::c_char,
                (*pScrollInfo).nPage,
            );
        } else {
            len = 0;
        }
        if len == -1 {
            at_end = true;
            break 'body;
        }
        buf = buf.offset(len as isize);
        size -= len;
        if ((*pScrollInfo).fMask & SIF_POS) != 0 {
            len = snprintf(
                buf,
                size as usize,
                b"nPos=%d, \0".as_ptr() as *const ::core::ffi::c_char,
                (*pScrollInfo).nPos,
            );
        } else {
            len = 0;
        }
        if len == -1 {
            at_end = true;
            break 'body;
        }
        buf = buf.offset(len as isize);
        size -= len;
        if ((*pScrollInfo).fMask & SIF_TRACKPOS) != 0 {
            len = snprintf(
                buf,
                size as usize,
                b"nTrackPos=%d, \0".as_ptr() as *const ::core::ffi::c_char,
                (*pScrollInfo).nTrackPos,
            );
        } else {
            len = 0;
        }
        if len == -1 {
            at_end = true;
            break 'body;
        }
        buf = buf.offset(len as isize);
    }
    if at_end {
        buf = text.offset(strlen(text) as isize);
    }
    if (buf.offset_from(text) as ::core::ffi::c_longlong) > 2 as ::core::ffi::c_longlong {
        *buf.offset(-(2) as isize) = b'}' as ::core::ffi::c_char;
        *buf.offset(-(1) as isize) = 0 as ::core::ffi::c_char;
    }
    return text;
}

unsafe extern "C" fn debugnmlistview(plvnm: *const NMLISTVIEW) -> *const ::core::ffi::c_char {
    if plvnm.is_null() {
        return b"(null)\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return wine_dbg_sprintf(
        b"iItem=%d, iSubItem=%d, uNewState=0x%x, uOldState=0x%x, uChanged=0x%x, ptAction=%s, lParam=%ld\0"
            as *const u8 as *const ::core::ffi::c_char,
        (*plvnm).iItem,
        (*plvnm).iSubItem,
        (*plvnm).uNewState,
        (*plvnm).uOldState,
        (*plvnm).uChanged,
        wine_dbgstr_point(core::ptr::addr_of!((*plvnm).ptAction)),
        (*plvnm).lParam,
    );
}

unsafe extern "C" fn debuglvitem_t(lpLVItem: *const LVITEMW, isW: BOOL) -> *const ::core::ffi::c_char {
    let mut buf: *mut ::core::ffi::c_char = debug_getbuf();
    let mut text: *mut ::core::ffi::c_char = buf;
    let mut len: ::core::ffi::c_int = 0;
    let mut size: ::core::ffi::c_int = DEBUG_BUFFER_SIZE;
    let mut failed: bool = false;

    if lpLVItem.is_null() {
        return b"(null)\0".as_ptr() as *const ::core::ffi::c_char;
    }

    'body: {
        len = snprintf(
            buf,
            size as usize,
            b"{iItem=%d, iSubItem=%d, \0".as_ptr() as *const ::core::ffi::c_char,
            (*lpLVItem).iItem,
            (*lpLVItem).iSubItem,
        );
        if len == -(1) {
            failed = true;
            break 'body;
        }
        buf = buf.offset(len as isize);
        size -= len;

        if ((*lpLVItem).mask & (LVIF_STATE as UINT)) != 0 {
            len = snprintf(
                buf,
                size as usize,
                b"state=%x, stateMask=%x, \0".as_ptr() as *const ::core::ffi::c_char,
                (*lpLVItem).state,
                (*lpLVItem).stateMask,
            );
        } else {
            len = 0;
        }
        if len == -(1) {
            failed = true;
            break 'body;
        }
        buf = buf.offset(len as isize);
        size -= len;

        if ((*lpLVItem).mask & (LVIF_TEXT as UINT)) != 0 {
            len = snprintf(
                buf,
                size as usize,
                b"pszText=%s, cchTextMax=%d, \0".as_ptr() as *const ::core::ffi::c_char,
                debugtext_tn(
                    (*lpLVItem).pszText as LPCWSTR,
                    isW,
                    80 as INT,
                ),
                (*lpLVItem).cchTextMax,
            );
        } else {
            len = 0;
        }
        if len == -(1) {
            failed = true;
            break 'body;
        }
        buf = buf.offset(len as isize);
        size -= len;

        if ((*lpLVItem).mask & (LVIF_IMAGE as UINT)) != 0 {
            len = snprintf(
                buf,
                size as usize,
                b"iImage=%d, \0".as_ptr() as *const ::core::ffi::c_char,
                (*lpLVItem).iImage,
            );
        } else {
            len = 0;
        }
        if len == -(1) {
            failed = true;
            break 'body;
        }
        buf = buf.offset(len as isize);
        size -= len;

        if ((*lpLVItem).mask & (LVIF_PARAM as UINT)) != 0 {
            len = snprintf(
                buf,
                size as usize,
                b"lParam=%lx, \0".as_ptr() as *const ::core::ffi::c_char,
                (*lpLVItem).lParam,
            );
        } else {
            len = 0;
        }
        if len == -(1) {
            failed = true;
            break 'body;
        }
        buf = buf.offset(len as isize);
        size -= len;

        if ((*lpLVItem).mask & (LVIF_INDENT as UINT)) != 0 {
            len = snprintf(
                buf,
                size as usize,
                b"iIndent=%d, \0".as_ptr() as *const ::core::ffi::c_char,
                (*lpLVItem).iIndent,
            );
        } else {
            len = 0;
        }
        if len == -(1) {
            failed = true;
            break 'body;
        }
        buf = buf.offset(len as isize);
    }

    if failed {
        buf = text.offset(strlen(text) as isize);
    }

    if buf.offset_from(text) as ::core::ffi::c_longlong > 2 as ::core::ffi::c_longlong {
        *buf.offset(-(2) as isize) = '}' as ::core::ffi::c_char;
        *buf.offset(-(1) as isize) = 0 as ::core::ffi::c_char;
    }
    return text;
}

unsafe extern "C" fn debuglvcolumn_t(lpColumn: *const LVCOLUMNW, isW: BOOL) -> *const ::core::ffi::c_char {
    let mut buf: *mut ::core::ffi::c_char = debug_getbuf();
    let text: *mut ::core::ffi::c_char = buf;
    let mut len: ::core::ffi::c_int = 0;
    let mut size: ::core::ffi::c_int = DEBUG_BUFFER_SIZE as ::core::ffi::c_int;
    if lpColumn.is_null() {
        return b"(null)\0" as *const u8 as *const ::core::ffi::c_char;
    }
    let ok: bool = 'chk: {
        len = snprintf(
            buf,
            size as usize,
            b"{\0" as *const u8 as *const ::core::ffi::c_char,
        );
        if len == -(1 as ::core::ffi::c_int) {
            break 'chk false;
        }
        buf = buf.offset(len as isize);
        size -= len;
        if ((*lpColumn).mask & (LVCF_SUBITEM as UINT)) != 0 {
            len = snprintf(
                buf,
                size as usize,
                b"iSubItem=%d, \0" as *const u8 as *const ::core::ffi::c_char,
                (*lpColumn).iSubItem,
            );
        } else {
            len = 0 as ::core::ffi::c_int;
        }
        if len == -(1 as ::core::ffi::c_int) {
            break 'chk false;
        }
        buf = buf.offset(len as isize);
        size -= len;
        if ((*lpColumn).mask & (LVCF_FMT as UINT)) != 0 {
            len = snprintf(
                buf,
                size as usize,
                b"fmt=%x, \0" as *const u8 as *const ::core::ffi::c_char,
                (*lpColumn).fmt,
            );
        } else {
            len = 0 as ::core::ffi::c_int;
        }
        if len == -(1 as ::core::ffi::c_int) {
            break 'chk false;
        }
        buf = buf.offset(len as isize);
        size -= len;
        if ((*lpColumn).mask & (LVCF_WIDTH as UINT)) != 0 {
            len = snprintf(
                buf,
                size as usize,
                b"cx=%d, \0" as *const u8 as *const ::core::ffi::c_char,
                (*lpColumn).cx,
            );
        } else {
            len = 0 as ::core::ffi::c_int;
        }
        if len == -(1 as ::core::ffi::c_int) {
            break 'chk false;
        }
        buf = buf.offset(len as isize);
        size -= len;
        if ((*lpColumn).mask & (LVCF_TEXT as UINT)) != 0 {
            len = snprintf(
                buf,
                size as usize,
                b"pszText=%s, cchTextMax=%d, \0" as *const u8 as *const ::core::ffi::c_char,
                debugtext_tn(
                    (*lpColumn).pszText as LPCWSTR,
                    isW,
                    80 as INT,
                ),
                (*lpColumn).cchTextMax,
            );
        } else {
            len = 0 as ::core::ffi::c_int;
        }
        if len == -(1 as ::core::ffi::c_int) {
            break 'chk false;
        }
        buf = buf.offset(len as isize);
        size -= len;
        if ((*lpColumn).mask & (LVCF_IMAGE as UINT)) != 0 {
            len = snprintf(
                buf,
                size as usize,
                b"iImage=%d, \0" as *const u8 as *const ::core::ffi::c_char,
                (*lpColumn).iImage,
            );
        } else {
            len = 0 as ::core::ffi::c_int;
        }
        if len == -(1 as ::core::ffi::c_int) {
            break 'chk false;
        }
        buf = buf.offset(len as isize);
        size -= len;
        if ((*lpColumn).mask & (LVCF_ORDER as UINT)) != 0 {
            len = snprintf(
                buf,
                size as usize,
                b"iOrder=%d, \0" as *const u8 as *const ::core::ffi::c_char,
                (*lpColumn).iOrder,
            );
        } else {
            len = 0 as ::core::ffi::c_int;
        }
        if len == -(1 as ::core::ffi::c_int) {
            break 'chk false;
        }
        buf = buf.offset(len as isize);
        true
    };
    if !ok {
        buf = text.offset(strlen(text) as isize);
    }
    if buf.offset_from(text) as ::core::ffi::c_longlong
        > 2 as ::core::ffi::c_longlong
    {
        *buf.offset(-(2 as ::core::ffi::c_int) as isize) =
            '}' as i32 as ::core::ffi::c_char;
        *buf.offset(-(1 as ::core::ffi::c_int) as isize) =
            0 as ::core::ffi::c_char;
    }
    return text;
}

unsafe extern "C" fn debuglvhittestinfo(
    lpht: *const LVHITTESTINFO,
) -> *const ::core::ffi::c_char {
    if lpht.is_null() {
        return b"(null)\0" as *const u8 as *const ::core::ffi::c_char;
    }
    return wine_dbg_sprintf(
        b"{pt=%s, flags=0x%x, iItem=%d, iSubItem=%d}\0" as *const u8 as *const ::core::ffi::c_char,
        wine_dbgstr_point(core::ptr::addr_of!((*lpht).pt)),
        (*lpht).flags,
        (*lpht).iItem,
        (*lpht).iSubItem,
    );
}

unsafe extern "C" fn debugscrollcode(nScrollCode: ::core::ffi::c_int) -> LPCSTR {
    match (((((((((nScrollCode) as u32) as i32) as u32) as i32) as u32) as i32) as u32) as i32) as u32 {
        SB_LINELEFT => return b"SB_LINELEFT\0" as *const u8 as LPCSTR,
        SB_LINERIGHT => return b"SB_LINERIGHT\0" as *const u8 as LPCSTR,
        SB_PAGELEFT => return b"SB_PAGELEFT\0" as *const u8 as LPCSTR,
        SB_PAGERIGHT => return b"SB_PAGERIGHT\0" as *const u8 as LPCSTR,
        SB_THUMBPOSITION => return b"SB_THUMBPOSITION\0" as *const u8 as LPCSTR,
        SB_THUMBTRACK => return b"SB_THUMBTRACK\0" as *const u8 as LPCSTR,
        SB_ENDSCROLL => return b"SB_ENDSCROLL\0" as *const u8 as LPCSTR,
        _ if (nScrollCode as u32) == (SB_INTERNAL as u32) => return b"SB_INTERNAL\0" as *const u8 as LPCSTR,
        _ => return b"unknown\0" as *const u8 as LPCSTR,
    }
}

unsafe extern "C" fn get_ansi_notification(
    mut unicodeNotificationCode: UINT,
) -> ::core::ffi::c_int {
    match (unicodeNotificationCode) as i32 {
        LVN_BEGINLABELEDITA | LVN_BEGINLABELEDITW => {
            return LVN_BEGINLABELEDITA as ::core::ffi::c_int;
        }
        LVN_ENDLABELEDITA | LVN_ENDLABELEDITW => {
            return LVN_ENDLABELEDITA as ::core::ffi::c_int;
        }
        LVN_GETDISPINFOA | LVN_GETDISPINFOW => {
            return LVN_GETDISPINFOA as ::core::ffi::c_int;
        }
        LVN_SETDISPINFOA | LVN_SETDISPINFOW => {
            return LVN_SETDISPINFOA as ::core::ffi::c_int;
        }
        LVN_ODFINDITEMA | LVN_ODFINDITEMW => {
            return LVN_ODFINDITEMA as ::core::ffi::c_int;
        }
        LVN_GETINFOTIPA | LVN_GETINFOTIPW => {
            return LVN_GETINFOTIPA as ::core::ffi::c_int;
        }
        HDN_TRACKA | HDN_TRACKW => {
            return HDN_TRACKA as ::core::ffi::c_int;
        }
        HDN_ENDTRACKA | HDN_ENDTRACKW => {
            return HDN_ENDTRACKA as ::core::ffi::c_int;
        }
        HDN_BEGINDRAG => return HDN_BEGINDRAG as ::core::ffi::c_int,
        HDN_ENDDRAG => return HDN_ENDDRAG as ::core::ffi::c_int,
        HDN_ITEMCHANGINGA | HDN_ITEMCHANGINGW => {
            return HDN_ITEMCHANGINGA as ::core::ffi::c_int;
        }
        HDN_ITEMCHANGEDA | HDN_ITEMCHANGEDW => {
            return HDN_ITEMCHANGEDA as ::core::ffi::c_int;
        }
        HDN_ITEMCLICKA | HDN_ITEMCLICKW => {
            return HDN_ITEMCLICKA as ::core::ffi::c_int;
        }
        HDN_DIVIDERDBLCLICKA | HDN_DIVIDERDBLCLICKW => {
            return HDN_DIVIDERDBLCLICKA as ::core::ffi::c_int;
        }
        _ => {}
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"get_ansi_notification\0" as *const u8 as *const ::core::ffi::c_char,
            777 as ::core::ffi::c_int,
            b"unknown notification %x\n\0" as *const u8 as *const ::core::ffi::c_char,
            unicodeNotificationCode,
        );
    }
    return unicodeNotificationCode as ::core::ffi::c_int;
}

unsafe extern "C" fn notify_forward_header(
    mut infoPtr: *const LISTVIEW_INFO,
    mut lpnmhW: *mut NMHEADERW,
) -> LRESULT {
    let mut text: LPCWSTR = ::core::ptr::null::<WCHAR>();
    let mut filter: LPCWSTR = ::core::ptr::null::<WCHAR>();
    let mut ret: LRESULT = 0;
    let mut lpnmh: *mut NMHEADERA = lpnmhW as *mut NMHEADERA;
    if ((*infoPtr).notifyFormat as ::core::ffi::c_int) as u32 == NFR_UNICODE {
        return SendMessageW(
            (*infoPtr).hwndNotify,
            WM_NOTIFY as UINT,
            (*lpnmh).hdr.idFrom as WPARAM,
            lpnmh as LPARAM,
        );
    }
    if !(*lpnmh).pitem.is_null() {
        if (*(*lpnmh).pitem).mask & HDI_TEXT as UINT != 0 {
            text = (*(*lpnmh).pitem).pszText as LPCWSTR;
            (*(*lpnmh).pitem).pszText = ::core::ptr::null_mut::<CHAR>();
            Str_SetPtrWtoA(core::ptr::addr_of_mut!((*(*lpnmh).pitem).pszText), text);
        }
        if (*(*lpnmh).pitem).mask & HDI_FILTER as UINT != 0
            && (*(*lpnmh).pitem).type_ == HDFT_ISSTRING as UINT
            && !(*(*lpnmh).pitem).pvFilter.is_null()
        {
            filter = (*((*(*lpnmh).pitem).pvFilter as *mut HD_TEXTFILTERA)).pszText as LPCWSTR;
            (*((*(*lpnmh).pitem).pvFilter as *mut HD_TEXTFILTERA)).pszText =
                ::core::ptr::null_mut::<CHAR>();
            Str_SetPtrWtoA(
                core::ptr::addr_of_mut!(
                    (*((*(*lpnmh).pitem).pvFilter as *mut HD_TEXTFILTERA)).pszText
                ),
                filter,
            );
        }
    }
    (*lpnmh).hdr.code = get_ansi_notification((*lpnmh).hdr.code) as UINT;
    ret = SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        (*lpnmh).hdr.idFrom as WPARAM,
        lpnmh as LPARAM,
    );
    if !text.is_null() {
        Free((*(*lpnmh).pitem).pszText as LPVOID);
        (*(*lpnmh).pitem).pszText = text as LPSTR;
    }
    if !filter.is_null() {
        Free(
            (*((*(*lpnmh).pitem).pvFilter as *mut HD_TEXTFILTERA)).pszText as LPVOID,
        );
        (*((*(*lpnmh).pitem).pvFilter as *mut HD_TEXTFILTERA)).pszText = filter as LPSTR;
    }
    return ret;
}

unsafe extern "C" fn notify_hdr(infoPtr: *const LISTVIEW_INFO, code: INT, pnmh: LPNMHDR) -> LRESULT {
    let mut result: LRESULT = 0;
    (*pnmh).hwndFrom = (*infoPtr).hwndSelf;
    (*pnmh).idFrom = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID) as UINT_PTR;
    (*pnmh).code = code as UINT;
    result = SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        (*pnmh).idFrom as WPARAM,
        pnmh as LPARAM,
    );
    return result;
}

unsafe extern "C" fn notify(infoPtr: *const LISTVIEW_INFO, code: INT) -> BOOL {
    let mut nmh: NMHDR = core::mem::zeroed();
    let mut hwnd: HWND = (*infoPtr).hwndSelf;
    notify_hdr(infoPtr, code, core::ptr::addr_of_mut!(nmh));
    return IsWindow(hwnd);
}

unsafe extern "C" fn notify_itemactivate(infoPtr: *const LISTVIEW_INFO, htInfo: *const LVHITTESTINFO) {
    let mut nmia: NMITEMACTIVATE = core::mem::zeroed();
    let mut item: LVITEMW = core::mem::zeroed();

    nmia.uNewState = 0 as UINT;
    nmia.uOldState = 0 as UINT;
    nmia.uChanged = 0 as UINT;
    nmia.uKeyFlags = 0 as UINT;

    item.mask = (LVIF_PARAM | LVIF_STATE) as UINT;
    item.iItem = (*htInfo).iItem;
    item.iSubItem = 0 as INT;
    item.stateMask = (-(1 as INT)) as UINT;
    if LISTVIEW_GetItemT(infoPtr, &raw mut item, TRUE) != 0 {
        nmia.lParam = item.lParam;
        nmia.uOldState = item.state;
        nmia.uNewState = item.state | LVIS_ACTIVATING as UINT;
        nmia.uChanged = LVIF_STATE as UINT;
    }

    nmia.iItem = (*htInfo).iItem;
    nmia.iSubItem = (*htInfo).iSubItem;
    nmia.ptAction = (*htInfo).pt;

    if (GetKeyState(VK_SHIFT as INT) as INT & 0x8000 as INT) != 0 {
        nmia.uKeyFlags |= LVKF_SHIFT as UINT;
    }
    if (GetKeyState(VK_CONTROL as INT) as INT & 0x8000 as INT) != 0 {
        nmia.uKeyFlags |= LVKF_CONTROL as UINT;
    }
    if (GetKeyState(VK_MENU as INT) as INT & 0x8000 as INT) != 0 {
        nmia.uKeyFlags |= LVKF_ALT as UINT;
    }

    notify_hdr(infoPtr, LVN_ITEMACTIVATE as INT, &raw mut nmia as LPNMHDR);
}

unsafe extern "C" fn notify_listview(
    infoPtr: *const LISTVIEW_INFO,
    code: INT,
    plvnm: LPNMLISTVIEW,
) -> LRESULT {
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"notify_listview\0" as *const u8 as *const ::core::ffi::c_char,
            891 as ::core::ffi::c_int,
            b"(code=%d, plvnm=%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            code,
            debugnmlistview(plvnm as *const NMLISTVIEW),
        );
    }
    return notify_hdr(infoPtr, code, plvnm as LPNMHDR);
}

unsafe extern "C" fn notify_click(
    infoPtr: *const LISTVIEW_INFO,
    code: INT,
    lvht: *const LVHITTESTINFO,
) -> BOOL {
    let mut nmia: NMITEMACTIVATE = tagNMITEMACTIVATE {
        hdr: tagNMHDR {
            hwndFrom: ::core::ptr::null_mut::<HWND__>(),
            idFrom: 0,
            code: 0,
        },
        iItem: 0,
        iSubItem: 0,
        uNewState: 0,
        uOldState: 0,
        uChanged: 0,
        ptAction: tagPOINT { x: 0, y: 0 },
        lParam: 0,
        uKeyFlags: 0,
    };
    let mut item: LVITEMW = tagLVITEMW {
        mask: 0,
        iItem: 0,
        iSubItem: 0,
        state: 0,
        stateMask: 0,
        pszText: ::core::ptr::null_mut::<WCHAR>(),
        cchTextMax: 0,
        iImage: 0,
        lParam: 0,
        iIndent: 0,
        iGroupId: 0,
        cColumns: 0,
        puColumns: ::core::ptr::null_mut::<::core::ffi::c_uint>(),
    };
    let hwnd: HWND = (*infoPtr).hwndSelf;
    let mut ret: LRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"notify_click\0" as *const u8 as *const ::core::ffi::c_char,
            903 as ::core::ffi::c_int,
            b"code=%d, lvht=%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            code,
            debuglvhittestinfo(lvht),
        );
    }
    ZeroMemory!(
        &raw mut nmia as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<NMITEMACTIVATE>() as size_t,
    );
    nmia.iItem = (*lvht).iItem;
    nmia.iSubItem = (*lvht).iSubItem;
    nmia.ptAction = (*lvht).pt;
    item.mask = LVIF_PARAM as UINT;
    item.iItem = (*lvht).iItem;
    item.iSubItem = 0 as ::core::ffi::c_int;
    if LISTVIEW_GetItemT(infoPtr, &raw mut item, TRUE) != 0 {
        nmia.lParam = item.lParam;
    }
    ret = notify_hdr(infoPtr, code, &raw mut nmia as LPNMHDR);
    return (IsWindow(hwnd) != 0
        && (if code as ::core::ffi::c_uint == NM_RCLICK as ::core::ffi::c_uint {
            (ret == 0) as ::core::ffi::c_int
        } else {
            TRUE as ::core::ffi::c_int
        }) != 0) as ::core::ffi::c_int;
}

unsafe extern "C" fn notify_deleteitem(infoPtr: *const LISTVIEW_INFO, nItem: INT) -> BOOL {
    let mut nmlv: NMLISTVIEW = core::mem::zeroed();
    let mut item: LVITEMW = core::mem::zeroed();
    let hwnd: HWND = (*infoPtr).hwndSelf;

    ZeroMemory!(
        core::ptr::addr_of_mut!(nmlv) as *mut ::core::ffi::c_void,
        core::mem::size_of::<NMLISTVIEW>(),
    );
    nmlv.iItem = nItem as ::core::ffi::c_int;
    item.mask = LVIF_PARAM as UINT;
    item.iItem = nItem as ::core::ffi::c_int;
    item.iSubItem = 0 as ::core::ffi::c_int;
    if LISTVIEW_GetItemT(infoPtr, core::ptr::addr_of_mut!(item), TRUE) != 0 {
        nmlv.lParam = item.lParam;
    }
    notify_listview(infoPtr, LVN_DELETEITEM as INT, core::ptr::addr_of_mut!(nmlv));
    return IsWindow(hwnd);
}

unsafe extern "C" fn notify_dispinfoT(
    infoPtr: *const LISTVIEW_INFO,
    mut code: UINT,
    pdi: LPNMLVDISPINFOW,
    isW: BOOL,
) -> BOOL {
    let mut length: INT = 0;
    let mut ret_length: INT = 0;
    let mut buffer: LPWSTR = core::ptr::null_mut();
    let mut ret_text: LPWSTR = core::ptr::null_mut();
    let mut return_ansi: BOOL = FALSE;
    let mut return_unicode: BOOL = FALSE;
    let mut ret: BOOL = 0;

    if (((*pdi).item.mask & LVIF_TEXT) != 0)
        && is_text((*pdi).item.pszText as LPCWSTR) != 0
    {
        return_unicode = (isW != 0
            && ((*infoPtr).notifyFormat as ::core::ffi::c_int) as u32 == NFR_ANSI)
            as ::core::ffi::c_int as BOOL;
        return_ansi = (isW == 0
            && ((*infoPtr).notifyFormat as ::core::ffi::c_int) as u32 == NFR_UNICODE)
            as ::core::ffi::c_int as BOOL;
    }

    ret_length = (*pdi).item.cchTextMax as INT;
    ret_text = (*pdi).item.pszText;

    if return_unicode != 0 || return_ansi != 0 {
        if code != (LVN_GETDISPINFOW) as u32 {
            length = (if return_ansi != 0 {
                MultiByteToWideChar(
                    CP_ACP,
                    0,
                    (*pdi).item.pszText as LPCSTR,
                    -(1),
                    core::ptr::null_mut::<WCHAR>(),
                    0,
                )
            } else {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    (*pdi).item.pszText as LPCWSTR,
                    -(1),
                    core::ptr::null_mut::<CHAR>(),
                    0,
                    core::ptr::null::<CHAR>(),
                    core::ptr::null_mut::<BOOL>(),
                )
            }) as INT;
        } else {
            length = (*pdi).item.cchTextMax as INT;
            *(*pdi).item.pszText = 0 as WCHAR; /* make sure we don't process garbage */
        }

        buffer = Alloc(
            (if return_ansi != 0 {
                ::core::mem::size_of::<WCHAR>() as usize
            } else {
                core::mem::size_of::<CHAR>() as usize
            })
            .wrapping_mul(length as usize) as DWORD,
        ) as LPWSTR;
        if buffer.is_null() {
            return FALSE;
        }

        if return_ansi != 0 {
            MultiByteToWideChar(
                CP_ACP,
                0,
                (*pdi).item.pszText as LPCSTR,
                -(1),
                buffer,
                length as ::core::ffi::c_int,
            );
        } else {
            WideCharToMultiByte(
                CP_ACP,
                0,
                (*pdi).item.pszText as LPCWSTR,
                -(1),
                buffer as LPSTR,
                length as ::core::ffi::c_int,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            );
        }

        (*pdi).item.pszText = buffer;
        (*pdi).item.cchTextMax = length as ::core::ffi::c_int;
    }

    if ((*infoPtr).notifyFormat as ::core::ffi::c_int) as u32 == NFR_ANSI {
        code = get_ansi_notification(code) as UINT;
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
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"notify_dispinfoT\0" as *const u8 as *const ::core::ffi::c_char,
            988 as ::core::ffi::c_int,
            b" pdi->item=%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            debuglvitem_t(
                core::ptr::addr_of_mut!((*pdi).item),
                ((*infoPtr).notifyFormat as ::core::ffi::c_int != 1 as ::core::ffi::c_int)
                    as ::core::ffi::c_int,
            ),
        );
    }

    ret = notify_hdr(infoPtr, code as INT, core::ptr::addr_of_mut!((*pdi).hdr)) as BOOL;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"notify_dispinfoT\0" as *const u8 as *const ::core::ffi::c_char,
            990 as ::core::ffi::c_int,
            b" resulting code=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*pdi).hdr.code,
        );
    }

    if return_ansi != 0 || return_unicode != 0 {
        if return_ansi != 0 && (*pdi).hdr.code == (LVN_GETDISPINFOA) as u32 {
            strcpy(
                ret_text as *mut ::core::ffi::c_char,
                (*pdi).item.pszText as *mut ::core::ffi::c_char,
            );
        } else if return_unicode != 0 && (*pdi).hdr.code == (LVN_GETDISPINFOW) as u32 {
            lstrcpyW(ret_text, (*pdi).item.pszText);
        } else if return_ansi != 0 {
            /* note : pointer can be changed by app ! */
            WideCharToMultiByte(
                CP_ACP,
                0,
                (*pdi).item.pszText as LPCWSTR,
                -(1),
                ret_text as LPSTR,
                ret_length as ::core::ffi::c_int,
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            );
        } else {
            MultiByteToWideChar(
                CP_ACP,
                0,
                (*pdi).item.pszText as LPSTR as LPCSTR,
                -(1),
                ret_text,
                ret_length as ::core::ffi::c_int,
            );
        }

        (*pdi).item.pszText = ret_text; /* restores our buffer */
        (*pdi).item.cchTextMax = ret_length as ::core::ffi::c_int;

        Free(buffer as LPVOID);
        return ret;
    }

    /* if dispinfo holder changed notification code then convert */
    if isW == 0
        && (*pdi).hdr.code == (LVN_GETDISPINFOW) as u32
        && ((*pdi).item.mask & LVIF_TEXT) != 0
    {
        length = WideCharToMultiByte(
            CP_ACP,
            0,
            (*pdi).item.pszText as LPCWSTR,
            -(1),
            core::ptr::null_mut::<CHAR>(),
            0,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) as INT;

        buffer = Alloc(
            (length as usize).wrapping_mul(core::mem::size_of::<CHAR>() as usize) as DWORD,
        ) as LPWSTR;
        if buffer.is_null() {
            return FALSE;
        }

        WideCharToMultiByte(
            CP_ACP,
            0,
            (*pdi).item.pszText as LPCWSTR,
            -(1),
            buffer as LPSTR,
            ret_length as ::core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );

        strcpy((*pdi).item.pszText as LPSTR, buffer as LPSTR);
        Free(buffer as LPVOID);
    }

    return ret;
}

unsafe extern "C" fn customdraw_fill(lpnmlvcd: *mut NMLVCUSTOMDRAW, infoPtr: *const LISTVIEW_INFO, hdc: HDC, rcBounds: *const RECT, lplvItem: *const LVITEMW) {
    ZeroMemory!(lpnmlvcd as *mut core::ffi::c_void, core::mem::size_of::<NMLVCUSTOMDRAW>());
    (*lpnmlvcd).nmcd.hdc = hdc;
    (*lpnmlvcd).nmcd.rc = *rcBounds;
    (*lpnmlvcd).clrTextBk = (*infoPtr).clrTextBk;
    (*lpnmlvcd).clrText = (*infoPtr).clrText;
    if lplvItem.is_null() {
        return;
    }
    (*lpnmlvcd).nmcd.dwItemSpec = ((*lplvItem).iItem + 1) as DWORD_PTR;
    (*lpnmlvcd).iSubItem = (*lplvItem).iSubItem;
    if (*lplvItem).state & LVIS_SELECTED as UINT != 0 {
        (*lpnmlvcd).nmcd.uItemState |= CDIS_SELECTED as UINT;
    }
    if (*lplvItem).state & LVIS_FOCUSED as UINT != 0 {
        (*lpnmlvcd).nmcd.uItemState |= CDIS_FOCUS as UINT;
    }
    if (*lplvItem).iItem == (*infoPtr).nHotItem {
        (*lpnmlvcd).nmcd.uItemState |= CDIS_HOT as UINT;
    }
    (*lpnmlvcd).nmcd.lItemlParam = (*lplvItem).lParam;
}

unsafe extern "C" fn notify_customdraw(
    infoPtr: *const LISTVIEW_INFO,
    dwDrawStage: DWORD,
    lpnmlvcd: *mut NMLVCUSTOMDRAW,
) -> DWORD {
    let isForItem: BOOL = ((*lpnmlvcd).nmcd.dwItemSpec != 0 as DWORD_PTR) as BOOL;
    let mut result: DWORD = 0;

    (*lpnmlvcd).nmcd.dwDrawStage = dwDrawStage;
    if isForItem != 0 {
        (*lpnmlvcd).nmcd.dwDrawStage |= CDDS_ITEM as DWORD;
    }
    if (*lpnmlvcd).iSubItem != 0 {
        (*lpnmlvcd).nmcd.dwDrawStage |= CDDS_SUBITEM as DWORD;
    }
    if isForItem != 0 {
        (*lpnmlvcd).nmcd.dwItemSpec = (*lpnmlvcd).nmcd.dwItemSpec.wrapping_sub(1);
    }
    result = notify_hdr(infoPtr, NM_CUSTOMDRAW as INT, &raw mut (*lpnmlvcd).nmcd.hdr) as DWORD;
    if isForItem != 0 {
        (*lpnmlvcd).nmcd.dwItemSpec = (*lpnmlvcd).nmcd.dwItemSpec.wrapping_add(1);
    }
    return result;
}

unsafe extern "C" fn prepaint_setup(infoPtr: *const LISTVIEW_INFO, hdc: HDC, lpnmlvcd: *mut NMLVCUSTOMDRAW, SubItem: BOOL) {
    let mut backcolor: COLORREF;
    let mut textcolor: COLORREF;

    /* apparently, for selected items, we have to override the returned values */
    if SubItem == 0 || ((*infoPtr).dwLvExStyle & LVS_EX_FULLROWSELECT) != 0 {
        if (*lpnmlvcd).nmcd.uItemState & CDIS_SELECTED != 0 {
            if (*infoPtr).bFocus != 0 {
                (*lpnmlvcd).clrTextBk = comctl32_color.clrHighlight;
                (*lpnmlvcd).clrText = comctl32_color.clrHighlightText;
            } else if (*infoPtr).dwStyle & LVS_SHOWSELALWAYS != 0 {
                (*lpnmlvcd).clrTextBk = comctl32_color.clr3dFace;
                (*lpnmlvcd).clrText = comctl32_color.clrBtnText;
            }
        }
    }

    backcolor = (*lpnmlvcd).clrTextBk;
    textcolor = (*lpnmlvcd).clrText;

    if backcolor == CLR_DEFAULT {
        backcolor = comctl32_color.clrWindow;
    }
    if textcolor == CLR_DEFAULT {
        textcolor = comctl32_color.clrWindowText;
    }

    /* Set the text attributes */
    if backcolor != CLR_NONE {
        SetBkMode(hdc, (OPAQUE as i32));
        SetBkColor(hdc, backcolor);
    } else {
        SetBkMode(hdc, (TRANSPARENT as i32));
    }
    SetTextColor(hdc, textcolor);
}

unsafe extern "C" fn notify_postpaint(infoPtr: *const LISTVIEW_INFO, lpnmlvcd: *mut NMLVCUSTOMDRAW) -> DWORD {
    return notify_customdraw(infoPtr, CDDS_POSTPAINT as DWORD, lpnmlvcd);
}

unsafe extern "C" fn notify_measureitem(mut infoPtr: *mut LISTVIEW_INFO) -> BOOL {
    let mut mis: MEASUREITEMSTRUCT = core::mem::zeroed();
    mis.CtlType = ODT_LISTVIEW as UINT;
    mis.CtlID = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID as i32) as UINT;
    mis.itemID = (-(1 as i32)) as UINT;
    mis.itemWidth = 0 as UINT;
    mis.itemData = 0 as ULONG_PTR;
    mis.itemHeight = (*infoPtr).nItemHeight as UINT;
    SendMessageW(
        (*infoPtr).hwndNotify,
        WM_MEASUREITEM as UINT,
        mis.CtlID as WPARAM,
        &raw mut mis as LPARAM,
    );
    if (*infoPtr).nItemHeight as UINT != max(mis.itemHeight, 1 as UINT) {
        (*infoPtr).nItemHeight = max(mis.itemHeight, 1 as UINT) as INT;
        (*infoPtr).nMeasureItemHeight = (*infoPtr).nItemHeight;
        return TRUE;
    }
    return FALSE;
}

unsafe extern "C" fn ranges_additem(ranges: RANGES, nItem: INT) -> BOOL {
    let range: RANGE = RANGE {
        lower: nItem,
        upper: nItem + 1,
    };

    return ranges_add(ranges, range);
}

unsafe extern "C" fn ranges_delitem(mut ranges: RANGES, mut nItem: INT) -> BOOL {
    let mut range: RANGE = RANGE {
        lower: nItem,
        upper: nItem + 1,
    };
    return ranges_del(ranges, range);
}

unsafe extern "C" fn iterator_next(i: *mut ITERATOR) -> BOOL {
    if (*i).nItem == -1 {
        (*i).nItem = (*i).nSpecial;
        if (*i).nItem != -1 {
            return TRUE;
        }
    }
    // state 0 = fall into pickarange, 1 = testitem, 2 = pickarange
    let mut at: u8;
    if (*i).nItem == (*i).nSpecial {
        if !(*i).ranges.is_null() {
            (*i).index = 0;
        }
        at = 2;
    } else {
        (*i).nItem += 1;
        at = 1;
    }
    loop {
        match at {
            1 => {
                // testitem:
                if (*i).nItem == (*i).nSpecial {
                    (*i).nItem += 1;
                }
                if (*i).nItem < (*i).range.upper {
                    return TRUE;
                }
                at = 2;
            }
            2 => {
                // pickarange:
                if !(*i).ranges.is_null() {
                    if (*i).index < DPA_GetPtrCount((*(*i).ranges).hdpa) {
                        (*i).range = core::ptr::read(DPA_GetPtr((*(*i).ranges).hdpa, (*i).index) as *mut RANGE);
                        (*i).index += 1;
                    } else {
                        break; // goto end
                    }
                } else if (*i).nItem >= (*i).range.upper {
                    break; // goto end
                }
                (*i).nItem = (*i).range.lower;
                if (*i).nItem >= 0 {
                    at = 1; // goto testitem
                    continue;
                }
                break; // goto end
            }
            _ => break,
        }
    }
    // end:
    (*i).nItem = -1;
    return FALSE;
}

unsafe extern "C" fn iterator_prev(i: *mut ITERATOR) -> BOOL {
    let mut start: BOOL = FALSE;
    let mut at: u8;

    if (*i).nItem == -1 {
        start = TRUE;
        if !(*i).ranges.is_null() {
            (*i).index = DPA_GetPtrCount((*(*i).ranges).hdpa);
        }
        at = 2; // pickarange
    } else {
        if (*i).nItem == (*i).nSpecial {
            (*i).nItem = -1;
            return FALSE;
        }
        at = 1; // testitem
    }

    loop {
        match at {
            1 => {
                // testitem:
                (*i).nItem -= 1;
                if (*i).nItem == (*i).nSpecial {
                    (*i).nItem -= 1;
                }
                if (*i).nItem >= (*i).range.lower {
                    return TRUE;
                }
                at = 2;
                continue;
            }
            _ => {
                // pickarange:
                if !(*i).ranges.is_null() {
                    if (*i).index > 0 {
                        (*i).index -= 1;
                        (*i).range = core::ptr::read(DPA_GetPtr((*(*i).ranges).hdpa, (*i).index) as *mut RANGE);
                    } else {
                        break; // goto end
                    }
                } else if start == 0 && (*i).nItem < (*i).range.lower {
                    break; // goto end
                }

                (*i).nItem = (*i).range.upper;
                if (*i).nItem > 0 {
                    at = 1;
                    continue;
                }
                break; // end
            }
        }
    }

    // end:
    (*i).nItem = (*i).nSpecial;
    return ((*i).nItem != -1) as BOOL;
}

unsafe extern "C" fn iterator_range(i: *const ITERATOR) -> RANGE {
    let mut range: RANGE = RANGE { lower: 0, upper: 0 };

    if (*i).ranges.is_null() {
        return RANGE { lower: (*i).range.lower, upper: (*i).range.upper };
    }

    if DPA_GetPtrCount((*(*i).ranges).hdpa) > 0 {
        range.lower = (*(DPA_GetPtr((*(*i).ranges).hdpa, 0) as *mut RANGE)).lower;
        range.upper = (*(DPA_GetPtr((*(*i).ranges).hdpa, DPA_GetPtrCount((*(*i).ranges).hdpa) - 1) as *mut RANGE)).upper;
    } else {
        range.upper = 0;
        range.lower = range.upper;
    }

    return range;
}

unsafe extern "C" fn iterator_destroy(i: *const ITERATOR) {
    ranges_destroy((*i).ranges);
}

unsafe extern "C" fn iterator_empty(i: *mut ITERATOR) {
    let v: INT = -1;
    ZeroMemory!(i as *mut core::ffi::c_void, core::mem::size_of::<ITERATOR>());
    (*i).nItem = v;
    (*i).nSpecial = v;
    (*i).range.lower = v;
    (*i).range.upper = v;
}

unsafe extern "C" fn iterator_rangeitems(i: *mut ITERATOR, range: RANGE) {
    iterator_empty(i);
    (*i).range = range;
}

unsafe extern "C" fn iterator_rangesitems(i: *mut ITERATOR, ranges: RANGES) {
    iterator_empty(i);
    (*i).ranges = ranges;
}

unsafe extern "C" fn iterator_frameditems_absolute(i: *mut ITERATOR, infoPtr: *const LISTVIEW_INFO, frame: *const RECT) -> BOOL {
    let mut rcItem: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcTemp: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut ranges: RANGES = core::ptr::null_mut();

    /* in case we fail, we want to return an empty iterator */
    iterator_empty(i);

    if (*infoPtr).nItemCount == 0 {
        return TRUE;
    }

    if (*infoPtr).uView == LV_VIEW_ICON || (*infoPtr).uView == LV_VIEW_SMALLICON {
        let mut nItem: INT;

        if (*infoPtr).uView == LV_VIEW_ICON && (*infoPtr).nFocusedItem != -1 {
            LISTVIEW_GetItemBox(infoPtr, (*infoPtr).nFocusedItem, core::ptr::addr_of_mut!(rcItem));
            if IntersectRect(core::ptr::addr_of_mut!(rcTemp), core::ptr::addr_of_mut!(rcItem) as LPCRECT, frame as LPCRECT) != 0 {
                (*i).nSpecial = (*infoPtr).nFocusedItem;
            }
        }
        ranges = ranges_create(50);
        if ranges.is_null() {
            return FALSE;
        }
        iterator_rangesitems(i, ranges);
        /* to do better here, we need to have PosX, and PosY sorted */

        nItem = 0;
        while nItem < (*infoPtr).nItemCount {
            rcItem.left = DPA_GetPtr((*infoPtr).hdpaPosX, nItem) as isize as LONG;
            rcItem.top = DPA_GetPtr((*infoPtr).hdpaPosY, nItem) as isize as LONG;
            rcItem.right = rcItem.left + (*infoPtr).nItemWidth;
            rcItem.bottom = rcItem.top + (*infoPtr).nItemHeight;
            if IntersectRect(core::ptr::addr_of_mut!(rcTemp), core::ptr::addr_of_mut!(rcItem) as LPCRECT, frame as LPCRECT) != 0 {
                ranges_additem((*i).ranges, nItem);
            }
            nItem += 1;
        }
        return TRUE;
    } else if (*infoPtr).uView == LV_VIEW_DETAILS {
        let mut range: RANGE = RANGE { lower: 0, upper: 0 };

        if (*frame).left >= (*infoPtr).nItemWidth {
            return TRUE;
        }
        if (*frame).top >= (*infoPtr).nItemHeight * (*infoPtr).nItemCount {
            return TRUE;
        }

        range.lower = max((*frame).top / (*infoPtr).nItemHeight, 0);
        range.upper = min(((*frame).bottom - 1) / (*infoPtr).nItemHeight, (*infoPtr).nItemCount - 1) + 1;
        if range.upper <= range.lower {
            return TRUE;
        }
        iterator_rangeitems(i, range);
    } else {
        let mut nPerCol: INT = max(((*infoPtr).rcList.bottom - (*infoPtr).rcList.top) / (*infoPtr).nItemHeight, 1);
        let mut nFirstRow: INT = max((*frame).top / (*infoPtr).nItemHeight, 0);
        let mut nLastRow: INT = min(((*frame).bottom - 1) / (*infoPtr).nItemHeight, nPerCol - 1);
        let mut nFirstCol: INT;
        let mut nLastCol: INT;
        let mut lower: INT;
        let mut item_range: RANGE = RANGE { lower: 0, upper: 0 };
        let mut nCol: INT;

        if (*infoPtr).nItemWidth != 0 {
            nFirstCol = max((*frame).left / (*infoPtr).nItemWidth, 0);
            nLastCol = min(((*frame).right - 1) / (*infoPtr).nItemWidth, ((*infoPtr).nItemCount + nPerCol - 1) / nPerCol);
        } else {
            nFirstCol = max((*frame).left, 0);
            nLastCol = min((*frame).right - 1, ((*infoPtr).nItemCount + nPerCol - 1) / nPerCol);
        }

        lower = nFirstCol * nPerCol + nFirstRow;

        if nLastCol < nFirstCol || nLastRow < nFirstRow {
            return TRUE;
        }

        ranges = ranges_create(nLastCol - nFirstCol + 1);
        if ranges.is_null() {
            return FALSE;
        }
        iterator_rangesitems(i, ranges);

        nCol = nFirstCol;
        while nCol <= nLastCol {
            item_range.lower = nCol * nPerCol + nFirstRow;
            if item_range.lower >= (*infoPtr).nItemCount {
                break;
            }
            item_range.upper = min(nCol * nPerCol + nLastRow + 1, (*infoPtr).nItemCount);
            ranges_add((*i).ranges, RANGE { lower: item_range.lower, upper: item_range.upper });
            nCol += 1;
        }
    }

    return TRUE;
}

unsafe extern "C" fn iterator_frameditems(i: *mut ITERATOR, infoPtr: *const LISTVIEW_INFO, lprc: *const RECT) -> BOOL {
    let mut frame: RECT = *lprc;
    let mut Origin: POINT = POINT { x: 0, y: 0 };

    LISTVIEW_GetOrigin(infoPtr, core::ptr::addr_of_mut!(Origin));
    OffsetRect(core::ptr::addr_of_mut!(frame), -Origin.x, -Origin.y);

    return iterator_frameditems_absolute(i, infoPtr, core::ptr::addr_of!(frame));
}

unsafe extern "C" fn iterator_visibleitems(
    mut i: *mut ITERATOR,
    mut infoPtr: *const LISTVIEW_INFO,
    mut hdc: HDC,
) -> BOOL {
    let mut Origin: POINT = POINT { x: 0, y: 0 };
    let mut Position: POINT = POINT { x: 0, y: 0 };
    let mut rcItem: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcClip: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rgntype: INT = 0;

    rgntype = GetClipBox(hdc, core::ptr::addr_of_mut!(rcClip)) as INT;
    if rgntype == (NULLREGION as i32) {
        iterator_empty(i);
        return TRUE;
    }
    if iterator_frameditems(i, infoPtr, core::ptr::addr_of_mut!(rcClip)) == 0 {
        return FALSE;
    }
    if rgntype == (SIMPLEREGION as i32) {
        return TRUE;
    }

    /* first deal with the special item */
    if (*i).nSpecial != -1 {
        LISTVIEW_GetItemBox(infoPtr, (*i).nSpecial, core::ptr::addr_of_mut!(rcItem));
        if RectVisible(hdc, core::ptr::addr_of_mut!(rcItem) as LPCRECT) == 0 {
            (*i).nSpecial = -1;
        }
    }

    /* if we can't deal with the region, we'll just go with the simple range */
    LISTVIEW_GetOrigin(infoPtr, core::ptr::addr_of_mut!(Origin));

    if (*i).ranges.is_null() && (*i).range.lower < (*i).range.upper {
        (*i).ranges = ranges_create(50);
        if (*i).ranges.is_null() {
            return TRUE;
        }
        if ranges_add((*i).ranges, RANGE { lower: (*i).range.lower, upper: (*i).range.upper }) == 0 {
            ranges_destroy((*i).ranges);
            (*i).ranges = core::ptr::null_mut();
            return TRUE;
        }
    }

    /* now delete the invisible items from the list */
    while iterator_next(i) != 0 {
        LISTVIEW_GetItemOrigin(infoPtr, (*i).nItem, core::ptr::addr_of_mut!(Position));
        rcItem.left = if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
            Origin.x
        } else {
            Position.x + Origin.x
        };
        rcItem.top = Position.y + Origin.y;
        rcItem.right = (rcItem.left as INT + (*infoPtr).nItemWidth) as LONG;
        rcItem.bottom = (rcItem.top as INT + (*infoPtr).nItemHeight) as LONG;
        if RectVisible(hdc, core::ptr::addr_of_mut!(rcItem) as LPCRECT) == 0 {
            ranges_delitem((*i).ranges, (*i).nItem);
        }
    }
    /* the iterator should restart on the next iterator_next */

    return TRUE;
}

unsafe extern "C" fn iterator_remove_common_items(iter1: *mut ITERATOR, iter2: *mut ITERATOR) -> BOOL {
    if (*iter1).ranges.is_null() || (*iter2).ranges.is_null() {
        let mut lower: i32 = 0;
        let mut upper: i32 = 0;

        if !(*iter1).ranges.is_null()
            || !(*iter2).ranges.is_null()
            || ((*iter1).range.lower < (*iter2).range.lower && (*iter1).range.upper > (*iter2).range.upper)
            || ((*iter1).range.lower > (*iter2).range.lower && (*iter1).range.upper < (*iter2).range.upper)
        {
            // ERR("result is not a one range iterator\n") trace omitted: debug channel symbols unavailable
            return FALSE;
        }

        if (*iter1).range.lower == -1 || (*iter2).range.lower == -1 {
            return TRUE;
        }

        lower = (*iter1).range.lower;
        upper = (*iter1).range.upper;

        if lower < (*iter2).range.lower {
            (*iter1).range.upper = (*iter2).range.lower;
        } else if upper > (*iter2).range.upper {
            (*iter1).range.lower = (*iter2).range.upper;
        } else {
            (*iter1).range.upper = -1;
            (*iter1).range.lower = (*iter1).range.upper;
        }

        if (*iter2).range.lower < lower {
            (*iter2).range.upper = lower;
        } else if (*iter2).range.upper > upper {
            (*iter2).range.lower = upper;
        } else {
            (*iter2).range.upper = -1;
            (*iter2).range.lower = (*iter2).range.upper;
        }

        return TRUE;
    }

    iterator_next(iter1);
    iterator_next(iter2);

    while (1) != 0 {
        if (*iter1).nItem == -1 || (*iter2).nItem == -1 {
            break;
        }

        if (*iter1).nItem == (*iter2).nItem {
            let delete: i32 = (*iter1).nItem;

            iterator_prev(iter1);
            iterator_prev(iter2);
            ranges_delitem((*iter1).ranges, delete);
            ranges_delitem((*iter2).ranges, delete);
            iterator_next(iter1);
            iterator_next(iter2);
        } else if (*iter1).nItem > (*iter2).nItem {
            iterator_next(iter2);
        } else {
            iterator_next(iter1);
        }
    }

    (*iter1).range.upper = -1;
    (*iter1).range.lower = (*iter1).range.upper;
    (*iter1).nItem = (*iter1).range.lower;
    (*iter2).range.upper = -1;
    (*iter2).range.lower = (*iter2).range.upper;
    (*iter2).nItem = (*iter2).range.lower;
    return TRUE;
}

unsafe extern "C" fn CallWindowProcT(proc_0: WNDPROC, hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, isW: BOOL) -> LRESULT {
    if isW != 0 {
        return CallWindowProcW(proc_0, hwnd, uMsg, wParam, lParam);
    } else {
        return CallWindowProcA(proc_0, hwnd, uMsg, wParam, lParam);
    }
}

unsafe extern "C" fn is_autoarrange(infoPtr: *const LISTVIEW_INFO) -> BOOL {
    return ((((*infoPtr).dwStyle & LVS_AUTOARRANGE) != 0)
        && ((*infoPtr).uView == LV_VIEW_ICON || (*infoPtr).uView == LV_VIEW_SMALLICON)) as i32;
}

unsafe extern "C" fn toggle_checkbox_state(infoPtr: *mut LISTVIEW_INFO, nItem: INT) {
    let mut state: DWORD = STATEIMAGEINDEX!(LISTVIEW_GetItemState(infoPtr, nItem, LVIS_STATEIMAGEMASK) as DWORD);
    if state == 1 || state == 2 {
        let mut lvitem: LVITEMW = core::mem::zeroed();
        state ^= 3;
        lvitem.state = INDEXTOSTATEIMAGEMASK!(state);
        lvitem.stateMask = LVIS_STATEIMAGEMASK;
        LISTVIEW_SetItemState(infoPtr, nItem, &mut lvitem);
    }
}

unsafe extern "C" fn map_style_view(infoPtr: *mut LISTVIEW_INFO) {
    match (*infoPtr).dwStyle & LVS_TYPEMASK {
        LVS_ICON => {
            (*infoPtr).uView = LV_VIEW_ICON;
        }
        LVS_REPORT => {
            (*infoPtr).uView = LV_VIEW_DETAILS;
        }
        LVS_SMALLICON => {
            (*infoPtr).uView = LV_VIEW_SMALLICON;
        }
        LVS_LIST => {
            (*infoPtr).uView = LV_VIEW_LIST;
        }
        _ => {}
    }
}

unsafe extern "C" fn get_next_itemid(infoPtr: *const LISTVIEW_INFO) -> DWORD {
    let count: INT = DPA_GetPtrCount((*infoPtr).hdpaItemIds);
    if count > 0 {
        let lpID: *mut ITEM_ID = DPA_GetPtr((*infoPtr).hdpaItemIds, count - 1) as *mut ITEM_ID;
        return (*lpID).id.wrapping_add(1);
    }
    return 0;
}

unsafe extern "C" fn LISTVIEW_GetColumnInfo(infoPtr: *const LISTVIEW_INFO, nSubItem: INT) -> *mut COLUMN_INFO {
    static mut mainItem: COLUMN_INFO = unsafe { core::mem::zeroed() };

    if nSubItem == 0 && DPA_GetPtrCount((*infoPtr).hdpaColumns) == 0 {
        return &raw mut mainItem;
    }
    assert!(nSubItem >= 0 && nSubItem < DPA_GetPtrCount((*infoPtr).hdpaColumns));

    /* update cached column rectangles */
    if (*infoPtr).colRectsDirty != 0 {
        let mut info: *mut COLUMN_INFO = core::ptr::null_mut();
        let Ptr: *mut LISTVIEW_INFO = infoPtr as *mut LISTVIEW_INFO;
        let mut i: INT = 0;

        while i < DPA_GetPtrCount((*infoPtr).hdpaColumns) {
            info = DPA_GetPtr((*infoPtr).hdpaColumns, i) as *mut COLUMN_INFO;
            SendMessageW(
                (*infoPtr).hwndHeader,
                HDM_GETITEMRECT as UINT,
                i as WPARAM,
                &raw mut (*info).rcHeader as LPARAM,
            );
            i += 1;
        }
        (*Ptr).colRectsDirty = FALSE as BOOL;
    }

    return DPA_GetPtr((*infoPtr).hdpaColumns, nSubItem) as *mut COLUMN_INFO;
}

unsafe extern "C" fn LISTVIEW_CreateHeader(infoPtr: *mut LISTVIEW_INFO) -> INT {
    let mut dFlags: DWORD = (WS_CHILD as DWORD) | (HDS_HORZ as DWORD) | (HDS_FULLDRAG as DWORD) | (HDS_DRAGDROP as DWORD);
    let mut hInst: HINSTANCE;

    if !(*infoPtr).hwndHeader.is_null() {
        return 0 as INT;
    }

    /* setup creation flags */
    dFlags |= if ((LVS_NOSORTHEADER as DWORD) & (*infoPtr).dwStyle) != 0 {
        0 as DWORD
    } else {
        HDS_BUTTONS as DWORD
    };
    dFlags |= if ((LVS_NOCOLUMNHEADER as DWORD) & (*infoPtr).dwStyle) != 0 {
        HDS_HIDDEN as DWORD
    } else {
        0 as DWORD
    };

    hInst = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_HINSTANCE as INT) as HINSTANCE;

    /* create header */
    (*infoPtr).hwndHeader = CreateWindowW!(
        WC_HEADERW.as_ptr(),
        core::ptr::null_mut(),
        dFlags,
        0 as INT,
        0 as INT,
        0 as INT,
        0 as INT,
        (*infoPtr).hwndSelf,
        core::ptr::null_mut(),
        hInst,
        core::ptr::null_mut(),
    );
    if (*infoPtr).hwndHeader.is_null() {
        return -(1 as INT);
    }

    /* set header unicode format */
    SendMessageW(
        (*infoPtr).hwndHeader,
        HDM_SETUNICODEFORMAT as UINT,
        TRUE as WPARAM,
        0 as LPARAM,
    );

    /* set header font */
    SendMessageW(
        (*infoPtr).hwndHeader,
        WM_SETFONT as UINT,
        (*infoPtr).hFont as WPARAM,
        TRUE as LPARAM,
    );

    /* set header image list */
    if !(*infoPtr).himlSmall.is_null() {
        SendMessageW(
            (*infoPtr).hwndHeader,
            HDM_SETIMAGELIST as UINT,
            0 as WPARAM,
            (*infoPtr).himlSmall as LPARAM,
        );
    }

    LISTVIEW_UpdateSize(infoPtr);

    return 0 as INT;
}

unsafe extern "C" fn LISTVIEW_GetHeaderRect(infoPtr: *const LISTVIEW_INFO, nSubItem: INT, lprc: LPRECT) {
    *lprc = (*LISTVIEW_GetColumnInfo(infoPtr, nSubItem)).rcHeader;
}

unsafe extern "C" fn LISTVIEW_IsHeaderEnabled(infoPtr: *const LISTVIEW_INFO) -> BOOL {
    return ((((*infoPtr).uView == LV_VIEW_DETAILS as DWORD)
        || ((*infoPtr).dwLvExStyle & (LVS_EX_HEADERINALLVIEWS as DWORD)) != 0)
        && ((*infoPtr).dwStyle & (LVS_NOCOLUMNHEADER as DWORD)) == 0) as BOOL;
}

unsafe extern "C" fn LISTVIEW_GetItemW(infoPtr: *const LISTVIEW_INFO, lpLVItem: LPLVITEMW) -> BOOL {
    return LISTVIEW_GetItemT(infoPtr, lpLVItem, TRUE);
}

unsafe extern "C" fn LISTVIEW_DrawFocusRect(infoPtr: *const LISTVIEW_INFO, hdc: HDC) -> BOOL {
    let mut Ret: BOOL = FALSE;

    if (*infoPtr).rcFocus.left < (*infoPtr).rcFocus.right {
        let mut dwOldBkColor: DWORD;
        let mut dwOldTextColor: DWORD;

        dwOldBkColor = SetBkColor(hdc, RGB!(255, 255, 255)) as DWORD;
        dwOldTextColor = SetBkColor(hdc, RGB!(0, 0, 0)) as DWORD;
        Ret = DrawFocusRect(hdc, core::ptr::addr_of!((*infoPtr).rcFocus));
        SetBkColor(hdc, dwOldBkColor as COLORREF);
        SetBkColor(hdc, dwOldTextColor as COLORREF);
    }
    return Ret;
}

unsafe extern "C" fn is_redrawing(infoPtr: *const LISTVIEW_INFO) -> BOOL {
    (*infoPtr).redraw
}

unsafe extern "C" fn LISTVIEW_InvalidateRect(infoPtr: *const LISTVIEW_INFO, rect: *const RECT) {
    if is_redrawing(infoPtr) == 0 {
        return;
    }
    InvalidateRect((*infoPtr).hwndSelf, rect, TRUE);
}

unsafe extern "C" fn LISTVIEW_InvalidateItem(infoPtr: *const LISTVIEW_INFO, nItem: INT) {
    let mut rcBox: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if is_redrawing(infoPtr) == 0 || nItem < 0 || nItem >= (*infoPtr).nItemCount {
        return;
    }
    LISTVIEW_GetItemBox(infoPtr, nItem, core::ptr::addr_of_mut!(rcBox));
    LISTVIEW_InvalidateRect(infoPtr, core::ptr::addr_of!(rcBox));
}

unsafe extern "C" fn LISTVIEW_InvalidateSubItem(infoPtr: *const LISTVIEW_INFO, nItem: INT, nSubItem: INT) {
    let mut Origin: POINT = POINT { x: 0, y: 0 };
    let mut Position: POINT = POINT { x: 0, y: 0 };
    let mut rcBox: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    if is_redrawing(infoPtr) == 0 {
        return;
    }
    assert!((*infoPtr).uView == LV_VIEW_DETAILS);
    LISTVIEW_GetOrigin(infoPtr, &raw mut Origin);
    LISTVIEW_GetItemOrigin(infoPtr, nItem, &raw mut Position);
    LISTVIEW_GetHeaderRect(infoPtr, nSubItem, &raw mut rcBox);
    rcBox.top = 0;
    rcBox.bottom = (*infoPtr).nItemHeight as LONG;
    OffsetRect(&raw mut rcBox, Origin.x, Origin.y + Position.y);
    LISTVIEW_InvalidateRect(infoPtr, &raw mut rcBox);
}

unsafe extern "C" fn LISTVIEW_InvalidateList(infoPtr: *const LISTVIEW_INFO) {
    LISTVIEW_InvalidateRect(infoPtr, core::ptr::null());
}

unsafe extern "C" fn LISTVIEW_InvalidateColumn(infoPtr: *const LISTVIEW_INFO, nColumn: INT) {
    let mut rcCol: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    if is_redrawing(infoPtr) == 0 { return; }
    LISTVIEW_GetHeaderRect(infoPtr, nColumn, core::ptr::addr_of_mut!(rcCol));
    rcCol.top = (*infoPtr).rcList.top;
    rcCol.bottom = (*infoPtr).rcList.bottom;
    LISTVIEW_InvalidateRect(infoPtr, core::ptr::addr_of_mut!(rcCol));
}

unsafe extern "C" fn LISTVIEW_GetCountPerRow(infoPtr: *const LISTVIEW_INFO) -> INT {
    let nListWidth: INT = (*infoPtr).rcList.right - (*infoPtr).rcList.left;

    return max(
        nListWidth
            / (if (*infoPtr).nItemWidth != 0 {
                (*infoPtr).nItemWidth
            } else {
                1
            }),
        1,
    );
}

unsafe extern "C" fn LISTVIEW_GetCountPerColumn(infoPtr: *const LISTVIEW_INFO) -> INT {
    let nListHeight: INT = (*infoPtr).rcList.bottom - (*infoPtr).rcList.top;

    return if (*infoPtr).nItemHeight != 0 {
        max(nListHeight / (*infoPtr).nItemHeight, 1)
    } else {
        0
    };
}

unsafe extern "C" fn LISTVIEW_ProcessLetterKeys(
    infoPtr: *mut LISTVIEW_INFO,
    mut charCode: WPARAM,
    keyData: LPARAM,
) -> INT {
    let mut buffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut prevTime: DWORD = 0;
    let mut item: LVITEMW = core::mem::zeroed();
    let mut startidx: INT = 0;
    let mut nItem: INT = 0;
    let mut diff: INT = 0;

    /* simple parameter checking */
    if charCode == 0 || keyData == 0 || (*infoPtr).nItemCount == 0 {
        return 0;
    }

    /* only allow the valid WM_CHARs through */
    if iswalnum(charCode as _) == 0
        && charCode != '.' as WPARAM
        && charCode != '`' as WPARAM
        && charCode != '!' as WPARAM
        && charCode != '@' as WPARAM
        && charCode != '#' as WPARAM
        && charCode != '$' as WPARAM
        && charCode != '%' as WPARAM
        && charCode != '^' as WPARAM
        && charCode != '&' as WPARAM
        && charCode != '*' as WPARAM
        && charCode != '(' as WPARAM
        && charCode != ')' as WPARAM
        && charCode != '-' as WPARAM
        && charCode != '_' as WPARAM
        && charCode != '+' as WPARAM
        && charCode != '=' as WPARAM
        && charCode != '\\' as WPARAM
        && charCode != ']' as WPARAM
        && charCode != '}' as WPARAM
        && charCode != '[' as WPARAM
        && charCode != '{' as WPARAM
        && charCode != '/' as WPARAM
        && charCode != '?' as WPARAM
        && charCode != '>' as WPARAM
        && charCode != '<' as WPARAM
        && charCode != ',' as WPARAM
        && charCode != '~' as WPARAM
    {
        return 0;
    }

    /* update the search parameters */
    prevTime = (*infoPtr).lastKeyPressTimestamp;
    (*infoPtr).lastKeyPressTimestamp = GetTickCount();
    diff = (*infoPtr).lastKeyPressTimestamp.wrapping_sub(prevTime) as INT;

    if diff >= 0 && diff < KEY_DELAY {
        if (((*infoPtr).nSearchParamLength) as u32) < MAX_PATH - 1 {
            let fresh = (*infoPtr).nSearchParamLength;
            (*infoPtr).nSearchParamLength += 1;
            (*infoPtr).szSearchParam[fresh as usize] = charCode as WCHAR;
        }

        if (*infoPtr).charCode != charCode {
            charCode = 0;
            (*infoPtr).charCode = charCode;
        }
    } else {
        (*infoPtr).charCode = charCode;
        (*infoPtr).szSearchParam[0 as usize] = charCode as WCHAR;
        (*infoPtr).nSearchParamLength = 1;
    }

    /* should start from next after focused item, so next item that matches
       will be selected, if there isn't any and focused matches it will be selected
       on second search stage from beginning of the list */
    if (*infoPtr).nFocusedItem >= 0 && (*infoPtr).nItemCount > 1 {
        /* with some accumulated search data available start with current focus, otherwise
           it's excluded from search */
        startidx = if (*infoPtr).nSearchParamLength > 1 {
            (*infoPtr).nFocusedItem
        } else {
            (*infoPtr).nFocusedItem + 1
        };
        if startidx == (*infoPtr).nItemCount {
            startidx = 0;
        }
    } else {
        startidx = 0;
    }

    /* let application handle this for virtual listview */
    if ((*infoPtr).dwStyle & LVS_OWNERDATA) != 0 {
        let mut nmlv: NMLVFINDITEMW = core::mem::zeroed();

        memset(
            core::ptr::addr_of_mut!(nmlv.lvfi) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<LVFINDINFOW>(),
        );
        nmlv.lvfi.flags = (LVFI_WRAP | LVFI_PARTIAL) as UINT;
        nmlv.lvfi.psz = core::ptr::addr_of_mut!((*infoPtr).szSearchParam) as *mut WCHAR as LPCWSTR;
        nmlv.iStart = startidx;

        (*infoPtr).szSearchParam[(*infoPtr).nSearchParamLength as usize] = 0;

        nItem = notify_hdr(
            infoPtr,
            LVN_ODFINDITEMW as INT,
            core::ptr::addr_of_mut!(nmlv.hdr) as LPNMHDR,
        ) as INT;
    } else {
        let mut i: INT = startidx;
        let mut endidx: INT = 0;

        /* and search from the current position */
        nItem = -(1);
        endidx = (*infoPtr).nItemCount;

        /* first search in [startidx, endidx), on failure continue in [0, startidx) */
        while 1 != 0 {
            /* start from first item if not found with >= startidx */
            if i == (*infoPtr).nItemCount && startidx > 0 {
                endidx = startidx;
                startidx = 0;
            }

            i = startidx;
            while i < endidx {
                /* retrieve text */
                item.mask = LVIF_TEXT;
                item.iItem = i;
                item.iSubItem = 0;
                item.pszText = core::ptr::addr_of_mut!(buffer) as *mut WCHAR as LPWSTR;
                item.cchTextMax = (MAX_PATH) as i32;
                if LISTVIEW_GetItemW(infoPtr, core::ptr::addr_of_mut!(item)) == 0 {
                    return 0;
                }

                if wcsnicmp(
                    item.pszText as *const WCHAR,
                    core::ptr::addr_of_mut!((*infoPtr).szSearchParam) as *mut WCHAR,
                    (*infoPtr).nSearchParamLength as usize,
                ) == 0
                {
                    nItem = i;
                    break;
                } else {
                    /* this is used to find first char match when search string is not available yet,
                       otherwise every WM_CHAR will search to next item by first char, ignoring that we're
                       already waiting for user to complete a string */
                    if nItem == -(1)
                        && (*infoPtr).nSearchParamLength == 1
                        && wcsnicmp(
                            item.pszText as *const WCHAR,
                            core::ptr::addr_of_mut!((*infoPtr).szSearchParam) as *mut WCHAR,
                            1 as usize,
                        ) == 0
                    {
                        /* this would work but we must keep looking for a longer match */
                        nItem = i;
                    }
                    i += 1;
                }
            }

            if nItem != -(1)  /* found something */
                || endidx != (*infoPtr).nItemCount  /* second search done */
                || (startidx == 0 && endidx == (*infoPtr).nItemCount)
            /* full range for first search */
            {
                break;
            }
        }
    }

    if nItem != -(1) {
        LISTVIEW_KeySelection(infoPtr, nItem, 0 as DWORD);
    }

    return 0;
}

unsafe extern "C" fn LISTVIEW_UpdateHeaderSize(infoPtr: *const LISTVIEW_INFO, nNewScrollPos: INT) {
    let mut winRect: RECT = core::mem::zeroed();
    let mut point: [POINT; 2] = [POINT { x: 0, y: 0 }; 2];

    if (*infoPtr).hwndHeader.is_null() {
        return;
    }

    GetWindowRect((*infoPtr).hwndHeader, core::ptr::addr_of_mut!(winRect));
    point[0 as usize].x = winRect.left;
    point[0 as usize].y = winRect.top;
    point[1 as usize].x = winRect.right;
    point[1 as usize].y = winRect.bottom;

    MapWindowPoints(
        HWND_DESKTOP,
        (*infoPtr).hwndSelf,
        core::ptr::addr_of_mut!(point) as LPPOINT,
        2 as UINT,
    );
    point[0 as usize].x = -nNewScrollPos as LONG;
    point[1 as usize].x += nNewScrollPos as LONG;

    SetWindowPos(
        (*infoPtr).hwndHeader,
        0 as HWND,
        point[0 as usize].x as ::core::ffi::c_int,
        point[0 as usize].y as ::core::ffi::c_int,
        point[1 as usize].x as ::core::ffi::c_int,
        point[1 as usize].y as ::core::ffi::c_int,
        (if ((*infoPtr).dwStyle & (LVS_NOCOLUMNHEADER as DWORD)) != 0 {
            SWP_HIDEWINDOW
        } else {
            SWP_SHOWWINDOW | SWP_NOZORDER | SWP_NOACTIVATE
        }) as UINT,
    );
}

unsafe extern "C" fn LISTVIEW_UpdateHScroll(infoPtr: *mut LISTVIEW_INFO) -> INT {
    let mut horzInfo: SCROLLINFO = tagSCROLLINFO {
        cbSize: 0,
        fMask: 0,
        nMin: 0,
        nMax: 0,
        nPage: 0,
        nPos: 0,
        nTrackPos: 0,
    };
    let mut dx: INT = 0;

    ZeroMemory!(
        core::ptr::addr_of_mut!(horzInfo) as *mut core::ffi::c_void,
        core::mem::size_of::<SCROLLINFO>(),
    );
    horzInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    horzInfo.nPage = ((*infoPtr).rcList.right - (*infoPtr).rcList.left) as UINT;

    /* for now, we'll set info.nMax to the _count_, and adjust it later */
    if (*infoPtr).uView == LV_VIEW_LIST as DWORD {
        let nPerCol: INT = LISTVIEW_GetCountPerColumn(infoPtr);
        horzInfo.nMax = (((*infoPtr).nItemCount + nPerCol - 1) / nPerCol) as INT;

        /* scroll by at least one column per page */
        if horzInfo.nPage < (*infoPtr).nItemWidth as UINT {
            horzInfo.nPage = (*infoPtr).nItemWidth as UINT;
        }

        if (*infoPtr).nItemWidth != 0 {
            horzInfo.nPage /= (*infoPtr).nItemWidth as UINT;
        }
    } else if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
        horzInfo.nMax = (*infoPtr).nItemWidth;
    } else {
        /* LV_VIEW_ICON, or LV_VIEW_SMALLICON */
        let mut rcView: RECT = core::mem::zeroed();

        if LISTVIEW_GetViewRect(infoPtr, core::ptr::addr_of_mut!(rcView)) != 0 {
            horzInfo.nMax = rcView.right - rcView.left;
        }
    }

    if LISTVIEW_IsHeaderEnabled(infoPtr) != 0 {
        if DPA_GetPtrCount((*infoPtr).hdpaColumns) != 0 {
            let mut rcHeader: RECT = core::mem::zeroed();
            let mut index: INT = 0;

            index = SendMessageW(
                (*infoPtr).hwndHeader,
                HDM_ORDERTOINDEX as UINT,
                (DPA_GetPtrCount((*infoPtr).hdpaColumns) - 1) as WPARAM,
                0 as LPARAM,
            ) as INT;

            LISTVIEW_GetHeaderRect(infoPtr, index, core::ptr::addr_of_mut!(rcHeader));
            horzInfo.nMax = rcHeader.right;
        }
    }

    horzInfo.fMask = (SIF_RANGE | SIF_PAGE) as UINT;
    horzInfo.nMax = max(horzInfo.nMax - 1, 0);
    horzInfo.nMax = if horzInfo.nPage == 0 as UINT {
        0
    } else {
        horzInfo.nMax
    };
    dx = GetScrollPos((*infoPtr).hwndSelf, (SB_HORZ as UINT) as i32) as INT;
    dx -= SetScrollInfo(
        (*infoPtr).hwndSelf,
        SB_HORZ as INT,
        core::ptr::addr_of_mut!(horzInfo),
        TRUE,
    );

    /* Update the Header Control */
    if !(*infoPtr).hwndHeader.is_null() {
        horzInfo.fMask = SIF_POS as UINT;
        GetScrollInfo(
            (*infoPtr).hwndSelf,
            SB_HORZ as INT,
            core::ptr::addr_of_mut!(horzInfo),
        );
        LISTVIEW_UpdateHeaderSize(infoPtr, horzInfo.nPos);
    }

    LISTVIEW_UpdateSize(infoPtr);
    return dx;
}

unsafe extern "C" fn LISTVIEW_UpdateVScroll(infoPtr: *mut LISTVIEW_INFO) -> INT {
    let mut vertInfo: SCROLLINFO = core::mem::zeroed();
    let mut dy: INT = 0;

    ZeroMemory!(core::ptr::addr_of_mut!(vertInfo) as *mut core::ffi::c_void, core::mem::size_of::<SCROLLINFO>() as usize);
    vertInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    vertInfo.nPage = max(0, (*infoPtr).rcList.bottom - (*infoPtr).rcList.top) as UINT;

    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
        if vertInfo.nPage != 0 as UINT {
            vertInfo.nMax = (*infoPtr).nItemCount;

            /* scroll by at least one page */
            if vertInfo.nPage < (*infoPtr).nItemHeight as UINT {
                vertInfo.nPage = (*infoPtr).nItemHeight as UINT;
            }

            if (*infoPtr).nItemHeight > 0 {
                vertInfo.nPage /= (*infoPtr).nItemHeight as UINT;
            }
        }
    } else if (*infoPtr).uView != LV_VIEW_LIST as DWORD {
        /* LV_VIEW_ICON, or LV_VIEW_SMALLICON */
        let mut rcView: RECT = core::mem::zeroed();

        if LISTVIEW_GetViewRect(infoPtr, core::ptr::addr_of_mut!(rcView)) != 0 {
            vertInfo.nMax = rcView.bottom - rcView.top;
        }
    }

    vertInfo.fMask = (SIF_RANGE | SIF_PAGE) as UINT;
    vertInfo.nMax = max(vertInfo.nMax - 1, 0);
    vertInfo.nMax = if vertInfo.nPage == 0 as UINT {
        0
    } else {
        vertInfo.nMax
    };
    dy = GetScrollPos((*infoPtr).hwndSelf, (SB_VERT) as i32) as INT;
    dy -= SetScrollInfo(
        (*infoPtr).hwndSelf,
        (SB_VERT) as i32,
        core::ptr::addr_of_mut!(vertInfo) as LPCSCROLLINFO,
        TRUE,
    );

    LISTVIEW_UpdateSize(infoPtr);
    return dy;
}

unsafe extern "C" fn LISTVIEW_UpdateScroll(infoPtr: *mut LISTVIEW_INFO) {
    let mut dx: INT;
    let mut dy: INT;
    let mut pass: INT;

    if ((*infoPtr).dwStyle & LVS_NOSCROLL as DWORD) != 0 || is_redrawing(infoPtr) == 0 {
        return;
    }

    /* Setting the horizontal scroll can change the listview size
     * (and potentially everything else) so we need to recompute
     * everything again for the vertical scroll and vice-versa
     */
    dx = 0 as INT;
    dy = 0 as INT;
    pass = 0 as INT;
    while pass <= 1 as INT {
        dx += LISTVIEW_UpdateHScroll(infoPtr);
        dy += LISTVIEW_UpdateVScroll(infoPtr);
        pass += 1;
    }

    /* Change of the range may have changed the scroll pos. If so move the content */
    if dx != 0 as INT || dy != 0 as INT {
        let mut listRect: RECT = tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        listRect = (*infoPtr).rcList;
        ScrollWindowEx(
            (*infoPtr).hwndSelf,
            dx as ::core::ffi::c_int,
            dy as ::core::ffi::c_int,
            &raw mut listRect as LPCRECT,
            &raw mut listRect as LPCRECT,
            0 as HRGN,
            0 as *mut RECT,
            (SW_ERASE | SW_INVALIDATE) as UINT,
        );
    }
}

unsafe extern "C" fn LISTVIEW_ShowFocusRect(infoPtr: *const LISTVIEW_INFO, fShow: BOOL) {
    let mut hdc: HDC = core::ptr::null_mut();

    if (*infoPtr).nFocusedItem < 0 {
        return;
    }

    /* we need some gymnastics in ICON mode to handle large items */
    if (*infoPtr).uView == LV_VIEW_ICON {
        let mut rcBox: RECT = core::mem::zeroed();

        LISTVIEW_GetItemBox(infoPtr, (*infoPtr).nFocusedItem, core::ptr::addr_of_mut!(rcBox));
        if (rcBox.bottom - rcBox.top) > (*infoPtr).nItemHeight {
            LISTVIEW_InvalidateRect(infoPtr, core::ptr::addr_of_mut!(rcBox));
            return;
        }
    }

    hdc = GetDC((*infoPtr).hwndSelf);
    if hdc.is_null() {
        return;
    }

    /* for some reason, owner draw should work only in report mode */
    'done: {
        if ((*infoPtr).dwStyle & LVS_OWNERDRAWFIXED) != 0 && (*infoPtr).uView == LV_VIEW_DETAILS {
            let mut dis: DRAWITEMSTRUCT = core::mem::zeroed();
            let mut item: LVITEMW = core::mem::zeroed();

            let hFont: HFONT = if !(*infoPtr).hFont.is_null() {
                (*infoPtr).hFont
            } else {
                (*infoPtr).hDefaultFont
            };
            let hOldFont: HFONT = SelectObject(hdc, hFont as HGDIOBJ) as HFONT;

            item.iItem = (*infoPtr).nFocusedItem;
            item.iSubItem = 0;
            item.mask = LVIF_PARAM;
            if LISTVIEW_GetItemW(infoPtr, core::ptr::addr_of_mut!(item)) == 0 {
                break 'done;
            }

            core::ptr::write_bytes(core::ptr::addr_of_mut!(dis), 0, 1); // ZeroMemory(&dis, sizeof(dis)): ONE element
            dis.CtlType = ODT_LISTVIEW;
            dis.CtlID = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID) as UINT;
            dis.itemID = item.iItem as UINT;
            dis.itemAction = ODA_FOCUS;
            if fShow != 0 {
                dis.itemState |= ODS_FOCUS;
            }
            dis.hwndItem = (*infoPtr).hwndSelf;
            dis.hDC = hdc;
            LISTVIEW_GetItemBox(infoPtr, dis.itemID as INT, core::ptr::addr_of_mut!(dis.rcItem));
            dis.itemData = item.lParam as ULONG_PTR;

            SendMessageW(
                (*infoPtr).hwndNotify,
                WM_DRAWITEM,
                dis.CtlID as WPARAM,
                core::ptr::addr_of_mut!(dis) as LPARAM,
            );

            SelectObject(hdc, hOldFont as HGDIOBJ);
        } else {
            LISTVIEW_InvalidateItem(infoPtr, (*infoPtr).nFocusedItem);
        }
    }

    ReleaseDC((*infoPtr).hwndSelf, hdc);
}

unsafe extern "C" fn LISTVIEW_InvalidateSelectedItems(infoPtr: *const LISTVIEW_INFO) {
    let mut i: ITERATOR = core::mem::zeroed();
    iterator_frameditems(core::ptr::addr_of_mut!(i), infoPtr, core::ptr::addr_of!((*infoPtr).rcList));
    while iterator_next(core::ptr::addr_of_mut!(i)) != 0 {
        if LISTVIEW_GetItemState(infoPtr, i.nItem, LVIS_SELECTED as UINT) != 0 {
            LISTVIEW_InvalidateItem(infoPtr, i.nItem);
        }
    }
    iterator_destroy(core::ptr::addr_of_mut!(i));
}

unsafe extern "C" fn LISTVIEW_GetItemOrigin(infoPtr: *const LISTVIEW_INFO, nItem: INT, lpptPosition: LPPOINT) {
    assert!(nItem >= 0 && nItem < (*infoPtr).nItemCount);

    if ((*infoPtr).uView == LV_VIEW_SMALLICON as DWORD) || ((*infoPtr).uView == LV_VIEW_ICON as DWORD)
    {
        (*lpptPosition).x = DPA_GetPtr((*infoPtr).hdpaPosX, nItem) as LONG_PTR as LONG;
        (*lpptPosition).y = DPA_GetPtr((*infoPtr).hdpaPosY, nItem) as LONG_PTR as LONG;
    }
    else if (*infoPtr).uView == LV_VIEW_LIST as DWORD
    {
        let mut nCountPerColumn: INT = LISTVIEW_GetCountPerColumn(infoPtr);
        (*lpptPosition).x = ((nItem / nCountPerColumn) * (*infoPtr).nItemWidth) as LONG;
        (*lpptPosition).y = ((nItem % nCountPerColumn) * (*infoPtr).nItemHeight) as LONG;
    }
    else
    {
        (*lpptPosition).x = (REPORT_MARGINX) as LONG;
        /* item is always at zero indexed column */
        if DPA_GetPtrCount((*infoPtr).hdpaColumns) > 0
        {
            (*lpptPosition).x += (*LISTVIEW_GetColumnInfo(infoPtr, 0)).rcHeader.left;
        }
        (*lpptPosition).y = (nItem * (*infoPtr).nItemHeight) as LONG;
    }
}

unsafe extern "C" fn LISTVIEW_GetItemMetrics(
    infoPtr: *const LISTVIEW_INFO,
    lpLVItem: *const LVITEMW,
    lprcBox: LPRECT,
    lprcSelectBox: LPRECT,
    lprcIcon: LPRECT,
    lprcStateIcon: LPRECT,
    lprcLabel: LPRECT,
) {
    let mut doSelectBox: BOOL = FALSE;
    let mut doIcon: BOOL = FALSE;
    let mut doLabel: BOOL = FALSE;
    let mut oversizedBox: BOOL = FALSE;
    let mut Box: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut SelectBox: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut Icon: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut Label: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut lpColumnInfo: *mut COLUMN_INFO = core::ptr::null_mut();
    let mut labelSize: SIZE = SIZE {
        cx: 0 as LONG,
        cy: 0 as LONG,
    };

    /* Be smart and try to figure out the minimum we have to do */
    if (*lpLVItem).iSubItem != 0 {
        assert!(((*infoPtr).uView == LV_VIEW_DETAILS) as INT);
    }
    if (*infoPtr).uView == LV_VIEW_ICON && (!lprcBox.is_null() || !lprcLabel.is_null()) {
        assert!((((*lpLVItem).mask & LVIF_STATE) != 0 && ((*lpLVItem).stateMask & LVIS_FOCUSED) != 0) as INT);
        if ((*lpLVItem).state & LVIS_FOCUSED) != 0 {
            doLabel = TRUE;
            oversizedBox = doLabel;
        }
    }
    if !lprcSelectBox.is_null() {
        doSelectBox = TRUE;
    }
    if !lprcLabel.is_null() {
        doLabel = TRUE;
    }
    if doLabel != 0 || !lprcIcon.is_null() || !lprcStateIcon.is_null() {
        doIcon = TRUE;
    }
    if doSelectBox != 0 {
        doIcon = TRUE;
        doLabel = TRUE;
    }

    /************************************************************/
    /* compute the box rectangle (it should be cheap to do)     */
    /************************************************************/
    if (*lpLVItem).iSubItem != 0 || (*infoPtr).uView == LV_VIEW_DETAILS {
        lpColumnInfo = LISTVIEW_GetColumnInfo(infoPtr, (*lpLVItem).iSubItem as INT);
    }

    if (*lpLVItem).iSubItem != 0 {
        Box = (*lpColumnInfo).rcHeader;
    } else {
        Box.left = 0 as LONG;
        Box.right = (*infoPtr).nItemWidth as LONG;
    }
    Box.top = 0 as LONG;
    Box.bottom = (*infoPtr).nItemHeight as LONG;

    /******************************************************************/
    /* compute ICON bounding box (ala LVM_GETITEMRECT) and STATEICON  */
    /******************************************************************/
    if doIcon != 0 {
        let mut state_width: LONG = 0 as LONG;

        if !(*infoPtr).himlState.is_null() && (*lpLVItem).iSubItem == 0 {
            state_width = (*infoPtr).iconStateSize.cx;
        }

        if (*infoPtr).uView == LV_VIEW_ICON {
            Icon.left = Box.left + state_width;
            if !(*infoPtr).himlNormal.is_null() {
                Icon.left += ((*infoPtr).nItemWidth - (*infoPtr).iconSize.cx - state_width) / 2;
            }
            Icon.top = Box.top + ICON_TOP_PADDING as LONG;
            Icon.right = Icon.left;
            Icon.bottom = Icon.top;
            if !(*infoPtr).himlNormal.is_null() {
                Icon.right += (*infoPtr).iconSize.cx;
                Icon.bottom += (*infoPtr).iconSize.cy;
            }
        } else {
            /* LV_VIEW_SMALLICON, LV_VIEW_LIST or LV_VIEW_DETAILS */
            Icon.left = Box.left + state_width;

            if (*infoPtr).uView == LV_VIEW_DETAILS && (*lpLVItem).iSubItem == 0 {
                /* we need the indent in report mode */
                assert!(((*lpLVItem).mask & LVIF_INDENT != 0) as INT);
                Icon.left += (*infoPtr).iconSize.cx * (*lpLVItem).iIndent + REPORT_MARGINX;
            }

            Icon.top = Box.top;
            Icon.right = Icon.left;
            if !(*infoPtr).himlSmall.is_null()
                && (lpColumnInfo.is_null()
                    || (*lpLVItem).iSubItem == 0
                    || (((*infoPtr).dwLvExStyle & LVS_EX_SUBITEMIMAGES) != 0
                        && (*lpLVItem).iImage != I_IMAGECALLBACK))
            {
                Icon.right += (*infoPtr).iconSize.cx;
            }
            Icon.bottom = Icon.top + (*infoPtr).iconSize.cy;
        }
        if !lprcIcon.is_null() {
            *lprcIcon = Icon;
        }

        /* TODO: is this correct? */
        if !lprcStateIcon.is_null() {
            (*lprcStateIcon).left = Icon.left - state_width;
            (*lprcStateIcon).right = Icon.left;
            (*lprcStateIcon).top = Icon.top;
            (*lprcStateIcon).bottom = (*lprcStateIcon).top + (*infoPtr).iconSize.cy;
        }
    } else {
        Icon.right = 0 as LONG;
    }

    /************************************************************/
    /* compute LABEL bounding box (ala LVM_GETITEMRECT)         */
    /************************************************************/
    if doLabel != 0 {
        /* calculate how far to the right can the label stretch */
        Label.right = Box.right;
        if (*infoPtr).uView == LV_VIEW_DETAILS {
            if (*lpLVItem).iSubItem == 0 {
                /* we need a zero based rect here */
                Label = (*lpColumnInfo).rcHeader;
                OffsetRect(&raw mut Label, -Label.left, 0);
            }
        }

        'calc_label: {
            if (*lpLVItem).iSubItem != 0
                || ((*infoPtr).dwStyle & LVS_OWNERDRAWFIXED) != 0 && (*infoPtr).uView == LV_VIEW_DETAILS
            {
                labelSize.cx = (*infoPtr).nItemWidth;
                labelSize.cy = (*infoPtr).nItemHeight;
                break 'calc_label;
            }

            /* we need the text in non owner draw mode */
            assert!(((*lpLVItem).mask & LVIF_TEXT != 0) as INT);
            if is_text((*lpLVItem).pszText as LPCWSTR) != 0 {
                let mut hFont: HFONT = if !(*infoPtr).hFont.is_null() {
                    (*infoPtr).hFont
                } else {
                    (*infoPtr).hDefaultFont
                };
                let mut hdc: HDC = GetDC((*infoPtr).hwndSelf);
                let mut hOldFont: HFONT = SelectObject(hdc, hFont as HGDIOBJ) as HFONT;
                let mut uFormat: UINT = 0;
                let mut rcText: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

                /* compute rough rectangle where the label will go */
                SetRectEmpty(&raw mut rcText);
                rcText.right = (*infoPtr).nItemWidth - TRAILING_LABEL_PADDING as LONG;
                rcText.bottom = (*infoPtr).nItemHeight;
                if (*infoPtr).uView == LV_VIEW_ICON {
                    rcText.bottom -= ICON_TOP_PADDING + (*infoPtr).iconSize.cy + ICON_BOTTOM_PADDING;
                }

                /* now figure out the flags */
                if (*infoPtr).uView == LV_VIEW_ICON {
                    uFormat = if oversizedBox != 0 {
                        LV_FL_DT_FLAGS
                    } else {
                        LV_ML_DT_FLAGS
                    };
                } else {
                    uFormat = LV_SL_DT_FLAGS;
                }

                DrawTextW(
                    hdc,
                    (*lpLVItem).pszText as LPCWSTR,
                    -1,
                    &raw mut rcText,
                    uFormat | DT_CALCRECT,
                );

                if rcText.right != rcText.left {
                    labelSize.cx = min(rcText.right - rcText.left + TRAILING_LABEL_PADDING, (*infoPtr).nItemWidth) as LONG;
                }

                labelSize.cy = rcText.bottom - rcText.top;

                SelectObject(hdc, hOldFont as HGDIOBJ);
                ReleaseDC((*infoPtr).hwndSelf, hdc);
            }
        }

        /* calc_label: */
        if (*infoPtr).uView == LV_VIEW_ICON {
            Label.left = Box.left + ((*infoPtr).nItemWidth - labelSize.cx) / 2;
            Label.top = Box.top + ICON_TOP_PADDING_HITABLE + (*infoPtr).iconSize.cy + ICON_BOTTOM_PADDING;
            Label.right = Label.left + labelSize.cx;
            Label.bottom = Label.top + (*infoPtr).nItemHeight;
            if oversizedBox == 0 && labelSize.cy > (*infoPtr).ntmHeight {
                labelSize.cy = min(Box.bottom - Label.top, labelSize.cy);
                labelSize.cy /= (*infoPtr).ntmHeight;
                labelSize.cy = max(labelSize.cy, 1);
                labelSize.cy *= (*infoPtr).ntmHeight;
            }
            Label.bottom = Label.top + labelSize.cy + HEIGHT_PADDING;
        } else if (*infoPtr).uView == LV_VIEW_DETAILS {
            Label.left = Icon.right;
            Label.top = Box.top;
            Label.right = if (*lpLVItem).iSubItem != 0 {
                (*lpColumnInfo).rcHeader.right
            } else {
                (*lpColumnInfo).rcHeader.right - (*lpColumnInfo).rcHeader.left
            };
            Label.bottom = Label.top + (*infoPtr).nItemHeight;
        } else {
            /* LV_VIEW_SMALLICON or LV_VIEW_LIST */
            Label.left = Icon.right;
            Label.top = Box.top;
            Label.right = min(Label.left + labelSize.cx, Label.right);
            Label.bottom = Label.top + (*infoPtr).nItemHeight;
        }

        if !lprcLabel.is_null() {
            *lprcLabel = Label;
        }
    }

    /************************************************************/
    /* compute SELECT bounding box                              */
    /************************************************************/
    if doSelectBox != 0 {
        if (*infoPtr).uView == LV_VIEW_DETAILS {
            SelectBox.left = Icon.left;
            SelectBox.top = Box.top;
            SelectBox.bottom = Box.bottom;

            if labelSize.cx != 0 {
                SelectBox.right = min(Label.left + labelSize.cx, Label.right);
            } else {
                SelectBox.right = min(Label.left + MAX_EMPTYTEXT_SELECT_WIDTH, Label.right);
            }
        } else {
            UnionRect(&raw mut SelectBox, &raw mut Icon as LPCRECT, &raw mut Label as LPCRECT);
        }
        if !lprcSelectBox.is_null() {
            *lprcSelectBox = SelectBox;
        }
    }

    /* Fix the Box if necessary */
    if !lprcBox.is_null() {
        if oversizedBox != 0 {
            UnionRect(lprcBox, &raw mut Box as LPCRECT, &raw mut Label as LPCRECT);
        } else {
            *lprcBox = Box;
        }
    }
}

unsafe extern "C" fn LISTVIEW_GetItemBox(infoPtr: *const LISTVIEW_INFO, nItem: INT, lprcBox: LPRECT) {
    let mut szDispText: [WCHAR; (DISP_TEXT_SIZE) as usize] = [0 as WCHAR; (DISP_TEXT_SIZE) as usize];
    let mut Position: POINT = POINT { x: 0, y: 0 };
    let mut Origin: POINT = POINT { x: 0, y: 0 };
    let mut lvItem: LVITEMW = core::mem::zeroed();

    LISTVIEW_GetOrigin(infoPtr, core::ptr::addr_of_mut!(Origin));
    LISTVIEW_GetItemOrigin(infoPtr, nItem, core::ptr::addr_of_mut!(Position));

    /* Be smart and try to figure out the minimum we have to do */
    lvItem.mask = 0 as UINT;
    if (*infoPtr).uView == LV_VIEW_ICON as DWORD
        && (*infoPtr).bFocus != 0
        && LISTVIEW_GetItemState(infoPtr, nItem, LVIS_FOCUSED as UINT) != 0
    {
        lvItem.mask |= LVIF_TEXT as UINT;
    }
    lvItem.iItem = nItem;
    lvItem.iSubItem = 0 as INT;
    lvItem.pszText = szDispText.as_mut_ptr();
    lvItem.cchTextMax = DISP_TEXT_SIZE as INT;
    if lvItem.mask != 0 {
        LISTVIEW_GetItemW(infoPtr, core::ptr::addr_of_mut!(lvItem));
    }
    if (*infoPtr).uView == LV_VIEW_ICON as DWORD {
        lvItem.mask |= LVIF_STATE as UINT;
        lvItem.stateMask = LVIS_FOCUSED as UINT;
        lvItem.state = (if lvItem.mask & LVIF_TEXT as UINT != 0 {
            LVIS_FOCUSED
        } else {
            0
        }) as UINT;
    }
    LISTVIEW_GetItemMetrics(
        infoPtr,
        core::ptr::addr_of_mut!(lvItem),
        lprcBox,
        0 as LPRECT,
        0 as LPRECT,
        0 as LPRECT,
        0 as LPRECT,
    );

    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD
        && (*infoPtr).dwLvExStyle & LVS_EX_FULLROWSELECT as DWORD != 0
        && SendMessageW(
            (*infoPtr).hwndHeader,
            HDM_ORDERTOINDEX as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) != 0
    {
        OffsetRect(lprcBox, Origin.x, Position.y + Origin.y);
    } else {
        OffsetRect(lprcBox, Position.x + Origin.x, Position.y + Origin.y);
    }
}
unsafe extern "system" fn MapIdSearchCompare(p1: LPVOID, p2: LPVOID, lParam: LPARAM) -> INT {
    let id1: *mut ITEM_ID = p1 as *mut ITEM_ID;
    let id2: *mut ITEM_ID = p2 as *mut ITEM_ID;

    if (*id1).id == (*id2).id {
        return 0;
    }

    return if (*id1).id < (*id2).id {
        -1
    } else {
        1
    };
}

unsafe extern "C" fn LISTVIEW_MapIdToIndex(infoPtr: *const LISTVIEW_INFO, iID: UINT) -> INT {
    let mut ID: ITEM_ID = core::mem::zeroed();
    let mut index: INT = 0;

    if ((*infoPtr).dwStyle & LVS_OWNERDATA) != 0 {
        return -1;
    }
    if (*infoPtr).nItemCount == 0 {
        return -1;
    }

    ID.id = iID;
    index = DPA_Search(
        (*infoPtr).hdpaItemIds,
        &mut ID as *mut ITEM_ID as *mut core::ffi::c_void,
        -1,
        Some(MapIdSearchCompare),
        0 as LPARAM,
        DPAS_SORTED as UINT,
    ) as INT;

    if index != -1 {
        let lpID: *mut ITEM_ID = DPA_GetPtr((*infoPtr).hdpaItemIds, index) as *mut ITEM_ID;
        return DPA_GetPtrIndex(
            (*infoPtr).hdpaItems,
            (*lpID).item as *const core::ffi::c_void,
        ) as INT;
    }

    return -1;
}

unsafe extern "C" fn LISTVIEW_MapIndexToId(infoPtr: *const LISTVIEW_INFO, iItem: INT) -> DWORD {
    let mut lpItem: *mut ITEM_INFO = core::ptr::null_mut();
    let mut hdpaSubItems: HDPA = core::ptr::null_mut();

    if (*infoPtr).dwStyle & LVS_OWNERDATA != 0 { return (-(1 as i32)) as DWORD; }
    if iItem < 0 || iItem >= (*infoPtr).nItemCount { return (-(1 as i32)) as DWORD; }

    hdpaSubItems = DPA_GetPtr((*infoPtr).hdpaItems, iItem) as HDPA;
    lpItem = DPA_GetPtr(hdpaSubItems, 0 as INT) as *mut ITEM_INFO;

    return (*(*lpItem).id).id as DWORD;
}

unsafe extern "C" fn LISTVIEW_NextIconPosTop(infoPtr: *mut LISTVIEW_INFO, lpPos: LPPOINT, nItem: INT) {
    let mut nListWidth: INT = (*infoPtr).rcList.right - (*infoPtr).rcList.left;

    *lpPos = (*infoPtr).currIconPos;

    (*infoPtr).currIconPos.x += (*infoPtr).nItemWidth;
    if (*infoPtr).currIconPos.x + (*infoPtr).nItemWidth <= nListWidth {
        return;
    }

    (*infoPtr).currIconPos.x = 0;
    (*infoPtr).currIconPos.y += (*infoPtr).nItemHeight;
}

unsafe extern "C" fn LISTVIEW_NextIconPosLeft(infoPtr: *mut LISTVIEW_INFO, lpPos: LPPOINT, nItem: INT) {
    let mut nListHeight: INT = (*infoPtr).rcList.bottom - (*infoPtr).rcList.top;

    *lpPos = (*infoPtr).currIconPos;

    (*infoPtr).currIconPos.y += (*infoPtr).nItemHeight;
    if (*infoPtr).currIconPos.y + (*infoPtr).nItemHeight <= nListHeight {
        return;
    }

    (*infoPtr).currIconPos.x += (*infoPtr).nItemWidth;
    (*infoPtr).currIconPos.y = 0;
}

unsafe extern "C" fn LISTVIEW_NextIconPosSnap(infoPtr: *mut LISTVIEW_INFO, lpPos: LPPOINT, nItem: INT) {
    let nListHeight: INT = (*infoPtr).rcList.bottom - (*infoPtr).rcList.top;
    let nListWidth: INT = (*infoPtr).rcList.right - (*infoPtr).rcList.left;
    let nMaxColumns: INT = nListWidth / (*infoPtr).nItemWidth;
    let nMaxRows: INT = nListHeight / (*infoPtr).nItemHeight;
    let mut oldPosition: POINT = POINT { x: 0, y: 0 };

    oldPosition.x = DPA_GetPtr((*infoPtr).hdpaPosX, nItem) as LONG_PTR as LONG;
    oldPosition.y = DPA_GetPtr((*infoPtr).hdpaPosY, nItem) as LONG_PTR as LONG;

    (*lpPos).x = max(
        0,
        oldPosition.x + ((*infoPtr).nItemWidth >> 1)
            - (oldPosition.x + ((*infoPtr).nItemWidth >> 1)) % (*infoPtr).nItemWidth,
    );
    (*lpPos).y = max(
        0,
        oldPosition.y + ((*infoPtr).nItemHeight >> 1)
            - (oldPosition.y + ((*infoPtr).nItemHeight >> 1)) % (*infoPtr).nItemHeight,
    );

    if (*lpPos).x > nListWidth {
        (*lpPos).x = nMaxColumns * (*infoPtr).nItemWidth;
    }
    if (*lpPos).y > nListHeight {
        (*lpPos).y = nMaxRows * (*infoPtr).nItemHeight;
    }
}

unsafe extern "C" fn LISTVIEW_MoveIconTo(infoPtr: *const LISTVIEW_INFO, nItem: INT, lppt: *const POINT, isNew: BOOL) -> BOOL {
    let mut old: POINT = POINT { x: 0, y: 0 };

    if isNew == 0 {
        old.x = DPA_GetPtr((*infoPtr).hdpaPosX, nItem) as LONG_PTR as LONG;
        old.y = DPA_GetPtr((*infoPtr).hdpaPosY, nItem) as LONG_PTR as LONG;

        if (*lppt).x == old.x && (*lppt).y == old.y {
            return TRUE;
        }
        LISTVIEW_InvalidateItem(infoPtr, nItem);
    }

    /* Allocating a POINTER for every item is too resource intensive,
     * so we'll keep the (x,y) in different arrays */
    if DPA_SetPtr((*infoPtr).hdpaPosX, nItem, (*lppt).x as LONG_PTR as *mut ::core::ffi::c_void) == 0 {
        return FALSE;
    }
    if DPA_SetPtr((*infoPtr).hdpaPosY, nItem, (*lppt).y as LONG_PTR as *mut ::core::ffi::c_void) == 0 {
        return FALSE;
    }

    LISTVIEW_InvalidateItem(infoPtr, nItem);

    return TRUE;
}

unsafe extern "C" fn LISTVIEW_Arrange(infoPtr: *mut LISTVIEW_INFO, mut nAlignCode: INT) -> BOOL {
    let mut next_pos: unsafe extern "C" fn(*mut LISTVIEW_INFO, LPPOINT, INT) = LISTVIEW_NextIconPosLeft;
    let mut pos: POINT = POINT { x: 0, y: 0 };
    let mut i: INT = 0;

    if (*infoPtr).uView != LV_VIEW_ICON as DWORD && (*infoPtr).uView != LV_VIEW_SMALLICON as DWORD {
        return FALSE;
    }

    if nAlignCode == (LVA_DEFAULT as i32) {
        if ((*infoPtr).dwStyle & (LVS_ALIGNLEFT as DWORD)) != 0 {
            nAlignCode = (LVA_ALIGNLEFT as i32);
        } else {
            nAlignCode = (LVA_ALIGNTOP as i32);
        }
    }

    match (nAlignCode) as u32 {
        LVA_ALIGNLEFT => {
            next_pos = LISTVIEW_NextIconPosLeft;
        }
        LVA_ALIGNTOP => {
            next_pos = LISTVIEW_NextIconPosTop;
        }
        LVA_SNAPTOGRID => {
            next_pos = LISTVIEW_NextIconPosSnap;
        }
        _ => return FALSE,
    }

    (*infoPtr).currIconPos.y = 0 as LONG;
    (*infoPtr).currIconPos.x = (*infoPtr).currIconPos.y;

    i = 0 as INT;
    while i < (*infoPtr).nItemCount {
        next_pos(infoPtr, &raw mut pos, i);
        LISTVIEW_MoveIconTo(infoPtr, i, &raw mut pos, FALSE);
        i += 1;
    }

    return TRUE;
}

unsafe extern "C" fn LISTVIEW_GetAreaRect(infoPtr: *const LISTVIEW_INFO, lprcView: LPRECT) {
    let mut i: INT = 0;
    let mut x: INT = 0;
    let mut y: INT = 0;

    SetRectEmpty(lprcView);

    match (*infoPtr).uView {
        LV_VIEW_ICON | LV_VIEW_SMALLICON => {
            i = 0;
            while i < (*infoPtr).nItemCount {
                x = DPA_GetPtr((*infoPtr).hdpaPosX, i) as LONG_PTR as INT;
                y = DPA_GetPtr((*infoPtr).hdpaPosY, i) as LONG_PTR as INT;
                (*lprcView).right = max((*lprcView).right, x);
                (*lprcView).bottom = max((*lprcView).bottom, y);
                i += 1;
            }
            if (*infoPtr).nItemCount > 0 {
                (*lprcView).right += (*infoPtr).nItemWidth;
                (*lprcView).bottom += (*infoPtr).nItemHeight;
            }
        }

        LV_VIEW_LIST => {
            y = LISTVIEW_GetCountPerColumn(infoPtr);
            x = (*infoPtr).nItemCount / y;
            if (*infoPtr).nItemCount % y != 0 {
                x += 1;
            }
            (*lprcView).right = (x * (*infoPtr).nItemWidth) as LONG;
            (*lprcView).bottom = (y * (*infoPtr).nItemHeight) as LONG;
        }

        _ => {}
    }
}

unsafe extern "C" fn LISTVIEW_GetViewRect(infoPtr: *const LISTVIEW_INFO, lprcView: LPRECT) -> BOOL {
    let mut ptOrigin: POINT = POINT { x: 0, y: 0 };
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_GetViewRect\0" as *const u8 as *const ::core::ffi::c_char,
            2978 as ::core::ffi::c_int,
            b"(lprcView=%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            lprcView,
        );
    }
    if lprcView.is_null() {
        return FALSE;
    }
    LISTVIEW_GetAreaRect(infoPtr, lprcView);
    if (*infoPtr).uView != LV_VIEW_DETAILS {
        LISTVIEW_GetOrigin(infoPtr, &raw mut ptOrigin);
        OffsetRect(
            lprcView,
            ptOrigin.x as ::core::ffi::c_int,
            ptOrigin.y as ::core::ffi::c_int,
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
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_GetViewRect\0" as *const u8 as *const ::core::ffi::c_char,
            2990 as ::core::ffi::c_int,
            b"lprcView=%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_rect(lprcView as *const RECT),
        );
    }
    return TRUE;
}

unsafe extern "C" fn LISTVIEW_GetSubItemPtr(hdpaSubItems: HDPA, nSubItem: INT) -> *mut SUBITEM_INFO {
    let mut lpSubItem: *mut SUBITEM_INFO = core::ptr::null_mut();
    let mut i: INT;

    /* we should binary search here if need be */
    i = 1;
    while i < DPA_GetPtrCount(hdpaSubItems) {
        lpSubItem = DPA_GetPtr(hdpaSubItems, i) as *mut SUBITEM_INFO;
        if (*lpSubItem).iSubItem == nSubItem {
            return lpSubItem;
        }
        i += 1;
    }

    return core::ptr::null_mut();
}

unsafe extern "C" fn LISTVIEW_CalculateItemWidth(infoPtr: *const LISTVIEW_INFO) -> INT {
    let mut nItemWidth: INT = 0 as INT;

    if (*infoPtr).uView == LV_VIEW_ICON as DWORD {
        nItemWidth = (*infoPtr).iconSpacing.cx as INT;
    } else if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
        if DPA_GetPtrCount((*infoPtr).hdpaColumns) > 0 {
            let mut rcHeader: RECT = core::mem::zeroed::<RECT>();
            let mut index: INT = 0;

            index = SendMessageW(
                (*infoPtr).hwndHeader,
                HDM_ORDERTOINDEX as UINT,
                (DPA_GetPtrCount((*infoPtr).hdpaColumns) - 1) as WPARAM,
                0 as LPARAM,
            ) as INT;

            LISTVIEW_GetHeaderRect(infoPtr, index, core::ptr::addr_of_mut!(rcHeader));
            nItemWidth = rcHeader.right as INT;
        }
    } else {
        let mut szDispText: [WCHAR; (DISP_TEXT_SIZE) as usize] = [0 as WCHAR; (DISP_TEXT_SIZE) as usize];
        let mut lvItem: LVITEMW = core::mem::zeroed::<LVITEMW>();
        let mut i: INT = 0;

        lvItem.mask = LVIF_TEXT as UINT;
        lvItem.iSubItem = 0 as INT;

        i = 0 as INT;
        while i < (*infoPtr).nItemCount {
            lvItem.iItem = i;
            lvItem.pszText = core::ptr::addr_of_mut!(szDispText) as *mut WCHAR as LPWSTR;
            lvItem.cchTextMax = DISP_TEXT_SIZE as INT;
            if LISTVIEW_GetItemW(infoPtr, core::ptr::addr_of_mut!(lvItem)) != 0 {
                nItemWidth = max(
                    LISTVIEW_GetStringWidthT(infoPtr, lvItem.pszText as LPCWSTR, TRUE),
                    nItemWidth,
                );
            }
            i += 1;
        }

        if !(*infoPtr).himlSmall.is_null() {
            nItemWidth += (*infoPtr).iconSize.cx as INT;
        }
        if !(*infoPtr).himlState.is_null() {
            nItemWidth += (*infoPtr).iconStateSize.cx as INT;
        }

        nItemWidth = max(DEFAULT_COLUMN_WIDTH, nItemWidth + WIDTH_PADDING);
    }

    return nItemWidth;
}

unsafe extern "C" fn LISTVIEW_CalculateItemHeight(infoPtr: *const LISTVIEW_INFO) -> INT {
    let mut nItemHeight: INT;
    // TRACE("uView=%d\n", infoPtr->uView);

    if (*infoPtr).uView == LV_VIEW_ICON {
        nItemHeight = (*infoPtr).iconSpacing.cy;
    } else {
        nItemHeight = (*infoPtr).ntmHeight;
        if !(*infoPtr).himlState.is_null() {
            nItemHeight = max(nItemHeight, (*infoPtr).iconStateSize.cy);
        }
        if !(*infoPtr).himlSmall.is_null() {
            nItemHeight = max(nItemHeight, (*infoPtr).iconSize.cy);
        }
        nItemHeight += HEIGHT_PADDING;
        if (*infoPtr).nMeasureItemHeight > 0 {
            nItemHeight = (*infoPtr).nMeasureItemHeight;
        }
    }

    return max(nItemHeight, 1);
}

unsafe extern "C" fn LISTVIEW_UpdateItemSize(mut infoPtr: *mut LISTVIEW_INFO) {
    (*infoPtr).nItemWidth = LISTVIEW_CalculateItemWidth(infoPtr);
    (*infoPtr).nItemHeight = LISTVIEW_CalculateItemHeight(infoPtr);
}

unsafe extern "C" fn LISTVIEW_SaveTextMetrics(infoPtr: *mut LISTVIEW_INFO) {
    let hdc: HDC = GetDC((*infoPtr).hwndSelf);
    let hFont: HFONT = if !(*infoPtr).hFont.is_null() {
        (*infoPtr).hFont
    } else {
        (*infoPtr).hDefaultFont
    };
    let hOldFont: HFONT = SelectObject(hdc, hFont as HGDIOBJ) as HFONT;
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let mut sz: SIZE = core::mem::zeroed();

    if GetTextMetricsW(hdc, &mut tm) != 0 {
        (*infoPtr).ntmHeight = tm.tmHeight as INT;
        (*infoPtr).ntmMaxCharWidth = tm.tmMaxCharWidth as INT;
    }

    if GetTextExtentPoint32A(hdc, b"...\0".as_ptr() as *const i8, 3, &mut sz) != 0 {
        (*infoPtr).nEllipsisWidth = sz.cx as INT;
    }

    SelectObject(hdc, hOldFont as HGDIOBJ);
    ReleaseDC((*infoPtr).hwndSelf, hdc);
}

unsafe extern "system" fn ranges_cmp(range1: LPVOID, range2: LPVOID, flags: LPARAM) -> INT {
    let mut cmp: INT = 0;
    if (*(range1 as *mut RANGE)).upper <= (*(range2 as *mut RANGE)).lower {
        cmp = -(1) as INT;
    } else if (*(range2 as *mut RANGE)).upper <= (*(range1 as *mut RANGE)).lower {
        cmp = 1 as INT;
    } else {
        cmp = 0 as INT;
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
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ranges_cmp\0" as *const u8 as *const ::core::ffi::c_char,
            3191 as ::core::ffi::c_int,
            b"range1=%s, range2=%s, cmp=%d\n\0" as *const u8
                as *const ::core::ffi::c_char,
            debugrange(range1 as *const RANGE),
            debugrange(range2 as *const RANGE),
            cmp,
        );
    }
    return cmp;
}

unsafe extern "C" fn ranges_assert(ranges: RANGES, desc: LPCSTR, file: *const core::ffi::c_char, line: core::ffi::c_int) {
    let mut i: INT;
    let mut prev: *mut RANGE;
    let mut curr: *mut RANGE;
    assert!(((!ranges.is_null()) as core::ffi::c_int));
    assert!((DPA_GetPtrCount((*ranges).hdpa) >= 0) as core::ffi::c_int);
    ranges_dump(ranges);
    if DPA_GetPtrCount((*ranges).hdpa) > 0 {
        prev = DPA_GetPtr((*ranges).hdpa, 0) as *mut RANGE;
        assert!((((*prev).lower >= 0 && (*prev).lower < (*prev).upper) as core::ffi::c_int));
        i = 1;
        while i < DPA_GetPtrCount((*ranges).hdpa) {
            curr = DPA_GetPtr((*ranges).hdpa, i) as *mut RANGE;
            assert!(((*prev).upper <= (*curr).lower) as core::ffi::c_int);
            assert!(((*curr).lower < (*curr).upper) as core::ffi::c_int);
            prev = curr;
            i += 1;
        }
    }
}

unsafe extern "C" fn ranges_create(count: i32) -> RANGES {
    let ranges: RANGES = Alloc(core::mem::size_of::<tagRANGES>() as DWORD) as RANGES;
    if ranges.is_null() {
        return core::ptr::null_mut();
    }
    (*ranges).hdpa = DPA_Create(count);
    if !(*ranges).hdpa.is_null() {
        return ranges;
    }
    Free(ranges as LPVOID);
    core::ptr::null_mut()
}

unsafe extern "C" fn ranges_clear(ranges: RANGES) {
    let mut i: INT = 0;
    while i < DPA_GetPtrCount((*ranges).hdpa) {
        Free(DPA_GetPtr((*ranges).hdpa, i));
        i += 1;
    }
    DPA_DeleteAllPtrs((*ranges).hdpa);
}

unsafe extern "C" fn ranges_destroy(ranges: RANGES) {
    if ranges.is_null() { return; }
    ranges_clear(ranges);
    DPA_Destroy((*ranges).hdpa);
    Free(ranges as LPVOID);
}

unsafe extern "C" fn ranges_clone(ranges: RANGES) -> RANGES {
    let mut clone: RANGES = ::core::ptr::null_mut::<tagRANGES>();
    let mut i: INT;

    if ranges.is_null() || (*ranges).hdpa.is_null() {
        return ::core::ptr::null_mut::<tagRANGES>();
    }
    'fail: {
        clone = ranges_create(DPA_GetPtrCount((*ranges).hdpa));
        if clone.is_null() {
            break 'fail;
        }
        i = 0;
        while i < DPA_GetPtrCount((*ranges).hdpa) {
            let newrng: *mut RANGE =
                Alloc(::core::mem::size_of::<RANGE>() as DWORD) as *mut RANGE;
            if newrng.is_null() {
                break 'fail;
            }
            *newrng = core::ptr::read(DPA_GetPtr((*ranges).hdpa, i) as *mut RANGE);
            if DPA_SetPtr(
                (*clone).hdpa,
                i,
                newrng as *mut ::core::ffi::c_void,
            ) == 0
            {
                Free(newrng as LPVOID);
                break 'fail;
            }
            i += 1;
        }
        return clone;
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
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ranges_clone\0" as *const u8 as *const ::core::ffi::c_char,
            3282 as ::core::ffi::c_int,
            b"clone failed\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    ranges_destroy(clone);
    return ::core::ptr::null_mut::<tagRANGES>();
}

unsafe extern "C" fn ranges_diff(mut ranges: RANGES, mut sub: RANGES) -> RANGES {
    let mut i: INT = 0;
    while i < DPA_GetPtrCount((*sub).hdpa) {
        ranges_del(ranges, core::ptr::read((DPA_GetPtr((*sub).hdpa, i)) as *mut RANGE));
        i += 1;
    }
    return ranges;
}

unsafe extern "C" fn ranges_dump(mut ranges: RANGES) {
    let mut i: INT = 0;
    i = 0 as INT;
    while i < DPA_GetPtrCount((*ranges).hdpa) {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/listview.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"ranges_dump\0" as *const u8 as *const ::core::ffi::c_char,
                3302 as ::core::ffi::c_int,
                b"   %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugrange(DPA_GetPtr((*ranges).hdpa, i) as *const RANGE),
            );
        }
        i += 1;
    }
}

unsafe extern "C" fn ranges_contain(ranges: RANGES, nItem: INT) -> BOOL {
    let mut srchrng: RANGE = RANGE {
        lower: nItem,
        upper: nItem + 1,
    };

    ranges_check!(ranges, b"before contain\0".as_ptr() as LPCSTR);
    return (DPA_Search(
        (*ranges).hdpa,
        core::ptr::addr_of_mut!(srchrng) as *mut core::ffi::c_void,
        0,
        Some(ranges_cmp),
        0,
        DPAS_SORTED,
    ) != -1) as BOOL;
}

unsafe extern "C" fn ranges_itemcount(ranges: RANGES) -> INT {
    let mut i: INT;
    let mut count: INT = 0;
    i = 0;
    while i < DPA_GetPtrCount((*ranges).hdpa) {
        let sel: *mut RANGE = DPA_GetPtr((*ranges).hdpa, i) as *mut RANGE;
        count += (*sel).upper - (*sel).lower;
        i += 1;
    }
    return count;
}

unsafe extern "C" fn ranges_shift(ranges: RANGES, nItem: INT, delta: INT, nUpper: INT) -> BOOL {
    let mut srchrng: RANGE = RANGE { lower: nItem, upper: nItem + 1 };
    let mut chkrng: *mut RANGE;
    let mut index: INT;
    index = DPA_Search((*ranges).hdpa, core::ptr::addr_of_mut!(srchrng) as LPVOID, 0, Some(ranges_cmp), 0, (DPAS_SORTED | DPAS_INSERTAFTER) as UINT) as INT;
    if index == -1 {
        return TRUE;
    }
    while index < DPA_GetPtrCount((*ranges).hdpa) {
        chkrng = DPA_GetPtr((*ranges).hdpa, index) as *mut RANGE;
        if (*chkrng).lower >= nItem {
            (*chkrng).lower = max(min((*chkrng).lower + delta, nUpper - 1), 0);
        }
        if (*chkrng).upper > nItem {
            (*chkrng).upper = max(min((*chkrng).upper + delta, nUpper), 0);
        }
        index += 1;
    }
    return TRUE;
}

unsafe extern "C" fn ranges_add(ranges: RANGES, range: RANGE) -> BOOL {
    let mut srchrgn: RANGE = RANGE { lower: 0, upper: 0 };
    let mut index: INT;

    ranges_check!(ranges, b"before add\0" as *const u8 as LPCSTR);

    /* try find overlapping regions first */
    srchrgn.lower = range.lower - 1;
    srchrgn.upper = range.upper + 1;
    index = DPA_Search(
        (*ranges).hdpa,
        core::ptr::addr_of_mut!(srchrgn) as *mut ::core::ffi::c_void,
        0,
        Some(ranges_cmp),
        0,
        DPAS_SORTED,
    );

    'fail: {
        if index == -1 {
            let mut newrgn: *mut RANGE = core::ptr::null_mut();

            /* create the brand new range to insert */
            newrgn = Alloc(core::mem::size_of::<RANGE>() as DWORD) as *mut RANGE;
            if newrgn.is_null() {
                break 'fail;
            }
            *newrgn = range;

            /* figure out where to insert it */
            index = DPA_Search(
                (*ranges).hdpa,
                newrgn as *mut ::core::ffi::c_void,
                0,
                Some(ranges_cmp),
                0,
                DPAS_SORTED | DPAS_INSERTAFTER,
            );
            if index == -1 {
                index = 0;
            }

            /* and get it over with */
            if DPA_InsertPtr((*ranges).hdpa, index, newrgn as *mut ::core::ffi::c_void) == -1 {
                Free(newrgn as LPVOID);
                break 'fail;
            }
        } else {
            let mut chkrgn: *mut RANGE = core::ptr::null_mut();
            let mut mrgrgn: *mut RANGE = core::ptr::null_mut();
            let mut fromindex: INT;
            let mut mergeindex: INT;

            chkrgn = DPA_GetPtr((*ranges).hdpa, index) as *mut RANGE;

            (*chkrgn).lower = min(range.lower, (*chkrgn).lower);
            (*chkrgn).upper = max(range.upper, (*chkrgn).upper);

            /* merge now common ranges */
            fromindex = 0;
            srchrgn.lower = (*chkrgn).lower - 1;
            srchrgn.upper = (*chkrgn).upper + 1;

            'dowhile: loop {
                'body: {
                    mergeindex = DPA_Search(
                        (*ranges).hdpa,
                        core::ptr::addr_of_mut!(srchrgn) as *mut ::core::ffi::c_void,
                        fromindex,
                        Some(ranges_cmp),
                        0,
                        0,
                    );
                    if mergeindex == -1 {
                        break 'dowhile;
                    }
                    if mergeindex == index {
                        fromindex = index + 1;
                        break 'body;
                    }

                    mrgrgn = DPA_GetPtr((*ranges).hdpa, mergeindex) as *mut RANGE;
                    (*chkrgn).lower = min((*chkrgn).lower, (*mrgrgn).lower);
                    (*chkrgn).upper = max((*chkrgn).upper, (*mrgrgn).upper);
                    Free(mrgrgn as LPVOID);
                    DPA_DeletePtr((*ranges).hdpa, mergeindex);
                    if mergeindex < index {
                        index -= 1;
                    }
                }
                if !(1 != 0) {
                    break 'dowhile;
                }
            }
        }

        ranges_check!(ranges, b"after add\0" as *const u8 as LPCSTR);
        return TRUE;
    }

    ranges_check!(ranges, b"failed add\0" as *const u8 as LPCSTR);
    return FALSE;
}

unsafe extern "C" fn ranges_del(mut ranges: RANGES, mut range: RANGE) -> BOOL {
    let mut chkrgn: *mut RANGE = core::ptr::null_mut();
    let mut index: INT;

    ranges_check!(ranges, b"before del\0" as *const u8 as LPCSTR);

    index = DPA_Search((*ranges).hdpa, core::ptr::addr_of_mut!(range) as *mut core::ffi::c_void, 0, Some(ranges_cmp), 0, 0) as INT;

    'fail: {
        while index != -1 {
            chkrgn = DPA_GetPtr((*ranges).hdpa, index) as *mut RANGE;

            /* case 1: Same range */
            if (*chkrgn).upper == range.upper && (*chkrgn).lower == range.lower {
                DPA_DeletePtr((*ranges).hdpa, index as core::ffi::c_int);
                Free(chkrgn as LPVOID);
                break;
            }
            /* case 2: engulf */
            else if (*chkrgn).upper <= range.upper && (*chkrgn).lower >= range.lower {
                DPA_DeletePtr((*ranges).hdpa, index as core::ffi::c_int);
                Free(chkrgn as LPVOID);
            }
            /* case 3: overlap upper */
            else if (*chkrgn).upper <= range.upper && (*chkrgn).lower < range.lower {
                (*chkrgn).upper = range.lower;
            }
            /* case 4: overlap lower */
            else if (*chkrgn).upper > range.upper && (*chkrgn).lower >= range.lower {
                (*chkrgn).lower = range.upper;
                break;
            }
            /* case 5: fully internal */
            else {
                let mut newrgn: *mut RANGE = core::ptr::null_mut();

                newrgn = Alloc(core::mem::size_of::<RANGE>() as DWORD) as *mut RANGE;
                if newrgn.is_null() {
                    break 'fail;
                }
                (*newrgn).lower = (*chkrgn).lower;
                (*newrgn).upper = range.lower;
                (*chkrgn).lower = range.upper;
                if DPA_InsertPtr((*ranges).hdpa, index as core::ffi::c_int, newrgn as *mut core::ffi::c_void) == -1 {
                    Free(newrgn as LPVOID);
                    break 'fail;
                }
                break;
            }

            index = DPA_Search((*ranges).hdpa, core::ptr::addr_of_mut!(range) as *mut core::ffi::c_void, index as core::ffi::c_int, Some(ranges_cmp), 0, 0) as INT;
        }

        ranges_check!(ranges, b"after del\0" as *const u8 as LPCSTR);
        return TRUE;
    }

    ranges_check!(ranges, b"failed del\0" as *const u8 as LPCSTR);
    return FALSE;
}

unsafe extern "C" fn LISTVIEW_DeselectAllSkipItems(infoPtr: *mut LISTVIEW_INFO, toSkip: RANGES) -> BOOL {
    let mut lvItem: LVITEMW = core::mem::zeroed();
    let mut i: ITERATOR = core::mem::zeroed();
    let mut clone: RANGES = core::ptr::null_mut();

    lvItem.state = 0;
    lvItem.stateMask = LVIS_SELECTED;

    /* need to clone the DPA because callbacks can change it */
    clone = ranges_clone((*infoPtr).selectionRanges);
    if clone.is_null() {
        return FALSE;
    }
    iterator_rangesitems(&raw mut i, ranges_diff(clone, toSkip));
    while iterator_next(&raw mut i) != 0 {
        LISTVIEW_SetItemState(infoPtr, i.nItem, &raw mut lvItem);
    }
    /* note that the iterator destructor will free the cloned range */
    iterator_destroy(&raw mut i);

    return TRUE;
}

unsafe extern "C" fn LISTVIEW_DeselectAllSkipItem(infoPtr: *mut LISTVIEW_INFO, nItem: INT) -> BOOL {
    let mut toSkip: RANGES = core::ptr::null_mut();
    toSkip = ranges_create(1);
    if toSkip.is_null() {
        return FALSE;
    }
    if nItem != -1 {
        ranges_additem(toSkip, nItem);
    }
    LISTVIEW_DeselectAllSkipItems(infoPtr, toSkip);
    ranges_destroy(toSkip);
    return TRUE;
}

unsafe extern "C" fn LISTVIEW_DeselectAll(infoPtr: *mut LISTVIEW_INFO) -> BOOL {
    return LISTVIEW_DeselectAllSkipItem(infoPtr, -1);
}

unsafe extern "C" fn LISTVIEW_GetSelectedCount(infoPtr: *const LISTVIEW_INFO) -> INT {
    let mut nSelectedCount: INT = 0;
    if (*infoPtr).uCallbackMask & LVIS_SELECTED != 0 {
        let mut i: INT = 0;
        i = 0;
        while i < (*infoPtr).nItemCount {
            if LISTVIEW_GetItemState(infoPtr, i, LVIS_SELECTED as UINT) != 0 {
                nSelectedCount += 1;
            }
            i += 1;
        }
    } else {
        nSelectedCount = ranges_itemcount((*infoPtr).selectionRanges);
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
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_GetSelectedCount\0" as *const u8 as *const ::core::ffi::c_char,
            3578 as ::core::ffi::c_int,
            b"nSelectedCount=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            nSelectedCount,
        );
    }
    return nSelectedCount;
}

unsafe extern "C" fn LISTVIEW_SetItemFocus(infoPtr: *mut LISTVIEW_INFO, nItem: INT) -> BOOL {
    let oldFocus: INT = (*infoPtr).nFocusedItem;
    let mut lvItem: LVITEMW = core::mem::zeroed();

    if nItem == (*infoPtr).nFocusedItem {
        return FALSE;
    }

    lvItem.state = (if nItem == -1 {
        0
    } else {
        LVIS_FOCUSED
    }) as UINT;
    lvItem.stateMask = LVIS_FOCUSED as UINT;
    LISTVIEW_SetItemState(
        infoPtr,
        if nItem == -1 {
            (*infoPtr).nFocusedItem
        } else {
            nItem
        },
        core::ptr::addr_of_mut!(lvItem),
    );

    return (oldFocus != (*infoPtr).nFocusedItem) as BOOL;
}

unsafe extern "C" fn shift_item(infoPtr: *const LISTVIEW_INFO, nShiftItem: INT, nItem: INT, direction: INT) -> INT {
    if nShiftItem < nItem { return nShiftItem; }

    if nShiftItem > nItem { return nShiftItem + direction; }

    if direction > 0 { return nShiftItem + direction; }

    return min(nShiftItem, (*infoPtr).nItemCount - 1);
}

unsafe extern "C" fn LISTVIEW_ShiftFocus(infoPtr: *mut LISTVIEW_INFO, mut focus: INT, item: INT, direction: INT) {
    let old_mask: DWORD = (*infoPtr).notify_mask & NOTIFY_MASK_ITEM_CHANGE as DWORD;

    (*infoPtr).notify_mask &= !(NOTIFY_MASK_ITEM_CHANGE as DWORD);
    focus = shift_item(infoPtr, focus, item, direction);
    if focus != (*infoPtr).nFocusedItem {
        LISTVIEW_SetItemFocus(infoPtr, focus);
    }
    (*infoPtr).notify_mask |= old_mask;
}

unsafe extern "C" fn LISTVIEW_ShiftIndices(infoPtr: *mut LISTVIEW_INFO, nItem: INT, direction: INT) {
    ranges_shift((*infoPtr).selectionRanges, nItem, direction, (*infoPtr).nItemCount);
    assert!((abs(direction) == 1) as INT);
    (*infoPtr).nSelectionMark = shift_item(infoPtr, (*infoPtr).nSelectionMark, nItem, direction);

    /* But we are not supposed to modify nHotItem! */
}

unsafe extern "C" fn LISTVIEW_AddGroupSelection(infoPtr: *mut LISTVIEW_INFO, nItem: INT) -> BOOL {
    let mut nFirst: INT = min((*infoPtr).nSelectionMark, nItem);
    let mut nLast: INT = max((*infoPtr).nSelectionMark, nItem);
    let hwndSelf: HWND = (*infoPtr).hwndSelf;
    let mut nmlv: NMLVODSTATECHANGE = core::mem::zeroed();
    let mut old_mask: DWORD = 0;
    let mut item: LVITEMW = core::mem::zeroed();
    let mut i: INT = 0;

    old_mask = (*infoPtr).notify_mask & (NOTIFY_MASK_ITEM_CHANGE as DWORD);
    if ((*infoPtr).dwStyle & (LVS_OWNERDATA as DWORD)) != 0 {
        (*infoPtr).notify_mask &= !(NOTIFY_MASK_ITEM_CHANGE as DWORD);
    }

    if nFirst == -(1 as INT) {
        nFirst = nItem;
    }

    item.state = LVIS_SELECTED as UINT;
    item.stateMask = LVIS_SELECTED as UINT;

    i = nFirst;
    while i <= nLast {
        LISTVIEW_SetItemState(infoPtr, i, core::ptr::addr_of_mut!(item));
        i += 1;
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(nmlv) as *mut core::ffi::c_void,
        core::mem::size_of::<NMLVODSTATECHANGE>(),
    );
    nmlv.iFrom = nFirst;
    nmlv.iTo = nLast;
    nmlv.uOldState = 0 as UINT;
    nmlv.uNewState = item.state;

    notify_hdr(
        infoPtr,
        LVN_ODSTATECHANGED as INT,
        core::ptr::addr_of_mut!(nmlv) as LPNMHDR,
    );
    if IsWindow(hwndSelf) == 0 {
        return FALSE as BOOL;
    }
    (*infoPtr).notify_mask |= old_mask;
    return TRUE as BOOL;
}

unsafe extern "C" fn LISTVIEW_SetGroupSelection(infoPtr: *mut LISTVIEW_INFO, nItem: INT) {
    let mut selection: RANGES = core::ptr::null_mut();
    let mut old_mask: DWORD = 0;
    let mut item: LVITEMW = core::mem::zeroed();
    let mut i: ITERATOR = core::mem::zeroed();

    selection = ranges_create(100);
    if selection.is_null() {
        return;
    }

    item.state = LVIS_SELECTED as UINT;
    item.stateMask = LVIS_SELECTED as UINT;

    if (*infoPtr).nSelectionMark == -(1 as INT) {
        (*infoPtr).nSelectionMark = nItem;
        ranges_additem(selection, nItem);
    } else {
        let mut sel: RANGE = core::mem::zeroed();

        sel.lower = min((*infoPtr).nSelectionMark, nItem);
        sel.upper = max((*infoPtr).nSelectionMark, nItem) + 1 as INT;
        ranges_add(selection, sel);
    }

    /* disable per item notifications on LVS_OWNERDATA style
       FIXME: single LVN_ODSTATECHANGED should be used */
    old_mask = (*infoPtr).notify_mask & (NOTIFY_MASK_ITEM_CHANGE as DWORD);
    if ((*infoPtr).dwStyle & (LVS_OWNERDATA as DWORD)) != 0 {
        (*infoPtr).notify_mask &= !(NOTIFY_MASK_ITEM_CHANGE as DWORD);
    }

    LISTVIEW_DeselectAllSkipItems(infoPtr, selection);

    iterator_rangesitems(core::ptr::addr_of_mut!(i), selection);
    while iterator_next(core::ptr::addr_of_mut!(i)) != 0 {
        LISTVIEW_SetItemState(infoPtr, i.nItem, core::ptr::addr_of_mut!(item));
    }
    /* this will also destroy the selection */
    iterator_destroy(core::ptr::addr_of_mut!(i));

    (*infoPtr).notify_mask |= old_mask;
    LISTVIEW_SetItemFocus(infoPtr, nItem);
}

unsafe extern "C" fn LISTVIEW_SetSelection(infoPtr: *mut LISTVIEW_INFO, nItem: INT) {
    let mut lvItem: LVITEMW = core::mem::zeroed();

    LISTVIEW_DeselectAllSkipItem(infoPtr, nItem);

    lvItem.state = (LVIS_FOCUSED | LVIS_SELECTED) as UINT;
    lvItem.stateMask = (LVIS_FOCUSED | LVIS_SELECTED) as UINT;
    LISTVIEW_SetItemState(infoPtr, nItem, core::ptr::addr_of_mut!(lvItem));

    (*infoPtr).nSelectionMark = nItem;
}

unsafe extern "C" fn LISTVIEW_KeySelection(infoPtr: *mut LISTVIEW_INFO, nItem: INT, keys: DWORD) -> BOOL {
    let mut wShift: WORD = ((keys & (SHIFT_KEY as DWORD)) != 0) as WORD;
    let mut wCtrl: WORD = ((keys & (CTRL_KEY as DWORD)) != 0) as WORD;
    let mut bResult: BOOL = FALSE as BOOL;
    if nItem >= (0 as INT) && nItem < (*infoPtr).nItemCount {
        bResult = TRUE as BOOL;
        if ((*infoPtr).dwStyle & (LVS_SINGLESEL as DWORD)) != 0
            || ((wShift as INT) == (0 as INT)
                && (wCtrl as INT) == (0 as INT))
        {
            LISTVIEW_SetSelection(infoPtr, nItem);
        } else {
            if wShift != 0 {
                LISTVIEW_SetGroupSelection(infoPtr, nItem);
            } else if wCtrl != 0 {
                let mut lvItem: LVITEMW = core::mem::zeroed();
                lvItem.state = !(LISTVIEW_GetItemState(infoPtr, nItem, LVIS_SELECTED as UINT));
                lvItem.stateMask = LVIS_SELECTED as UINT;
                if (keys & (SPACE_KEY as DWORD)) != 0 {
                    LISTVIEW_SetItemState(infoPtr, nItem, &raw mut lvItem);
                    if (lvItem.state & (LVIS_SELECTED as UINT)) != 0 {
                        (*infoPtr).nSelectionMark = nItem;
                    }
                }
                bResult = LISTVIEW_SetItemFocus(infoPtr, nItem);
            }
        }
        LISTVIEW_EnsureVisible(infoPtr, nItem, FALSE as BOOL);
    }

    UpdateWindow((*infoPtr).hwndSelf); /* update client area */
    return bResult;
}

unsafe extern "C" fn LISTVIEW_GetItemAtPt(infoPtr: *const LISTVIEW_INFO, lpLVItem: LPLVITEMW, pt: POINT) -> BOOL {
    let mut lvHitTestInfo: LVHITTESTINFO = core::mem::zeroed();
    ZeroMemory!(
        core::ptr::addr_of_mut!(lvHitTestInfo) as *mut core::ffi::c_void,
        core::mem::size_of::<LVHITTESTINFO>() as usize,
    );
    lvHitTestInfo.pt.x = pt.x;
    lvHitTestInfo.pt.y = pt.y;

    LISTVIEW_HitTest(infoPtr, core::ptr::addr_of_mut!(lvHitTestInfo), TRUE, FALSE);

    (*lpLVItem).mask = LVIF_PARAM;
    (*lpLVItem).iItem = lvHitTestInfo.iItem;
    (*lpLVItem).iSubItem = 0;

    return LISTVIEW_GetItemT(infoPtr, lpLVItem, TRUE);
}

unsafe extern "C" fn LISTVIEW_IsHotTracking(infoPtr: *const LISTVIEW_INFO) -> BOOL {
    return (((*infoPtr).dwLvExStyle & LVS_EX_TRACKSELECT != 0)
        || ((*infoPtr).dwLvExStyle & LVS_EX_ONECLICKACTIVATE != 0)
        || ((*infoPtr).dwLvExStyle & LVS_EX_TWOCLICKACTIVATE != 0)) as BOOL;
}

unsafe extern "C" fn LISTVIEW_MouseHover(infoPtr: *mut LISTVIEW_INFO, x: INT, y: INT) -> LRESULT {
    let mut hdr: NMHDR = core::mem::zeroed();

    if notify_hdr(infoPtr, NM_HOVER as INT, core::ptr::addr_of_mut!(hdr)) != 0 {
        return 0;
    }

    if LISTVIEW_IsHotTracking(infoPtr) != 0 {
        let mut item: LVITEMW = core::mem::zeroed();
        let mut pt: POINT = core::mem::zeroed();

        pt.x = x;
        pt.y = y;

        if LISTVIEW_GetItemAtPt(infoPtr, core::ptr::addr_of_mut!(item), pt) != 0 {
            LISTVIEW_SetSelection(infoPtr, item.iItem);
        }

        SetFocus((*infoPtr).hwndSelf);
    }

    return 0;
}

unsafe extern "C" fn LISTVIEW_MarqueeHighlight(
    infoPtr: *mut LISTVIEW_INFO,
    coords_orig: *const POINT,
    scroll: INT,
) {
    let mut controlDown: BOOL = FALSE;
    let mut item: LVITEMW = core::mem::zeroed();
    let mut old_elems: ITERATOR = core::mem::zeroed();
    let mut new_elems: ITERATOR = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let mut coords_offs: POINT = core::mem::zeroed();
    let mut offset: POINT = core::mem::zeroed();

    /* Ensure coordinates are within client bounds */
    coords_offs.x = max(min((*coords_orig).x, (*infoPtr).rcList.right), 0) as LONG;
    coords_offs.y = max(min((*coords_orig).y, (*infoPtr).rcList.bottom), 0) as LONG;

    /* Get offset */
    LISTVIEW_GetOrigin(infoPtr, core::ptr::addr_of_mut!(offset));

    /* Offset coordinates by the appropriate amount */
    coords_offs.x -= offset.x;
    coords_offs.y -= offset.y;

    if coords_offs.x > (*infoPtr).marqueeOrigin.x {
        rect.left = (*infoPtr).marqueeOrigin.x;
        rect.right = coords_offs.x;
    } else {
        rect.left = coords_offs.x;
        rect.right = (*infoPtr).marqueeOrigin.x;
    }

    if coords_offs.y > (*infoPtr).marqueeOrigin.y {
        rect.top = (*infoPtr).marqueeOrigin.y;
        rect.bottom = coords_offs.y;
    } else {
        rect.top = coords_offs.y;
        rect.bottom = (*infoPtr).marqueeOrigin.y;
    }

    /* Cancel out the old marquee rectangle and draw the new one */
    LISTVIEW_InvalidateRect(infoPtr, core::ptr::addr_of_mut!((*infoPtr).marqueeDrawRect));

    /* Scroll by the appropriate distance if applicable - speed up scrolling as
       the cursor is further away */

    if ((scroll & SCROLL_LEFT) != 0) && ((*coords_orig).x <= 0) {
        LISTVIEW_Scroll(infoPtr, (*coords_orig).x, 0);
    }

    if ((scroll & SCROLL_RIGHT) != 0) && ((*coords_orig).x >= (*infoPtr).rcList.right) {
        LISTVIEW_Scroll(infoPtr, ((*coords_orig).x - (*infoPtr).rcList.right), 0);
    }

    if ((scroll & SCROLL_UP) != 0) && ((*coords_orig).y <= 0) {
        LISTVIEW_Scroll(infoPtr, 0, (*coords_orig).y);
    }

    if ((scroll & SCROLL_DOWN) != 0) && ((*coords_orig).y >= (*infoPtr).rcList.bottom) {
        LISTVIEW_Scroll(infoPtr, 0, ((*coords_orig).y - (*infoPtr).rcList.bottom));
    }

    iterator_frameditems_absolute(core::ptr::addr_of_mut!(old_elems), infoPtr, core::ptr::addr_of_mut!((*infoPtr).marqueeRect));

    (*infoPtr).marqueeRect = rect;
    (*infoPtr).marqueeDrawRect = rect;
    OffsetRect(core::ptr::addr_of_mut!((*infoPtr).marqueeDrawRect), offset.x, offset.y);

    iterator_frameditems_absolute(core::ptr::addr_of_mut!(new_elems), infoPtr, core::ptr::addr_of_mut!((*infoPtr).marqueeRect));
    iterator_remove_common_items(core::ptr::addr_of_mut!(old_elems), core::ptr::addr_of_mut!(new_elems));

    /* Iterate over no longer selected items */
    while iterator_next(core::ptr::addr_of_mut!(old_elems)) != 0 {
        if old_elems.nItem > -1 {
            if LISTVIEW_GetItemState(infoPtr, old_elems.nItem, LVIS_SELECTED as UINT)
                == LVIS_SELECTED as UINT
            {
                item.state = 0 as UINT;
            } else {
                item.state = LVIS_SELECTED as UINT;
            }

            item.stateMask = LVIS_SELECTED as UINT;

            LISTVIEW_SetItemState(infoPtr, old_elems.nItem, core::ptr::addr_of_mut!(item));
        }
    }
    iterator_destroy(core::ptr::addr_of_mut!(old_elems));

    /* Iterate over newly selected items */
    if ((GetKeyState(VK_CONTROL as ::core::ffi::c_int) as ::core::ffi::c_int) & 0x8000) != 0 {
        controlDown = TRUE;
    }

    while iterator_next(core::ptr::addr_of_mut!(new_elems)) != 0 {
        if new_elems.nItem > -1 {
            /* If CTRL is pressed, invert. If not, always select the item. */
            if (controlDown != 0)
                && (LISTVIEW_GetItemState(infoPtr, new_elems.nItem, LVIS_SELECTED as UINT) != 0)
            {
                item.state = 0 as UINT;
            } else {
                item.state = LVIS_SELECTED as UINT;
            }

            item.stateMask = LVIS_SELECTED as UINT;

            LISTVIEW_SetItemState(infoPtr, new_elems.nItem, core::ptr::addr_of_mut!(item));
        }
    }
    iterator_destroy(core::ptr::addr_of_mut!(new_elems));

    LISTVIEW_InvalidateRect(infoPtr, core::ptr::addr_of_mut!((*infoPtr).marqueeDrawRect));
}

unsafe extern "system" fn LISTVIEW_ScrollTimer(hWnd: HWND, uMsg: UINT, idEvent: UINT_PTR, dwTime: DWORD) {
    let mut infoPtr: *mut LISTVIEW_INFO = core::ptr::null_mut::<LISTVIEW_INFO>();
    let mut scrollInfo: SCROLLINFO = core::mem::zeroed::<SCROLLINFO>();
    let mut coords: POINT = core::mem::zeroed::<POINT>();
    let mut scroll: INT = 0;

    infoPtr = idEvent as *mut LISTVIEW_INFO;

    if infoPtr.is_null() {
        return;
    }

    /* Get the current cursor position and convert to client coordinates */
    GetCursorPos(&mut coords);
    ScreenToClient(hWnd, &mut coords);

    scrollInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    scrollInfo.fMask = SIF_ALL as UINT;

    /* Work out in which directions we can scroll */
    if GetScrollInfo((*infoPtr).hwndSelf, SB_VERT as INT, &mut scrollInfo) != 0 {
        if scrollInfo.nPos != scrollInfo.nMin {
            scroll |= SCROLL_UP;
        }

        if scrollInfo
            .nPage
            .wrapping_add(scrollInfo.nPos as UINT)
            .wrapping_sub(1 as UINT)
            != scrollInfo.nMax as UINT
        {
            scroll |= SCROLL_DOWN;
        }
    }

    if GetScrollInfo((*infoPtr).hwndSelf, SB_HORZ as INT, &mut scrollInfo) != 0 {
        if scrollInfo.nPos != scrollInfo.nMin {
            scroll |= SCROLL_LEFT;
        }

        if scrollInfo
            .nPage
            .wrapping_add(scrollInfo.nPos as UINT)
            .wrapping_sub(1 as UINT)
            != scrollInfo.nMax as UINT
        {
            scroll |= SCROLL_RIGHT;
        }
    }

    if ((coords.x <= 0) && (scroll & SCROLL_LEFT) != 0)
        || ((coords.y <= 0) && (scroll & SCROLL_UP) != 0)
        || ((coords.x >= (*infoPtr).rcList.right) && (scroll & SCROLL_RIGHT) != 0)
        || ((coords.y >= (*infoPtr).rcList.bottom) && (scroll & SCROLL_DOWN) != 0)
    {
        LISTVIEW_MarqueeHighlight(infoPtr, &mut coords, scroll);
    }
}

unsafe extern "C" fn LISTVIEW_MouseMove(
    infoPtr: *mut LISTVIEW_INFO,
    fwKeys: WORD,
    x: INT,
    y: INT,
) -> LRESULT {
    let mut ht: LVHITTESTINFO = tagLVHITTESTINFO {
        pt: tagPOINT { x: 0, y: 0 },
        flags: 0,
        iItem: 0,
        iSubItem: 0,
    };
    let mut rect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut pt: POINT = tagPOINT { x: 0, y: 0 };

    pt.x = x as LONG;
    pt.y = y as LONG;

    if ((fwKeys as INT) & (MK_LBUTTON as INT)) == 0 {
        (*infoPtr).bLButtonDown = (FALSE as BOOL);
    }

    if (*infoPtr).bLButtonDown != 0 {
        rect.right = (*infoPtr).ptClickPos.x;
        rect.left = rect.right;
        rect.bottom = (*infoPtr).ptClickPos.y;
        rect.top = rect.bottom;

        InflateRect(
            &raw mut rect,
            GetSystemMetrics(SM_CXDRAG as INT),
            GetSystemMetrics(SM_CYDRAG as INT),
        );

        if (*infoPtr).bMarqueeSelect != 0 {
            /* Enable the timer if we're going outside our bounds, in case the user doesn't
               move the mouse again */

            if (x <= (0 as INT))
                || (y <= (0 as INT))
                || (x >= (*infoPtr).rcList.right)
                || (y >= (*infoPtr).rcList.bottom)
            {
                if (*infoPtr).bScrolling == 0 {
                    (*infoPtr).bScrolling = (TRUE as BOOL);
                    SetTimer(
                        (*infoPtr).hwndSelf,
                        infoPtr as UINT_PTR,
                        (1 as UINT),
                        Some(
                            LISTVIEW_ScrollTimer,
                        ),
                    );
                }
            } else {
                (*infoPtr).bScrolling = (FALSE as BOOL);
                KillTimer((*infoPtr).hwndSelf, infoPtr as UINT_PTR);
            }

            LISTVIEW_MarqueeHighlight(infoPtr, &raw mut pt, (0 as INT));
            return (0 as LRESULT);
        }

        ht.pt = pt;
        LISTVIEW_HitTest(
            infoPtr,
            &raw mut ht,
            (TRUE as BOOL),
            (TRUE as BOOL),
        );

        /* reset item marker */
        if (*infoPtr).nLButtonDownItem != ht.iItem {
            (*infoPtr).nLButtonDownItem = -(1 as INT);
        }

        if PtInRect(&raw mut rect as LPCRECT, pt) == 0 {
            /* this path covers the following:
               1. WM_LBUTTONDOWN over selected item (sets focus on it)
               2. change focus with keys
               3. move mouse over item from step 1 selects it and moves focus on it */
            if (*infoPtr).nLButtonDownItem != -(1 as INT)
                && LISTVIEW_GetItemState(
                    infoPtr,
                    (*infoPtr).nLButtonDownItem,
                    (LVIS_SELECTED as UINT),
                ) == 0
            {
                let mut lvItem: LVITEMW = tagLVITEMW {
                    mask: 0,
                    iItem: 0,
                    iSubItem: 0,
                    state: 0,
                    stateMask: 0,
                    pszText: ::core::ptr::null_mut::<WCHAR>(),
                    cchTextMax: 0,
                    iImage: 0,
                    lParam: 0,
                    iIndent: 0,
                    iGroupId: 0,
                    cColumns: 0,
                    puColumns: ::core::ptr::null_mut::<::core::ffi::c_uint>(),
                };

                lvItem.state = ((LVIS_FOCUSED as UINT) | (LVIS_SELECTED as UINT));
                lvItem.stateMask =
                    ((LVIS_FOCUSED as UINT) | (LVIS_SELECTED as UINT));

                LISTVIEW_SetItemState(
                    infoPtr,
                    (*infoPtr).nLButtonDownItem,
                    &raw mut lvItem,
                );
                (*infoPtr).nLButtonDownItem = -(1 as INT);
            }

            if (*infoPtr).bDragging == 0 {
                ht.pt = (*infoPtr).ptClickPos;
                LISTVIEW_HitTest(
                    infoPtr,
                    &raw mut ht,
                    (TRUE as BOOL),
                    (TRUE as BOOL),
                );

                /* If the click is outside the range of an item, begin a
                   highlight. If not, begin an item drag. */
                if ht.iItem == -(1 as INT) {
                    let mut hdr: NMHDR = tagNMHDR {
                        hwndFrom: ::core::ptr::null_mut::<HWND__>(),
                        idFrom: 0,
                        code: 0,
                    };

                    /* If we're allowing multiple selections, send notification.
                       If return value is non-zero, cancel. */
                    if ((*infoPtr).dwStyle & (LVS_SINGLESEL as DWORD)) == 0
                        && (notify_hdr(
                            infoPtr,
                            (LVN_MARQUEEBEGIN as INT),
                            &raw mut hdr,
                        ) == (0 as LRESULT))
                    {
                        /* Store the absolute coordinates of the click */
                        let mut offset: POINT = tagPOINT { x: 0, y: 0 };
                        LISTVIEW_GetOrigin(infoPtr, &raw mut offset);

                        (*infoPtr).marqueeOrigin.x = (*infoPtr).ptClickPos.x - offset.x;
                        (*infoPtr).marqueeOrigin.y = (*infoPtr).ptClickPos.y - offset.y;

                        /* Begin selection and capture mouse */
                        (*infoPtr).bMarqueeSelect = (TRUE as BOOL);
                        (*infoPtr).marqueeRect = rect;
                        SetCapture((*infoPtr).hwndSelf);
                    }
                } else {
                    let mut nmlv: NMLISTVIEW = tagNMLISTVIEW {
                        hdr: tagNMHDR {
                            hwndFrom: ::core::ptr::null_mut::<HWND__>(),
                            idFrom: 0,
                            code: 0,
                        },
                        iItem: 0,
                        iSubItem: 0,
                        uNewState: 0,
                        uOldState: 0,
                        uChanged: 0,
                        ptAction: tagPOINT { x: 0, y: 0 },
                        lParam: 0,
                    };

                    ZeroMemory!(
                        &raw mut nmlv as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<NMLISTVIEW>() as size_t,
                    );
                    nmlv.iItem = ht.iItem;
                    nmlv.ptAction = (*infoPtr).ptClickPos;

                    notify_listview(
                        infoPtr,
                        (LVN_BEGINDRAG as INT),
                        &raw mut nmlv,
                    );
                    (*infoPtr).bDragging = (TRUE as BOOL);
                }
            }

            return (0 as LRESULT);
        }
    }

    /* see if we are supposed to be tracking mouse hovering */
    if LISTVIEW_IsHotTracking(infoPtr) != 0 {
        let mut trackinfo: TRACKMOUSEEVENT = tagTRACKMOUSEEVENT {
            cbSize: 0,
            dwFlags: 0,
            hwndTrack: ::core::ptr::null_mut::<HWND__>(),
            dwHoverTime: 0,
        };
        let mut flags: DWORD = 0;

        trackinfo.cbSize = (core::mem::size_of::<TRACKMOUSEEVENT>() as DWORD);
        trackinfo.dwFlags = (TME_QUERY as DWORD);

        /* see if we are already tracking this hwnd */
        _TrackMouseEvent(&raw mut trackinfo);

        flags = (TME_LEAVE as DWORD);
        if (*infoPtr).dwLvExStyle & (LVS_EX_TRACKSELECT as DWORD) != 0 {
            flags |= (TME_HOVER as DWORD);
        }

        if (trackinfo.dwFlags & flags) != flags || trackinfo.hwndTrack != (*infoPtr).hwndSelf {
            trackinfo.dwFlags = flags;
            trackinfo.dwHoverTime = (*infoPtr).dwHoverTime;
            trackinfo.hwndTrack = (*infoPtr).hwndSelf;

            /* call TRACKMOUSEEVENT so we receive WM_MOUSEHOVER messages */
            _TrackMouseEvent(&raw mut trackinfo);
        }
    }

    return (0 as LRESULT);
}

unsafe extern "C" fn is_assignable_item(lpLVItem: *const LVITEMW, lStyle: LONG) -> BOOL {
    if ((*lpLVItem).mask & LVIF_TEXT as UINT) != 0
        && ((*lpLVItem).pszText == LPSTR_TEXTCALLBACKW)
        && ((lStyle as UINT) & (LVS_SORTASCENDING | LVS_SORTDESCENDING)) != 0
    {
        return FALSE;
    }

    return TRUE;
}

unsafe extern "C" fn set_main_item(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut lpLVItem: *const LVITEMW,
    mut isNew: BOOL,
    mut isW: BOOL,
    mut bChanged: *mut BOOL,
) -> BOOL {
    let mut lpItem: *mut ITEM_INFO = core::ptr::null_mut::<ITEM_INFO>();
    let mut nmlv: NMLISTVIEW = core::mem::zeroed();
    let mut uChanged: UINT = 0 as UINT;
    let mut item: LVITEMW = core::mem::zeroed();
    /* stateMask is ignored for LVM_INSERTITEM */
    let mut stateMask: UINT = if isNew != 0 {
        !(0 as UINT)
    } else {
        (*lpLVItem).stateMask
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
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"set_main_item\0" as *const u8 as *const ::core::ffi::c_char,
            4311 as ::core::ffi::c_int,
            b"()\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }

    assert!(
        ((*lpLVItem).iItem >= 0 && (*lpLVItem).iItem < (*infoPtr).nItemCount)
            as ::core::ffi::c_int,
    );

    if (*lpLVItem).mask == 0 as UINT {
        return TRUE as BOOL;
    }

    if (*infoPtr).dwStyle & LVS_OWNERDATA as DWORD != 0 {
        /* a virtual listview only stores selection and focus */
        if (*lpLVItem).mask & !(LVIF_STATE as UINT) != 0 {
            return FALSE as BOOL;
        }
        lpItem = core::ptr::null_mut::<ITEM_INFO>();
    } else {
        let mut hdpaSubItems: HDPA = DPA_GetPtr(
            (*infoPtr).hdpaItems,
            (*lpLVItem).iItem as INT,
        ) as HDPA;
        lpItem = DPA_GetPtr(hdpaSubItems, 0 as INT)
            as *mut ITEM_INFO;
        assert!(!lpItem.is_null() as ::core::ffi::c_int);
    }

    /* we need to get the lParam and state of the item */
    item.iItem = (*lpLVItem).iItem;
    item.iSubItem = (*lpLVItem).iSubItem;
    item.mask = (LVIF_STATE | LVIF_PARAM) as UINT;
    item.stateMask = if (*infoPtr).dwStyle & LVS_OWNERDATA as DWORD != 0 {
        (LVIS_FOCUSED | LVIS_SELECTED) as UINT
    } else {
        !(0 as UINT)
    };

    item.state = 0 as UINT;
    item.lParam = 0 as LPARAM;
    if isNew == 0 && LISTVIEW_GetItemW(infoPtr, &raw mut item) == 0 {
        return FALSE as BOOL;
    }

    /* determine what fields will change */
    if (*lpLVItem).mask & LVIF_STATE as UINT != 0
        && ((item.state ^ (*lpLVItem).state) & stateMask & !(*infoPtr).uCallbackMask) != 0
    {
        uChanged |= LVIF_STATE as UINT;
    }

    if (*lpLVItem).mask & LVIF_IMAGE as UINT != 0
        && (*lpItem).hdr.iImage != (*lpLVItem).iImage
    {
        uChanged |= LVIF_IMAGE as UINT;
    }

    if (*lpLVItem).mask & LVIF_PARAM as UINT != 0
        && (*lpItem).lParam != (*lpLVItem).lParam
    {
        uChanged |= LVIF_PARAM as UINT;
    }

    if (*lpLVItem).mask & LVIF_INDENT as UINT != 0
        && (*lpItem).iIndent != (*lpLVItem).iIndent
    {
        uChanged |= LVIF_INDENT as UINT;
    }

    if (*lpLVItem).mask & LVIF_TEXT as UINT != 0
        && textcmpWT(
            (*lpItem).hdr.pszText as LPCWSTR,
            (*lpLVItem).pszText as LPCWSTR,
            isW,
        ) != 0
    {
        uChanged |= LVIF_TEXT as UINT;
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
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"set_main_item\0" as *const u8 as *const ::core::ffi::c_char,
            4358 as ::core::ffi::c_int,
            b"change mask=0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
            uChanged,
        );
    }

    memset(
        &raw mut nmlv as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<NMLISTVIEW>() as size_t,
    );
    nmlv.iItem = (*lpLVItem).iItem;
    if (*lpLVItem).mask & LVIF_STATE as UINT != 0 {
        nmlv.uNewState = (item.state & !stateMask) | ((*lpLVItem).state & stateMask);
        nmlv.uOldState = item.state;
    }
    nmlv.uChanged = if uChanged != 0 {
        uChanged
    } else {
        (*lpLVItem).mask
    };
    nmlv.lParam = item.lParam;

    /* Send LVN_ITEMCHANGING notification, if the item is not being inserted
       and we are _NOT_ virtual (LVS_OWNERDATA), and change notifications
       are enabled. Even nothing really changed we still need to send this,
       in this case uChanged mask is just set to passed item mask. */
    if !lpItem.is_null()
        && isNew == 0
        && (*infoPtr).notify_mask & NOTIFY_MASK_ITEM_CHANGE as DWORD != 0
    {
        let mut hwndSelf: HWND = (*infoPtr).hwndSelf;

        if notify_listview(infoPtr, LVN_ITEMCHANGING as INT, &raw mut nmlv) != 0 {
            return FALSE as BOOL;
        }
        if IsWindow(hwndSelf) == 0 {
            return FALSE as BOOL;
        }
    }

    /* When item is inserted we need to shift existing focus index if new item has lower index. */
    if isNew != 0
        && (stateMask & !(*infoPtr).uCallbackMask & LVIS_FOCUSED as UINT) != 0
        && (
            /* this means we won't hit a focus change path later */
            (uChanged & LVIF_STATE as UINT) == 0 as UINT
                || (((*lpLVItem).state & LVIS_FOCUSED as UINT) == 0
                    && (*infoPtr).nFocusedItem != (*lpLVItem).iItem)
        )
    {
        if (*infoPtr).nFocusedItem != -(1 as INT)
            && (*lpLVItem).iItem <= (*infoPtr).nFocusedItem
        {
            (*infoPtr).nFocusedItem += 1;
        }
    }

    if uChanged == 0 {
        return TRUE as BOOL;
    }
    *bChanged = TRUE as BOOL;

    /* copy information */
    if (*lpLVItem).mask & LVIF_TEXT as UINT != 0 {
        textsetptrT(
            &raw mut (*lpItem).hdr.pszText,
            (*lpLVItem).pszText as LPCWSTR,
            isW,
        );
    }

    if (*lpLVItem).mask & LVIF_IMAGE as UINT != 0 {
        (*lpItem).hdr.iImage = (*lpLVItem).iImage as INT;
    }

    if (*lpLVItem).mask & LVIF_PARAM as UINT != 0 {
        (*lpItem).lParam = (*lpLVItem).lParam;
    }

    if (*lpLVItem).mask & LVIF_INDENT as UINT != 0 {
        (*lpItem).iIndent = (*lpLVItem).iIndent as INT;
    }

    if uChanged & LVIF_STATE as UINT != 0 {
        if !lpItem.is_null() && (stateMask & !(*infoPtr).uCallbackMask) != 0 {
            (*lpItem).state &= !stateMask;
            (*lpItem).state |= (*lpLVItem).state & stateMask;
        }
        if (*lpLVItem).state & stateMask & !(*infoPtr).uCallbackMask
            & LVIS_SELECTED as UINT
            != 0
        {
            if (*infoPtr).dwStyle & LVS_SINGLESEL as DWORD != 0 {
                LISTVIEW_DeselectAllSkipItem(infoPtr, (*lpLVItem).iItem as INT);
            }
            ranges_additem((*infoPtr).selectionRanges, (*lpLVItem).iItem as INT);
        } else if stateMask & LVIS_SELECTED as UINT != 0 {
            ranges_delitem((*infoPtr).selectionRanges, (*lpLVItem).iItem as INT);
        }
        /* If we are asked to change focus, and we manage it, do it.
           It's important to have all new item data stored at this point,
           because changing existing focus could result in a redrawing operation,
           which in turn could ask for disp data, application should see all data
           for inserted item when processing LVN_GETDISPINFO.

           The way this works application will see nested item change notifications -
           changed item notifications interrupted by ones from item losing focus. */
        if stateMask & !(*infoPtr).uCallbackMask & LVIS_FOCUSED as UINT != 0 {
            if (*lpLVItem).state & LVIS_FOCUSED as UINT != 0 {
                /* update selection mark */
                if (*infoPtr).nFocusedItem == -(1 as INT)
                    && (*infoPtr).nSelectionMark == -(1 as INT)
                {
                    (*infoPtr).nSelectionMark = (*lpLVItem).iItem as INT;
                }

                if (*infoPtr).nFocusedItem != -(1 as INT) {
                    /* remove current focus */
                    item.mask = LVIF_STATE as UINT;
                    item.state = 0 as UINT;
                    item.stateMask = LVIS_FOCUSED as UINT;

                    /* recurse with redrawing an item */
                    LISTVIEW_SetItemState(infoPtr, (*infoPtr).nFocusedItem, &raw mut item);
                }

                (*infoPtr).nFocusedItem = (*lpLVItem).iItem as INT;
                LISTVIEW_EnsureVisible(
                    infoPtr,
                    (*lpLVItem).iItem as INT,
                    ((*infoPtr).uView == LV_VIEW_LIST as DWORD) as ::core::ffi::c_int,
                );
            } else if (*infoPtr).nFocusedItem == (*lpLVItem).iItem {
                (*infoPtr).nFocusedItem = -(1 as INT);
            }
        }
    }

    /* if we're inserting the item, we're done */
    if isNew != 0 {
        return TRUE as BOOL;
    }

    /* send LVN_ITEMCHANGED notification */
    if (*lpLVItem).mask & LVIF_PARAM as UINT != 0 {
        nmlv.lParam = (*lpLVItem).lParam;
    }
    if (*infoPtr).notify_mask & NOTIFY_MASK_ITEM_CHANGE as DWORD != 0 {
        notify_listview(infoPtr, LVN_ITEMCHANGED as INT, &raw mut nmlv);
    }

    return TRUE as BOOL;
}

unsafe extern "C" fn set_sub_item(infoPtr: *const LISTVIEW_INFO, lpLVItem: *const LVITEMW, isW: BOOL, bChanged: *mut BOOL) -> BOOL {
    let mut hdpaSubItems: HDPA = core::ptr::null_mut();
    let mut lpSubItem: *mut SUBITEM_INFO = core::ptr::null_mut();

    /* we do not support subitems for virtual listviews */
    if ((*infoPtr).dwStyle & (LVS_OWNERDATA as DWORD)) != 0 { return FALSE; }

    /* set subitem only if column is present */
    if (*lpLVItem).iSubItem >= DPA_GetPtrCount((*infoPtr).hdpaColumns) { return FALSE; }

    /* First do some sanity checks */
    /* The LVIF_STATE flag is valid for subitems, but does not appear to be
       particularly useful. We currently do not actually do anything with
       the flag on subitems.
    */
    if (((*lpLVItem).mask & !(LVIF_TEXT as UINT | LVIF_IMAGE as UINT | LVIF_STATE as UINT | LVIF_DI_SETITEM as UINT))) != 0 { return FALSE; }
    if ((*lpLVItem).mask & (LVIF_TEXT as UINT | LVIF_IMAGE as UINT | LVIF_STATE as UINT)) == 0 { return TRUE; }

    /* get the subitem structure, and create it if not there */
    hdpaSubItems = (DPA_GetPtr((*infoPtr).hdpaItems, (*lpLVItem).iItem)).cast();
    assert!((!hdpaSubItems.is_null()) as i32);

    lpSubItem = LISTVIEW_GetSubItemPtr(hdpaSubItems, (*lpLVItem).iSubItem);
    if lpSubItem.is_null()
    {
        let mut tmpSubItem: *mut SUBITEM_INFO = core::ptr::null_mut();
        let mut i: INT = 0;

        lpSubItem = Alloc(core::mem::size_of::<SUBITEM_INFO>() as DWORD) as *mut SUBITEM_INFO;
        if lpSubItem.is_null() { return FALSE; }
        /* we could binary search here, if need be...*/
        i = 1 as INT;
        while i < DPA_GetPtrCount(hdpaSubItems)
        {
            tmpSubItem = DPA_GetPtr(hdpaSubItems, i) as *mut SUBITEM_INFO;
            if (*tmpSubItem).iSubItem > (*lpLVItem).iSubItem { break; }
            i += 1;
        }
        if DPA_InsertPtr(hdpaSubItems, i, lpSubItem as *mut core::ffi::c_void) == -(1 as INT)
        {
            Free(lpSubItem as LPVOID);
            return FALSE;
        }
        (*lpSubItem).iSubItem = (*lpLVItem).iSubItem;
        (*lpSubItem).hdr.iImage = I_IMAGECALLBACK as INT;
        *bChanged = TRUE;
    }

    if ((*lpLVItem).mask & (LVIF_IMAGE as UINT)) != 0 && ((*lpSubItem).hdr.iImage != (*lpLVItem).iImage)
    {
        (*lpSubItem).hdr.iImage = (*lpLVItem).iImage;
        *bChanged = TRUE;
    }

    if ((*lpLVItem).mask & (LVIF_TEXT as UINT)) != 0 && textcmpWT((*lpSubItem).hdr.pszText as LPCWSTR, (*lpLVItem).pszText as LPCWSTR, isW) != 0
    {
        textsetptrT(core::ptr::addr_of_mut!((*lpSubItem).hdr.pszText), (*lpLVItem).pszText as LPCWSTR, isW);
        *bChanged = TRUE;
    }

    return TRUE;
}

unsafe extern "C" fn LISTVIEW_SetItemT(infoPtr: *mut LISTVIEW_INFO, lpLVItem: *mut LVITEMW, isW: BOOL) -> BOOL {
    let hwndSelf: HWND = (*infoPtr).hwndSelf;
    let mut pszText: LPWSTR = core::ptr::null_mut();
    let mut bResult: BOOL = 0;
    let mut bChanged: BOOL = FALSE;
    let mut oldItemArea: RECT = core::mem::zeroed();

    if lpLVItem.is_null() || (*lpLVItem).iItem < 0 || (*lpLVItem).iItem >= (*infoPtr).nItemCount {
        return FALSE;
    }

    /* Store old item area */
    LISTVIEW_GetItemBox(infoPtr, (*lpLVItem).iItem as INT, core::ptr::addr_of_mut!(oldItemArea));

    /* For efficiency, we transform the lpLVItem->pszText to Unicode here */
    if ((*lpLVItem).mask & (LVIF_TEXT as UINT)) != 0 && is_text((*lpLVItem).pszText as LPCWSTR) != 0 {
        pszText = (*lpLVItem).pszText;
        (*lpLVItem).pszText = textdupTtoW((*lpLVItem).pszText as LPCWSTR, isW);
    }

    /* actually set the fields */
    if is_assignable_item(lpLVItem, (*infoPtr).dwStyle as LONG) == 0 {
        return FALSE;
    }

    if (*lpLVItem).iSubItem != 0 {
        bResult = set_sub_item(infoPtr, lpLVItem, TRUE, core::ptr::addr_of_mut!(bChanged));
    } else {
        bResult = set_main_item(infoPtr, lpLVItem, FALSE, TRUE, core::ptr::addr_of_mut!(bChanged));
    }
    if IsWindow(hwndSelf) == 0 {
        return FALSE;
    }

    /* redraw item, if necessary */
    if bChanged != 0 && (*infoPtr).bIsDrawing == 0 {
        /* this little optimization eliminates some nasty flicker */
        if (*infoPtr).uView == (LV_VIEW_DETAILS as DWORD)
            && ((*infoPtr).dwStyle & (LVS_OWNERDRAWFIXED as DWORD)) == 0
            && ((*infoPtr).dwLvExStyle & (LVS_EX_FULLROWSELECT as DWORD)) == 0
            && (*lpLVItem).iSubItem > 0
            && (*lpLVItem).iSubItem <= DPA_GetPtrCount((*infoPtr).hdpaColumns)
        {
            LISTVIEW_InvalidateSubItem(infoPtr, (*lpLVItem).iItem as INT, (*lpLVItem).iSubItem as INT);
        } else {
            LISTVIEW_InvalidateRect(infoPtr, core::ptr::addr_of!(oldItemArea));
            LISTVIEW_InvalidateItem(infoPtr, (*lpLVItem).iItem as INT);
        }
    }
    /* restore text */
    if !pszText.is_null() {
        textfreeT((*lpLVItem).pszText, isW);
        (*lpLVItem).pszText = pszText;
    }

    return bResult;
}

unsafe extern "C" fn LISTVIEW_GetTopIndex(infoPtr: *const LISTVIEW_INFO) -> INT {
    let mut nItem: INT = 0 as INT;
    let mut scrollInfo: SCROLLINFO = core::mem::zeroed();

    scrollInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    scrollInfo.fMask = SIF_POS as UINT;

    if (*infoPtr).uView == LV_VIEW_LIST as DWORD {
        if GetScrollInfo((*infoPtr).hwndSelf, SB_HORZ as INT, core::ptr::addr_of_mut!(scrollInfo)) != 0 {
            nItem = scrollInfo.nPos as INT * LISTVIEW_GetCountPerColumn(infoPtr);
        }
    } else if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
        if GetScrollInfo((*infoPtr).hwndSelf, SB_VERT as INT, core::ptr::addr_of_mut!(scrollInfo)) != 0 {
            nItem = scrollInfo.nPos as INT;
        }
    } else {
        if GetScrollInfo((*infoPtr).hwndSelf, SB_VERT as INT, core::ptr::addr_of_mut!(scrollInfo)) != 0 {
            nItem = LISTVIEW_GetCountPerRow(infoPtr) * (scrollInfo.nPos as INT / (*infoPtr).nItemHeight);
        }
    }

    return nItem;
}

unsafe extern "C" fn LISTVIEW_FillBkgnd(infoPtr: *const LISTVIEW_INFO, hdc: HDC, lprcBox: *const RECT) -> BOOL {
    if (*infoPtr).hBkBrush.is_null() {
        return FALSE;
    }

    FillRect(hdc, lprcBox, (*infoPtr).hBkBrush)
}

unsafe extern "C" fn LISTVIEW_DrawItemPart(infoPtr: *mut LISTVIEW_INFO, item: *mut LVITEMW, nmlvcd: *const NMLVCUSTOMDRAW, pos: *const POINT) {
    let mut rcSelect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcLabel: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcBox: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcStateIcon: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcIcon: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut background: *const RECT = ::core::ptr::null::<RECT>();
    let mut himl: HIMAGELIST = ::core::ptr::null_mut();
    let mut format: UINT = 0;
    let mut focus: *mut RECT = ::core::ptr::null_mut::<RECT>();

    /* now check if we need to update the focus rectangle */
    focus = if (*infoPtr).bFocus != 0 && ((*item).state & (LVIS_FOCUSED as UINT)) != 0 {
        &raw mut (*infoPtr).rcFocus
    } else {
        0 as *mut RECT
    };
    if focus.is_null() {
        (*item).state &= !(LVIS_FOCUSED as UINT);
    }

    LISTVIEW_GetItemMetrics(infoPtr, item, &raw mut rcBox, &raw mut rcSelect, &raw mut rcIcon, &raw mut rcStateIcon, &raw mut rcLabel);
    OffsetRect(&raw mut rcBox, (*pos).x, (*pos).y);
    OffsetRect(&raw mut rcSelect, (*pos).x, (*pos).y);
    OffsetRect(&raw mut rcIcon, (*pos).x, (*pos).y);
    OffsetRect(&raw mut rcStateIcon, (*pos).x, (*pos).y);
    OffsetRect(&raw mut rcLabel, (*pos).x, (*pos).y);

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_DrawItemPart\0" as *const u8 as *const ::core::ffi::c_char,
            4701 as ::core::ffi::c_int,
            b"%d: box=%s, select=%s, icon=%s. label=%s\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*item).iSubItem,
            wine_dbgstr_rect(&raw const rcBox),
            wine_dbgstr_rect(&raw const rcSelect),
            wine_dbgstr_rect(&raw const rcIcon),
            wine_dbgstr_rect(&raw const rcLabel),
        );
    }

    /* FIXME: temporary hack */
    rcSelect.left = rcLabel.left;

    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD && (*item).iSubItem == 0 as INT {
        if ((*infoPtr).dwLvExStyle & (LVS_EX_FULLROWSELECT as DWORD)) == 0 {
            OffsetRect(
                &raw mut rcSelect,
                (*LISTVIEW_GetColumnInfo(infoPtr, 0 as INT)).rcHeader.left,
                0 as INT,
            );
        }
        OffsetRect(
            &raw mut rcIcon,
            (*LISTVIEW_GetColumnInfo(infoPtr, 0 as INT)).rcHeader.left,
            0 as INT,
        );
        OffsetRect(
            &raw mut rcStateIcon,
            (*LISTVIEW_GetColumnInfo(infoPtr, 0 as INT)).rcHeader.left,
            0 as INT,
        );
        OffsetRect(
            &raw mut rcLabel,
            (*LISTVIEW_GetColumnInfo(infoPtr, 0 as INT)).rcHeader.left,
            0 as INT,
        );
    }

    /* in icon mode, the label rect is really what we want to draw the
     * background for */
    /* in detail mode, we want to paint background for label rect when
     * item is not selected or listview has full row select; otherwise paint
     * background for text only */
    if (*infoPtr).uView == LV_VIEW_ICON as DWORD
        || ((*infoPtr).uView == LV_VIEW_DETAILS as DWORD
            && (((*item).state & (LVIS_SELECTED as UINT)) == 0
                || ((*infoPtr).dwLvExStyle & (LVS_EX_FULLROWSELECT as DWORD)) != 0))
    {
        background = &raw const rcLabel;
    } else {
        background = &raw const rcSelect;
    }

    if (*nmlvcd).clrTextBk != CLR_NONE as _ {
        ExtTextOutW(
            (*nmlvcd).nmcd.hdc,
            (*background).left,
            (*background).top,
            ETO_OPAQUE as UINT,
            background,
            ::core::ptr::null::<WCHAR>(),
            0 as UINT,
            ::core::ptr::null::<INT>(),
        );
    }

    if ((*item).state & (LVIS_FOCUSED as UINT)) != 0 {
        if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
            if ((*infoPtr).dwLvExStyle & (LVS_EX_FULLROWSELECT as DWORD)) != 0 {
                /* we have to update left focus bound too if item isn't in leftmost column
                   and reduce right box bound */
                if DPA_GetPtrCount((*infoPtr).hdpaColumns) > 0 as INT {
                    let mut leftmost: INT = 0;

                    leftmost = SendMessageW(
                        (*infoPtr).hwndHeader,
                        HDM_ORDERTOINDEX as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) as INT;
                    if leftmost != 0 {
                        let mut Originx: INT = (*pos).x
                            - (*LISTVIEW_GetColumnInfo(infoPtr, leftmost)).rcHeader.left;
                        let mut rightmost: INT = SendMessageW(
                            (*infoPtr).hwndHeader,
                            HDM_ORDERTOINDEX as UINT,
                            (DPA_GetPtrCount((*infoPtr).hdpaColumns) - 1 as INT) as WPARAM,
                            0 as LPARAM,
                        ) as INT;

                        rcBox.right = (*LISTVIEW_GetColumnInfo(infoPtr, rightmost)).rcHeader.right + Originx;
                        rcSelect.left = (*LISTVIEW_GetColumnInfo(infoPtr, leftmost)).rcHeader.left + Originx;
                    }
                }
                rcSelect.right = rcBox.right;
            }
            (*infoPtr).rcFocus = rcSelect;
        } else {
            (*infoPtr).rcFocus = rcLabel;
        }
    }

    /* state icons */
    if !(*infoPtr).himlState.is_null()
        && STATEIMAGEINDEX!((*item).state) != 0
        && (*item).iSubItem == 0 as INT
    {
        let mut stateimage: UINT = STATEIMAGEINDEX!((*item).state);
        if stateimage != 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/listview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"LISTVIEW_DrawItemPart\0" as *const u8 as *const ::core::ffi::c_char,
                    4766 as ::core::ffi::c_int,
                    b"stateimage=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
                    stateimage,
                );
            }
            ImageList_Draw(
                (*infoPtr).himlState,
                stateimage.wrapping_sub(1 as UINT) as INT,
                (*nmlvcd).nmcd.hdc,
                rcStateIcon.left,
                rcStateIcon.top,
                ILD_NORMAL as UINT,
            );
        }
    }

    /* item icons */
    himl = if (*infoPtr).uView == LV_VIEW_ICON as DWORD {
        (*infoPtr).himlNormal
    } else {
        (*infoPtr).himlSmall
    };
    if !himl.is_null()
        && (*item).iImage >= 0 as INT
        && IsRectEmpty(&raw mut rcIcon as LPCRECT) == 0
    {
        let mut style: UINT = 0;

        if ((*item).state & ((LVIS_CUT as UINT)
            | (if (*infoPtr).bFocus != 0 {
                LVIS_SELECTED as UINT
            } else {
                0 as UINT
            })))
            != 0
        {
            style = ILD_SELECTED as UINT;
        } else {
            style = ILD_NORMAL as UINT;
        }

        ImageList_DrawEx(
            himl,
            (*item).iImage,
            (*nmlvcd).nmcd.hdc,
            rcIcon.left,
            rcIcon.top,
            rcIcon.right - rcIcon.left,
            rcIcon.bottom - rcIcon.top,
            (*infoPtr).clrBk,
            if ((*item).state & (LVIS_CUT as UINT)) != 0 {
                RGB!(
                    255 as BYTE,
                    255 as BYTE,
                    255 as BYTE,
                )
            } else {
                CLR_DEFAULT as COLORREF
            },
            style | ((*item).state & (LVIS_OVERLAYMASK as UINT)),
        );
    }

    /* Don't bother painting item being edited */
    if !(*infoPtr).hwndEdit.is_null()
        && (*item).iItem == (*infoPtr).nEditLabelItem
        && (*item).iSubItem == 0 as INT
    {
        return;
    }

    /* figure out the text drawing flags */
    format = (if (*infoPtr).uView == LV_VIEW_ICON as DWORD {
        if !focus.is_null() {
            LV_FL_DT_FLAGS as UINT
        } else {
            LV_ML_DT_FLAGS as UINT
        }
    } else {
        LV_SL_DT_FLAGS as UINT
    }) as UINT;
    if (*infoPtr).uView == LV_VIEW_ICON as DWORD {
        format = if !focus.is_null() {
            LV_FL_DT_FLAGS as UINT
        } else {
            LV_ML_DT_FLAGS as UINT
        };
    } else if (*item).iSubItem != 0 {
        match ((*LISTVIEW_GetColumnInfo(infoPtr, (*item).iSubItem)).fmt as INT
            & (LVCFMT_JUSTIFYMASK as INT)) as u32
        {
            LVCFMT_RIGHT => {
                format |= DT_RIGHT as UINT;
            }
            LVCFMT_CENTER => {
                format |= DT_CENTER as UINT;
            }
            _ => {
                format |= DT_LEFT as UINT;
            }
        }
    }
    if (format & (DT_RIGHT as UINT | DT_CENTER as UINT)) == 0 {
        if !himl.is_null()
            && (*item).iImage >= 0 as INT
            && IsRectEmpty(&raw mut rcIcon as LPCRECT) == 0
        {
            rcLabel.left += IMAGE_PADDING as LONG;
        } else {
            rcLabel.left += LABEL_HOR_PADDING as LONG;
        }
    } else if (format & (DT_RIGHT as UINT)) != 0 {
        rcLabel.right -= LABEL_HOR_PADDING as LONG;
    }

    /* for GRIDLINES reduce the bottom so the text formats correctly */
    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD
        && ((*infoPtr).dwLvExStyle & (LVS_EX_GRIDLINES as DWORD)) != 0
    {
        rcLabel.bottom -= 1;
    }

    if (((*item).state & (LVIS_SELECTED as UINT)) == 0 || (*infoPtr).bFocus == 0)
        && ((*infoPtr).dwLvExStyle & (LVS_EX_TRANSPARENTSHADOWTEXT as DWORD)) != 0
    {
        DrawShadowText(
            (*nmlvcd).nmcd.hdc,
            (*item).pszText as LPCWSTR,
            (-(1 as INT)) as UINT,
            &raw mut rcLabel,
            format as DWORD,
            RGB!(
                255 as BYTE,
                255 as BYTE,
                255 as BYTE,
            ),
            RGB!(
                0 as BYTE,
                0 as BYTE,
                0 as BYTE,
            ),
            2 as INT,
            2 as INT,
        );
    } else {
        DrawTextW(
            (*nmlvcd).nmcd.hdc,
            (*item).pszText as LPCWSTR,
            -(1 as INT),
            &raw mut rcLabel,
            format,
        );
    }
}

unsafe extern "C" fn LISTVIEW_DrawItem(
    infoPtr: *mut LISTVIEW_INFO,
    hdc: HDC,
    nItem: INT,
    subitems: *mut ITERATOR,
    mut pos: POINT,
    cdmode: DWORD,
) -> BOOL {
    let mut szDispText: [WCHAR; (DISP_TEXT_SIZE) as usize] = [0 as WCHAR; (DISP_TEXT_SIZE) as usize];
    static mut callbackW: [WCHAR; 11] = [
    0x0028u16, 0x0063, 0x0061, 0x006C, 0x006C, 0x0062, 0x0061, 0x0063, 0x006B, 0x0029,
    0x0000,
];
    let mut cdsubitemmode: DWORD = CDRF_DODEFAULT as DWORD;
    let mut focus: *mut RECT;
    let mut rcBox: RECT = core::mem::zeroed();
    let mut nmlvcd: NMLVCUSTOMDRAW = core::mem::zeroed();
    let mut lvItem: LVITEMW = core::mem::zeroed();

    /* get information needed for drawing the item */
    lvItem.mask = (LVIF_TEXT | LVIF_IMAGE | LVIF_PARAM | LVIF_STATE) as UINT;
    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
        lvItem.mask |= LVIF_INDENT as UINT;
    }
    lvItem.stateMask = (LVIS_SELECTED | LVIS_FOCUSED | LVIS_STATEIMAGEMASK | LVIS_CUT | LVIS_OVERLAYMASK) as UINT;
    lvItem.iItem = nItem as ::core::ffi::c_int;
    lvItem.iSubItem = 0 as ::core::ffi::c_int;
    lvItem.state = 0 as UINT;
    lvItem.lParam = 0 as LPARAM;
    lvItem.cchTextMax = DISP_TEXT_SIZE;
    lvItem.pszText = &raw mut szDispText as *mut WCHAR as LPWSTR;
    if LISTVIEW_GetItemW(infoPtr, &raw mut lvItem) == 0 {
        return FALSE;
    }
    if lvItem.pszText == LPSTR_TEXTCALLBACKW {
        lvItem.pszText = &raw mut callbackW as *mut WCHAR as LPWSTR;
    }

    /* now check if we need to update the focus rectangle */
    focus = if (*infoPtr).bFocus != 0 && (lvItem.state & LVIS_FOCUSED as UINT) != 0 {
        &raw mut (*infoPtr).rcFocus
    } else {
        core::ptr::null_mut()
    };
    if focus.is_null() {
        lvItem.state &= !(LVIS_FOCUSED as UINT);
    }

    LISTVIEW_GetItemMetrics(infoPtr, &raw mut lvItem, &raw mut rcBox, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
    OffsetRect(&raw mut rcBox, pos.x as ::core::ffi::c_int, pos.y as ::core::ffi::c_int);

    /* fill in the custom draw structure */
    customdraw_fill(&raw mut nmlvcd, infoPtr, hdc, &raw mut rcBox, &raw mut lvItem);
    if (cdmode & CDRF_NOTIFYITEMDRAW as DWORD) != 0 {
        cdsubitemmode = notify_customdraw(infoPtr, CDDS_ITEMPREPAINT as DWORD, &raw mut nmlvcd);
    }

    'postpaint: {
        if (cdsubitemmode & CDRF_SKIPDEFAULT as DWORD) != 0 {
            break 'postpaint;
        }

        if !subitems.is_null() {
            while iterator_next(subitems) != 0 {
                let mut subitemstage: DWORD = CDRF_DODEFAULT as DWORD;
                let mut temp_nmlvcd: NMLVCUSTOMDRAW = core::mem::zeroed();

                /* We need to query for each subitem, item's data (subitem == 0) is already here at this point */
                if (*subitems).nItem != 0 {
                    lvItem.mask = (LVIF_TEXT | LVIF_IMAGE | LVIF_PARAM | LVIF_INDENT) as UINT;
                    lvItem.stateMask = (LVIS_SELECTED | LVIS_FOCUSED | LVIS_STATEIMAGEMASK | LVIS_CUT | LVIS_OVERLAYMASK) as UINT;
                    lvItem.iItem = nItem as ::core::ffi::c_int;
                    lvItem.iSubItem = (*subitems).nItem as ::core::ffi::c_int;
                    lvItem.state = 0 as UINT;
                    lvItem.lParam = 0 as LPARAM;
                    lvItem.cchTextMax = DISP_TEXT_SIZE;
                    lvItem.pszText = &raw mut szDispText as *mut WCHAR as LPWSTR;
                    if LISTVIEW_GetItemW(infoPtr, &raw mut lvItem) == 0 {
                        return FALSE;
                    }
                    if (*infoPtr).dwLvExStyle & LVS_EX_FULLROWSELECT as DWORD != 0 {
                        lvItem.state = LISTVIEW_GetItemState(infoPtr, nItem, LVIS_SELECTED as UINT);
                    }
                    if lvItem.pszText == LPSTR_TEXTCALLBACKW {
                        lvItem.pszText = &raw mut callbackW as *mut WCHAR as LPWSTR;
                    }

                    /* update custom draw data */
                    LISTVIEW_GetItemMetrics(infoPtr, &raw mut lvItem, &raw mut nmlvcd.nmcd.rc, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());
                    OffsetRect(&raw mut nmlvcd.nmcd.rc, pos.x as ::core::ffi::c_int, pos.y as ::core::ffi::c_int);
                    nmlvcd.iSubItem = (*subitems).nItem as ::core::ffi::c_int;
                }

                if (cdsubitemmode & CDRF_NOTIFYSUBITEMDRAW as DWORD) != 0 {
                    subitemstage = notify_customdraw(infoPtr, (CDDS_SUBITEM | CDDS_ITEMPREPAINT) as DWORD, &raw mut nmlvcd);
                }

                /*
                 * A selection should neither affect the colors in the post paint notification nor
                 * affect the colors of the next drawn subitem. Copy the structure to prevent this.
                 */
                temp_nmlvcd = nmlvcd;
                prepaint_setup(infoPtr, hdc, &raw mut temp_nmlvcd, (*subitems).nItem as BOOL);

                if (subitemstage & CDRF_SKIPDEFAULT as DWORD) == 0 {
                    LISTVIEW_DrawItemPart(infoPtr, &raw mut lvItem, &raw mut temp_nmlvcd, &raw mut pos);
                }

                if (subitemstage & CDRF_NOTIFYPOSTPAINT as DWORD) != 0 {
                    subitemstage = notify_customdraw(infoPtr, (CDDS_SUBITEM | CDDS_ITEMPOSTPAINT) as DWORD, &raw mut nmlvcd);
                }
            }
        } else {
            prepaint_setup(infoPtr, hdc, &raw mut nmlvcd, FALSE);
            LISTVIEW_DrawItemPart(infoPtr, &raw mut lvItem, &raw mut nmlvcd, &raw mut pos);
        }
    }

    /* postpaint: */
    if (cdsubitemmode & CDRF_NOTIFYPOSTPAINT as DWORD) != 0 {
        nmlvcd.iSubItem = 0 as ::core::ffi::c_int;
        notify_customdraw(infoPtr, CDDS_ITEMPOSTPAINT as DWORD, &raw mut nmlvcd);
    }

    return TRUE;
}

unsafe extern "C" fn LISTVIEW_RefreshOwnerDraw(
    mut infoPtr: *const LISTVIEW_INFO,
    mut i: *mut ITERATOR,
    mut hdc: HDC,
    mut cdmode: DWORD,
) {
    let mut uID: UINT = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID) as UINT;
    let mut cditemmode: DWORD = CDRF_DODEFAULT as DWORD;
    let mut nmlvcd: NMLVCUSTOMDRAW = tagNMLVCUSTOMDRAW {
        nmcd: tagNMCUSTOMDRAWINFO {
            hdr: tagNMHDR {
                hwndFrom: ::core::ptr::null_mut::<HWND__>(),
                idFrom: 0,
                code: 0,
            },
            dwDrawStage: 0,
            hdc: ::core::ptr::null_mut::<HDC__>(),
            rc: tagRECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            dwItemSpec: 0,
            uItemState: 0,
            lItemlParam: 0,
        },
        clrText: 0,
        clrTextBk: 0,
        iSubItem: 0,
        dwItemType: 0,
        clrFace: 0,
        iIconEffect: 0,
        iIconPhase: 0,
        iPartId: 0,
        iStateId: 0,
        rcText: tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        uAlign: 0,
    };
    let mut Origin: POINT = tagPOINT { x: 0, y: 0 };
    let mut Position: POINT = tagPOINT { x: 0, y: 0 };
    let mut dis: DRAWITEMSTRUCT = tagDRAWITEMSTRUCT {
        CtlType: 0,
        CtlID: 0,
        itemID: 0,
        itemAction: 0,
        itemState: 0,
        hwndItem: ::core::ptr::null_mut::<HWND__>(),
        hDC: ::core::ptr::null_mut::<HDC__>(),
        rcItem: tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        itemData: 0,
    };
    let mut item: LVITEMW = tagLVITEMW {
        mask: 0,
        iItem: 0,
        iSubItem: 0,
        state: 0,
        stateMask: 0,
        pszText: ::core::ptr::null_mut::<WCHAR>(),
        cchTextMax: 0,
        iImage: 0,
        lParam: 0,
        iIndent: 0,
        iGroupId: 0,
        cColumns: 0,
        puColumns: ::core::ptr::null_mut::<::core::ffi::c_uint>(),
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
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_RefreshOwnerDraw\0" as *const u8 as *const ::core::ffi::c_char,
            4981 as ::core::ffi::c_int,
            b"()\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    ZeroMemory!(
        &raw mut dis as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<DRAWITEMSTRUCT>() as usize,
    );
    LISTVIEW_GetOrigin(infoPtr, &raw mut Origin);
    while iterator_next(i) != 0 {
        item.iItem = (*i).nItem as ::core::ffi::c_int;
        item.iSubItem = 0 as ::core::ffi::c_int;
        item.mask = (LVIF_PARAM | LVIF_STATE) as UINT;
        item.stateMask = (LVIS_SELECTED | LVIS_FOCUSED) as UINT;
        if LISTVIEW_GetItemW(infoPtr, &raw mut item) == 0 {
            continue;
        }
        dis.CtlType = ODT_LISTVIEW as UINT;
        dis.CtlID = uID;
        dis.itemID = item.iItem as UINT;
        dis.itemAction = ODA_DRAWENTIRE as UINT;
        dis.itemState = 0 as UINT;
        if (item.state & (LVIS_SELECTED as UINT)) != 0 {
            dis.itemState |= ODS_SELECTED as UINT;
        }
        if (*infoPtr).bFocus != 0 && (item.state & (LVIS_FOCUSED as UINT)) != 0 {
            dis.itemState |= ODS_FOCUS as UINT;
        }
        dis.hwndItem = (*infoPtr).hwndSelf;
        dis.hDC = hdc;
        LISTVIEW_GetItemOrigin(infoPtr, dis.itemID as INT, &raw mut Position);
        dis.rcItem.left = Position.x + Origin.x;
        dis.rcItem.right = (dis.rcItem.left as INT + (*infoPtr).nItemWidth) as LONG;
        dis.rcItem.top = Position.y + Origin.y;
        dis.rcItem.bottom = (dis.rcItem.top as INT + (*infoPtr).nItemHeight) as LONG;
        dis.itemData = item.lParam as ULONG_PTR;
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/listview.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"LISTVIEW_RefreshOwnerDraw\0" as *const u8
                    as *const ::core::ffi::c_char,
                5013 as ::core::ffi::c_int,
                b"item=%s, rcItem=%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                debuglvitem_t(&raw mut item, 1 as BOOL),
                wine_dbgstr_rect(&raw mut dis.rcItem),
            );
        }
        customdraw_fill(
            &raw mut nmlvcd,
            infoPtr,
            hdc,
            &raw mut dis.rcItem,
            &raw mut item,
        );
        if cdmode & (CDRF_NOTIFYITEMDRAW as DWORD) != 0 {
            cditemmode = notify_customdraw(
                infoPtr,
                CDDS_PREPAINT as DWORD,
                &raw mut nmlvcd,
            );
        }
        if (cditemmode & (CDRF_SKIPDEFAULT as DWORD)) == 0 {
            prepaint_setup(
                infoPtr,
                hdc,
                &raw mut nmlvcd,
                FALSE as BOOL,
            );
            SendMessageW(
                (*infoPtr).hwndNotify,
                WM_DRAWITEM as UINT,
                dis.CtlID as WPARAM,
                &raw mut dis as LPARAM,
            );
        }
        if cditemmode & (CDRF_NOTIFYPOSTPAINT as DWORD) != 0 {
            notify_postpaint(infoPtr, &raw mut nmlvcd);
        }
    }
}

unsafe extern "C" fn LISTVIEW_RefreshReport(infoPtr: *mut LISTVIEW_INFO, i: *mut ITERATOR, hdc: HDC, cdmode: DWORD) {
    let mut rgntype: INT = 0;
    let mut rcClip: RECT = core::mem::zeroed();
    let mut rcItem: RECT = core::mem::zeroed();
    let mut Origin: POINT = core::mem::zeroed();
    let mut colRanges: RANGES = core::ptr::null_mut();
    let mut col: INT = 0;
    let mut j: ITERATOR = core::mem::zeroed();

    /* figure out what to draw */
    rgntype = GetClipBox(hdc, core::ptr::addr_of_mut!(rcClip)) as INT;
    if rgntype == (NULLREGION as i32) {
        return;
    }

    /* Get scroll info once before loop */
    LISTVIEW_GetOrigin(infoPtr, core::ptr::addr_of_mut!(Origin));

    colRanges = ranges_create(DPA_GetPtrCount((*infoPtr).hdpaColumns));

    /* narrow down the columns we need to paint */
    col = 0;
    while col < DPA_GetPtrCount((*infoPtr).hdpaColumns) {
        let mut index: INT = SendMessageW(
            (*infoPtr).hwndHeader,
            HDM_ORDERTOINDEX as UINT,
            col as WPARAM,
            0 as LPARAM,
        ) as INT;

        LISTVIEW_GetHeaderRect(infoPtr, index, core::ptr::addr_of_mut!(rcItem));
        if (rcItem.right + Origin.x >= rcClip.left) && (rcItem.left + Origin.x < rcClip.right) {
            ranges_additem(colRanges, index);
        }
        col += 1;
    }
    iterator_rangesitems(core::ptr::addr_of_mut!(j), colRanges);

    /* in full row select, we _have_ to draw the main item */
    if ((*infoPtr).dwLvExStyle & (LVS_EX_FULLROWSELECT as DWORD)) != 0 {
        j.nSpecial = 0 as INT;
    }

    /* iterate through the invalidated rows */
    while iterator_next(i) != 0 {
        let mut subitems: RANGES = core::ptr::null_mut();
        let mut Position: POINT = core::mem::zeroed();
        let mut k: ITERATOR = core::mem::zeroed();

        SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ);
        LISTVIEW_GetItemOrigin(infoPtr, (*i).nItem, core::ptr::addr_of_mut!(Position));
        Position.x = Origin.x;
        Position.y += Origin.y;

        subitems = ranges_create(DPA_GetPtrCount((*infoPtr).hdpaColumns));

        /* iterate through the invalidated columns */
        while iterator_next(core::ptr::addr_of_mut!(j)) != 0 {
            LISTVIEW_GetHeaderRect(infoPtr, j.nItem, core::ptr::addr_of_mut!(rcItem));

            if rgntype == (COMPLEXREGION as i32)
                && !(((*infoPtr).dwLvExStyle & (LVS_EX_FULLROWSELECT as DWORD)) != 0
                    && j.nItem == 0)
            {
                rcItem.top = 0 as LONG;
                rcItem.bottom = (*infoPtr).nItemHeight as LONG;
                OffsetRect(core::ptr::addr_of_mut!(rcItem), Origin.x, Position.y);
                if RectVisible(hdc, core::ptr::addr_of_mut!(rcItem)) == 0 {
                    continue;
                }
            }

            ranges_additem(subitems, j.nItem);
        }

        iterator_rangesitems(core::ptr::addr_of_mut!(k), subitems);
        LISTVIEW_DrawItem(infoPtr, hdc, (*i).nItem, core::ptr::addr_of_mut!(k), Position, cdmode);
        iterator_destroy(core::ptr::addr_of_mut!(k));
    }
    iterator_destroy(core::ptr::addr_of_mut!(j));
}

unsafe extern "C" fn LISTVIEW_RefreshReportGrid(infoPtr: *mut LISTVIEW_INFO, hdc: HDC) {
    let mut rgntype: INT = 0;
    let mut y: INT = 0;
    let mut itemheight: INT = 0;
    let mut col: INT = 0;
    let mut index: INT = 0;
    let mut hPen: HPEN = core::ptr::null_mut();
    let mut hOldPen: HPEN = core::ptr::null_mut();
    let mut rcClip: RECT = core::mem::zeroed();
    let mut rcItem: RECT = core::mem::zeroed();
    let mut Origin: POINT = core::mem::zeroed();
    let mut colRanges: RANGES = core::ptr::null_mut();
    let mut j: ITERATOR = core::mem::zeroed();
    let mut rmost: BOOL = FALSE;

    /* figure out what to draw */
    rgntype = GetClipBox(hdc, core::ptr::addr_of_mut!(rcClip));
    if rgntype == (NULLREGION as i32) {
        return;
    }

    /* Get scroll info once before loop */
    LISTVIEW_GetOrigin(infoPtr, core::ptr::addr_of_mut!(Origin));

    colRanges = ranges_create(DPA_GetPtrCount((*infoPtr).hdpaColumns));

    /* narrow down the columns we need to paint */
    col = 0;
    while col < DPA_GetPtrCount((*infoPtr).hdpaColumns) {
        index = SendMessageW(
            (*infoPtr).hwndHeader,
            HDM_ORDERTOINDEX,
            col as WPARAM,
            0 as LPARAM,
        ) as INT;

        LISTVIEW_GetHeaderRect(infoPtr, index, core::ptr::addr_of_mut!(rcItem));
        if (rcItem.right + Origin.x >= rcClip.left) && (rcItem.left + Origin.x < rcClip.right) {
            ranges_additem(colRanges, index);
        }
        col += 1;
    }

    /* is right most vertical line visible? */
    if DPA_GetPtrCount((*infoPtr).hdpaColumns) > 0 {
        index = SendMessageW(
            (*infoPtr).hwndHeader,
            HDM_ORDERTOINDEX,
            (DPA_GetPtrCount((*infoPtr).hdpaColumns) - 1) as WPARAM,
            0 as LPARAM,
        ) as INT;
        LISTVIEW_GetHeaderRect(infoPtr, index, core::ptr::addr_of_mut!(rcItem));
        rmost = (rcItem.right + Origin.x < rcClip.right) as BOOL;
    }

    hPen = CreatePen(
        (PS_SOLID as i32),
        1,
        comctl32_color.clr3dFace,
    );
    if !hPen.is_null() {
        hOldPen = SelectObject(hdc, hPen as HGDIOBJ) as HPEN;

        /* draw the vertical lines for the columns */
        iterator_rangesitems(core::ptr::addr_of_mut!(j), colRanges);
        while iterator_next(core::ptr::addr_of_mut!(j)) != 0 {
            LISTVIEW_GetHeaderRect(infoPtr, j.nItem, core::ptr::addr_of_mut!(rcItem));
            if rcItem.left == 0 {
                continue; /* skip leftmost column */
            }
            rcItem.left += Origin.x;
            rcItem.right += Origin.x;
            rcItem.top = (*infoPtr).rcList.top;
            rcItem.bottom = (*infoPtr).rcList.bottom;

            MoveToEx(
                hdc,
                rcItem.left,
                rcItem.top,
                core::ptr::null_mut(),
            );
            LineTo(hdc, rcItem.left, rcItem.bottom);
        }
        iterator_destroy(core::ptr::addr_of_mut!(j));

        /* draw rightmost grid line if visible */
        if rmost != 0 {
            index = SendMessageW(
                (*infoPtr).hwndHeader,
                HDM_ORDERTOINDEX,
                (DPA_GetPtrCount((*infoPtr).hdpaColumns) - 1) as WPARAM,
                0 as LPARAM,
            ) as INT;
            LISTVIEW_GetHeaderRect(infoPtr, index, core::ptr::addr_of_mut!(rcItem));

            rcItem.right += Origin.x;

            MoveToEx(
                hdc,
                rcItem.right,
                (*infoPtr).rcList.top,
                core::ptr::null_mut(),
            );
            LineTo(hdc, rcItem.right, (*infoPtr).rcList.bottom);
        }

        /* draw the horizontal lines for the rows */
        itemheight = LISTVIEW_CalculateItemHeight(infoPtr);
        rcItem.left = (*infoPtr).rcList.left;
        rcItem.right = (*infoPtr).rcList.right;
        y = if Origin.y > 1 {
            Origin.y - 1
        } else {
            itemheight - 1 + Origin.y % itemheight
        };
        while y <= (*infoPtr).rcList.bottom {
            rcItem.top = y;
            rcItem.bottom = y;

            MoveToEx(
                hdc,
                rcItem.left,
                rcItem.top,
                core::ptr::null_mut(),
            );
            LineTo(hdc, rcItem.right, rcItem.top);
            y += itemheight;
        }

        SelectObject(hdc, hOldPen as HGDIOBJ);
        DeleteObject(hPen as HGDIOBJ);
    } else {
        ranges_destroy(colRanges);
    }
}

unsafe extern "C" fn LISTVIEW_RefreshList(infoPtr: *mut LISTVIEW_INFO, i: *mut ITERATOR, hdc: HDC, cdmode: DWORD) {
    let mut Origin: POINT = POINT { x: 0, y: 0 };
    let mut Position: POINT = POINT { x: 0, y: 0 };

    /* Get scroll info once before loop */
    LISTVIEW_GetOrigin(infoPtr, core::ptr::addr_of_mut!(Origin));

    while iterator_prev(i) != 0 {
        SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ);
        LISTVIEW_GetItemOrigin(infoPtr, (*i).nItem, core::ptr::addr_of_mut!(Position));
        Position.x += Origin.x;
        Position.y += Origin.y;

        LISTVIEW_DrawItem(infoPtr, hdc, (*i).nItem, core::ptr::null_mut::<ITERATOR>(), Position, cdmode);
    }
}

unsafe extern "C" fn LISTVIEW_Refresh(infoPtr: *mut LISTVIEW_INFO, mut hdc: HDC, prcErase: *const RECT) {
    let mut oldTextColor: COLORREF = 0;
    let mut oldBkColor: COLORREF = 0;
    let mut nmlvcd: NMLVCUSTOMDRAW = core::mem::zeroed();
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut cdmode: DWORD = 0;
    let mut oldBkMode: INT = 0;
    let mut rcClient: RECT = core::mem::zeroed();
    let mut i: ITERATOR = core::mem::zeroed();
    let hdcOrig: HDC = hdc;
    let mut hbmp: HBITMAP = core::ptr::null_mut();
    let mut range: RANGE = core::mem::zeroed();

    LISTVIEW_DUMP!(infoPtr);

    if ((*infoPtr).dwLvExStyle & LVS_EX_DOUBLEBUFFER) != 0 {
        hdc = CreateCompatibleDC(hdcOrig);
        if hdc.is_null() {
            return;
        }
        hbmp = CreateCompatibleBitmap(hdcOrig, (*infoPtr).rcList.right, (*infoPtr).rcList.bottom);
        if hbmp.is_null() {
            DeleteDC(hdc);
            return;
        }

        SelectObject(hdc, hbmp as HGDIOBJ);
        SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ);

        if GetClipBox(hdcOrig, core::ptr::addr_of_mut!(rcClient)) != 0 {
            IntersectClipRect(hdc, rcClient.left, rcClient.top, rcClient.right, rcClient.bottom);
        }
    } else {
        hOldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;
        oldBkMode = GetBkMode(hdc) as INT;
        oldBkColor = GetBkColor(hdc);
        oldTextColor = GetTextColor(hdc);
    }

    (*infoPtr).bIsDrawing = TRUE;

    if !prcErase.is_null() {
        LISTVIEW_FillBkgnd(infoPtr, hdc, prcErase);
    } else if ((*infoPtr).dwLvExStyle & LVS_EX_DOUBLEBUFFER) != 0 {
        BitBlt(
            hdc,
            (*infoPtr).rcList.left,
            (*infoPtr).rcList.top,
            (*infoPtr).rcList.right - (*infoPtr).rcList.left,
            (*infoPtr).rcList.bottom - (*infoPtr).rcList.top,
            hdcOrig,
            (*infoPtr).rcList.left,
            (*infoPtr).rcList.top,
            SRCCOPY,
        );
    }

    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcClient));
    customdraw_fill(
        core::ptr::addr_of_mut!(nmlvcd),
        infoPtr,
        hdc,
        core::ptr::addr_of_mut!(rcClient),
        core::ptr::null(),
    );
    cdmode = notify_customdraw(infoPtr, CDDS_PREPAINT, core::ptr::addr_of_mut!(nmlvcd));

    'draw: {
        if (cdmode & CDRF_SKIPDEFAULT) != 0 {
            break 'draw;
        }

        // nothing to draw
        if (*infoPtr).nItemCount == 0 {
            break 'draw;
        }

        // figure out what we need to draw
        iterator_visibleitems(core::ptr::addr_of_mut!(i), infoPtr, hdc);
        range = iterator_range(core::ptr::addr_of_mut!(i));

        // send cache hint notification
        if ((*infoPtr).dwStyle & LVS_OWNERDATA) != 0 {
            let mut nmlv: NMLVCACHEHINT = core::mem::zeroed();

            ZeroMemory!(core::ptr::addr_of_mut!(nmlv) as *mut core::ffi::c_void, core::mem::size_of::<NMLVCACHEHINT>());
            nmlv.iFrom = range.lower;
            nmlv.iTo = range.upper - 1;
            notify_hdr(infoPtr, LVN_ODCACHEHINT, core::ptr::addr_of_mut!(nmlv.hdr));
        }

        if ((*infoPtr).dwStyle & LVS_OWNERDRAWFIXED) != 0 && (*infoPtr).uView == LV_VIEW_DETAILS {
            LISTVIEW_RefreshOwnerDraw(infoPtr, core::ptr::addr_of_mut!(i), hdc, cdmode);
        } else {
            if (*infoPtr).uView == LV_VIEW_DETAILS {
                LISTVIEW_RefreshReport(infoPtr, core::ptr::addr_of_mut!(i), hdc, cdmode);
            } else {
                // LV_VIEW_LIST, LV_VIEW_ICON or LV_VIEW_SMALLICON
                LISTVIEW_RefreshList(infoPtr, core::ptr::addr_of_mut!(i), hdc, cdmode);
            }

            // if we have a focus rect and it's visible, draw it
            if (*infoPtr).bFocus != 0
                && range.lower <= (*infoPtr).nFocusedItem
                && (range.upper - 1) >= (*infoPtr).nFocusedItem
            {
                LISTVIEW_DrawFocusRect(infoPtr, hdc);
            }
        }
        iterator_destroy(core::ptr::addr_of_mut!(i));
    }

    // enddraw:
    // For LVS_EX_GRIDLINES go and draw lines
    // This includes the case where there were *no* items
    if ((*infoPtr).uView == LV_VIEW_DETAILS) && ((*infoPtr).dwLvExStyle & LVS_EX_GRIDLINES) != 0 {
        LISTVIEW_RefreshReportGrid(infoPtr, hdc);
    }

    // Draw marquee rectangle if appropriate
    if (*infoPtr).bMarqueeSelect != 0 {
        SetBkColor(hdc, RGB!(255, 255, 255));
        SetTextColor(hdc, RGB!(0, 0, 0));
        DrawFocusRect(hdc, core::ptr::addr_of_mut!((*infoPtr).marqueeDrawRect) as LPCRECT);
    }

    if (cdmode & CDRF_NOTIFYPOSTPAINT) != 0 {
        notify_postpaint(infoPtr, core::ptr::addr_of_mut!(nmlvcd));
    }

    if !hbmp.is_null() {
        BitBlt(
            hdcOrig,
            (*infoPtr).rcList.left,
            (*infoPtr).rcList.top,
            (*infoPtr).rcList.right - (*infoPtr).rcList.left,
            (*infoPtr).rcList.bottom - (*infoPtr).rcList.top,
            hdc,
            (*infoPtr).rcList.left,
            (*infoPtr).rcList.top,
            SRCCOPY,
        );

        DeleteObject(hbmp as HGDIOBJ);
        DeleteDC(hdc);
    } else {
        SelectObject(hdc, hOldFont as HGDIOBJ);
        SetBkMode(hdc, oldBkMode);
        SetBkColor(hdc, oldBkColor);
        SetTextColor(hdc, oldTextColor);
    }

    (*infoPtr).bIsDrawing = FALSE;
}

unsafe extern "C" fn LISTVIEW_ApproximateViewRect(infoPtr: *const LISTVIEW_INFO, mut nItemCount: INT, mut wWidth: WORD, mut wHeight: WORD) -> DWORD {
    let mut dwViewRect: DWORD = 0;

    if nItemCount == -1 {
        nItemCount = (*infoPtr).nItemCount;
    }

    if (*infoPtr).uView == LV_VIEW_LIST {
        let mut nItemCountPerColumn: INT = 1;
        let mut nColumnCount: INT = 0;

        if wHeight == 0xFFFF {
            /* use current height */
            wHeight = ((*infoPtr).rcList.bottom - (*infoPtr).rcList.top) as WORD;
        }

        if (wHeight as INT) < (*infoPtr).nItemHeight {
            wHeight = (*infoPtr).nItemHeight as WORD;
        }

        if nItemCount > 0 {
            if (*infoPtr).nItemHeight > 0 {
                nItemCountPerColumn = wHeight as INT / (*infoPtr).nItemHeight;
                if nItemCountPerColumn == 0 {
                    nItemCountPerColumn = 1;
                }

                if nItemCount % nItemCountPerColumn != 0 {
                    nColumnCount = nItemCount / nItemCountPerColumn;
                } else {
                    nColumnCount = nItemCount / nItemCountPerColumn + 1;
                }
            }
        }

        /* Microsoft padding magic */
        wHeight = (nItemCountPerColumn * (*infoPtr).nItemHeight + 2) as WORD;
        wWidth = (nColumnCount * (*infoPtr).nItemWidth + 2) as WORD;

        dwViewRect = MAKELONG!(wWidth, wHeight) as DWORD;
    } else if (*infoPtr).uView == LV_VIEW_DETAILS {
        let mut rcBox: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

        if (*infoPtr).nItemCount > 0 {
            LISTVIEW_GetItemBox(infoPtr, 0, core::ptr::addr_of_mut!(rcBox));
            wWidth = (rcBox.right - rcBox.left) as WORD;
            wHeight = ((rcBox.bottom - rcBox.top) * nItemCount) as WORD;
        } else {
            /* use current height and width */
            if wHeight == 0xffff {
                wHeight = ((*infoPtr).rcList.bottom - (*infoPtr).rcList.top) as WORD;
            }
            if wWidth == 0xffff {
                wWidth = ((*infoPtr).rcList.right - (*infoPtr).rcList.left) as WORD;
            }
        }

        dwViewRect = MAKELONG!(wWidth, wHeight) as DWORD;
    } else if (*infoPtr).uView == LV_VIEW_ICON {
        let mut rows: UINT = 0;
        let mut cols: UINT = 0;
        let mut nItemWidth: UINT = 0;
        let mut nItemHeight: UINT = 0;

        nItemWidth = (*infoPtr).iconSpacing.cx as UINT;
        nItemHeight = (*infoPtr).iconSpacing.cy as UINT;

        if wWidth == 0xffff {
            wWidth = ((*infoPtr).rcList.right - (*infoPtr).rcList.left) as WORD;
        }

        if (wWidth as UINT) < nItemWidth {
            wWidth = nItemWidth as WORD;
        }

        cols = (wWidth as UINT).wrapping_div(nItemWidth);
        if cols > nItemCount as UINT {
            cols = nItemCount as UINT;
        }
        if cols < 1 {
            cols = 1;
        }

        if nItemCount != 0 {
            rows = (nItemCount as UINT).wrapping_div(cols);
            if (nItemCount as UINT).wrapping_rem(cols) != 0 {
                rows = rows.wrapping_add(1);
            }
        } else {
            rows = 0;
        }

        wHeight = nItemHeight.wrapping_mul(rows).wrapping_add(2) as WORD;
        wWidth = nItemWidth.wrapping_mul(cols).wrapping_add(2) as WORD;

        dwViewRect = MAKELONG!(wWidth, wHeight) as DWORD;
    } else if (*infoPtr).uView == LV_VIEW_SMALLICON {
        FIXME!(b"uView == LV_VIEW_SMALLICON: not implemented\n\0".as_ptr() as *const ::core::ffi::c_char);
    }

    return dwViewRect;
}

unsafe extern "C" fn LISTVIEW_CancelEditLabel(infoPtr: *mut LISTVIEW_INFO) -> LRESULT {
    if !(*infoPtr).hwndEdit.is_null() {
        /* handle value will be lost after LISTVIEW_EndEditLabelT */
        let edit: HWND = (*infoPtr).hwndEdit;

        LISTVIEW_EndEditLabelT(infoPtr, TRUE as BOOL, IsWindowUnicode((*infoPtr).hwndEdit));
        SendMessageW(edit, WM_CLOSE as UINT, 0 as WPARAM, 0 as LPARAM);
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn LISTVIEW_CreateDragImage(infoPtr: *mut LISTVIEW_INFO, iItem: INT, lppt: LPPOINT) -> HIMAGELIST {
    let mut rcItem: RECT = tagRECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut size: SIZE = tagSIZE { cx: 0, cy: 0 };
    let mut pos: POINT = tagPOINT { x: 0, y: 0 };
    let mut hdc: HDC = core::ptr::null_mut::<HDC__>();
    let mut hdcOrig: HDC = core::ptr::null_mut::<HDC__>();
    let mut hbmp: HBITMAP = core::ptr::null_mut::<HBITMAP__>();
    let mut hOldbmp: HBITMAP = core::ptr::null_mut::<HBITMAP__>();
    let mut hOldFont: HFONT = core::ptr::null_mut::<HFONT__>();
    let mut dragList: HIMAGELIST = core::ptr::null_mut();

    if iItem < 0 || iItem >= (*infoPtr).nItemCount || lppt.is_null() {
        return core::ptr::null_mut();
    }

    rcItem.left = (LVIR_BOUNDS as i32);
    if LISTVIEW_GetItemRect(infoPtr, iItem, core::ptr::addr_of_mut!(rcItem)) == 0 {
        return core::ptr::null_mut();
    }

    (*lppt).x = rcItem.left;
    (*lppt).y = rcItem.top;

    size.cx = rcItem.right - rcItem.left;
    size.cy = rcItem.bottom - rcItem.top;

    hdcOrig = GetDC((*infoPtr).hwndSelf);
    hdc = CreateCompatibleDC(hdcOrig);
    hbmp = CreateCompatibleBitmap(hdcOrig, size.cx, size.cy);
    hOldbmp = SelectObject(hdc, hbmp as HGDIOBJ) as HBITMAP;
    hOldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;

    SetRect(
        core::ptr::addr_of_mut!(rcItem),
        0,
        0,
        size.cx,
        size.cy,
    );
    FillRect(hdc, core::ptr::addr_of_mut!(rcItem) as LPCRECT, (*infoPtr).hBkBrush);

    pos.y = 0;
    pos.x = 0;
    if LISTVIEW_DrawItem(
        infoPtr,
        hdc,
        iItem,
        core::ptr::null_mut::<ITERATOR>(),
        pos,
        CDRF_DODEFAULT,
    ) != 0
    {
        dragList = ImageList_Create(
            size.cx,
            size.cy,
            ILC_COLOR,
            10,
            10,
        );
        SelectObject(hdc, hOldbmp as HGDIOBJ);
        ImageList_Add(dragList, hbmp, core::ptr::null_mut());
    } else {
        SelectObject(hdc, hOldbmp as HGDIOBJ);
    }

    SelectObject(hdc, hOldFont as HGDIOBJ);
    DeleteObject(hbmp as HGDIOBJ);
    DeleteDC(hdc);
    ReleaseDC((*infoPtr).hwndSelf, hdcOrig);

    return dragList;
}

unsafe extern "C" fn LISTVIEW_DeleteAllItems(infoPtr: *mut LISTVIEW_INFO, destroy: BOOL) -> BOOL {
    let mut hdpaSubItems: HDPA = core::ptr::null_mut();
    let mut suppress: BOOL = FALSE;
    let mut hdrItem: *mut ITEMHDR = core::ptr::null_mut();
    let mut lpItem: *mut ITEM_INFO = core::ptr::null_mut();
    let mut lpID: *mut ITEM_ID = core::ptr::null_mut();
    let mut i: INT = 0;
    let mut j: INT = 0;

    /* we do it directly, to avoid notifications */
    ranges_clear((*infoPtr).selectionRanges);
    (*infoPtr).nSelectionMark = -(1);
    (*infoPtr).nFocusedItem = -(1);
    SetRectEmpty(core::ptr::addr_of_mut!((*infoPtr).rcFocus));
    /* But we are supposed to leave nHotItem as is! */

    /* send LVN_DELETEALLITEMS notification */
    if ((*infoPtr).dwStyle & LVS_OWNERDATA) == 0 || destroy == 0 {
        let mut nmlv: NMLISTVIEW = core::mem::zeroed();

        memset(
            core::ptr::addr_of_mut!(nmlv) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<NMLISTVIEW>(),
        );
        nmlv.iItem = -(1);
        suppress = notify_listview(
            infoPtr,
            LVN_DELETEALLITEMS as INT,
            core::ptr::addr_of_mut!(nmlv),
        ) as BOOL;
    }

    i = (*infoPtr).nItemCount - 1;
    while i >= 0 {
        if ((*infoPtr).dwStyle & LVS_OWNERDATA) == 0 {
            /* send LVN_DELETEITEM notification, if not suppressed
               and if it is not a virtual listview */
            if suppress == 0 {
                notify_deleteitem(infoPtr, i);
            }
            hdpaSubItems = DPA_GetPtr((*infoPtr).hdpaItems, i) as HDPA;
            lpItem = DPA_GetPtr(hdpaSubItems, 0) as *mut ITEM_INFO;
            /* free id struct */
            j = DPA_GetPtrIndex(
                (*infoPtr).hdpaItemIds,
                (*lpItem).id as *const core::ffi::c_void,
            ) as INT;
            lpID = DPA_GetPtr((*infoPtr).hdpaItemIds, j) as *mut ITEM_ID;
            DPA_DeletePtr((*infoPtr).hdpaItemIds, j);
            Free(lpID as LPVOID);
            /* both item and subitem start with ITEMHDR header */
            j = 0;
            while j < (DPA_GetPtrCount(hdpaSubItems) as INT) {
                hdrItem = DPA_GetPtr(hdpaSubItems, j) as *mut ITEMHDR;
                if is_text((*hdrItem).pszText as LPCWSTR) != 0 {
                    Free((*hdrItem).pszText as LPVOID);
                }
                Free(hdrItem as LPVOID);
                j += 1;
            }
            DPA_Destroy(hdpaSubItems);
            DPA_DeletePtr((*infoPtr).hdpaItems, i);
        }
        DPA_DeletePtr((*infoPtr).hdpaPosX, i);
        DPA_DeletePtr((*infoPtr).hdpaPosY, i);
        (*infoPtr).nItemCount -= 1;
        i -= 1;
    }

    if destroy == 0 {
        LISTVIEW_Arrange(infoPtr, (LVA_DEFAULT) as INT);
        LISTVIEW_UpdateScroll(infoPtr);
    }
    LISTVIEW_InvalidateList(infoPtr);

    return TRUE;
}

unsafe extern "C" fn LISTVIEW_ScrollColumns(infoPtr: *mut LISTVIEW_INFO, nColumn: INT, dx: INT) {
    let mut lpColumnInfo: *mut COLUMN_INFO = core::ptr::null_mut();
    let mut rcOld: RECT = core::mem::zeroed();
    let mut rcCol: RECT = core::mem::zeroed();
    let mut ptOrigin: POINT = core::mem::zeroed();
    let mut nCol: INT = 0;
    let mut hdi: HDITEMW = core::mem::zeroed();

    if nColumn < 0 || DPA_GetPtrCount((*infoPtr).hdpaColumns) < 1 {
        return;
    }
    lpColumnInfo = LISTVIEW_GetColumnInfo(
        infoPtr,
        min(nColumn, DPA_GetPtrCount((*infoPtr).hdpaColumns) - 1),
    );
    rcCol = (*lpColumnInfo).rcHeader;
    if nColumn >= DPA_GetPtrCount((*infoPtr).hdpaColumns) {
        rcCol.left = rcCol.right;
    }

    /* adjust the other columns */
    hdi.mask = HDI_ORDER;
    if SendMessageW(
        (*infoPtr).hwndHeader,
        HDM_GETITEMW,
        nColumn as WPARAM,
        core::ptr::addr_of_mut!(hdi) as LPARAM,
    ) != 0
    {
        let mut nOrder: INT = hdi.iOrder;
        nCol = 0;
        while nCol < DPA_GetPtrCount((*infoPtr).hdpaColumns) {
            hdi.mask = HDI_ORDER;
            SendMessageW(
                (*infoPtr).hwndHeader,
                HDM_GETITEMW,
                nCol as WPARAM,
                core::ptr::addr_of_mut!(hdi) as LPARAM,
            );
            if hdi.iOrder >= nOrder {
                lpColumnInfo = LISTVIEW_GetColumnInfo(infoPtr, nCol);
                (*lpColumnInfo).rcHeader.left += dx;
                (*lpColumnInfo).rcHeader.right += dx;
            }
            nCol += 1;
        }
    }

    /* do not update screen if not in report mode */
    if is_redrawing(infoPtr) == 0 || (*infoPtr).uView != LV_VIEW_DETAILS {
        return;
    }

    /* Need to reset the item width when inserting a new column */
    (*infoPtr).nItemWidth += dx;

    LISTVIEW_UpdateScroll(infoPtr);
    LISTVIEW_GetOrigin(infoPtr, core::ptr::addr_of_mut!(ptOrigin));

    /* scroll to cover the deleted column, and invalidate for redraw */
    rcOld = (*infoPtr).rcList;
    rcOld.left = ptOrigin.x + rcCol.left + dx;
    ScrollWindowEx(
        (*infoPtr).hwndSelf,
        dx,
        0,
        core::ptr::addr_of_mut!(rcOld),
        core::ptr::addr_of_mut!(rcOld),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        SW_ERASE | SW_INVALIDATE,
    );
}

unsafe extern "C" fn LISTVIEW_DeleteColumn(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut nColumn: INT,
) -> BOOL {
    let mut rcCol: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
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
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_DeleteColumn\0" as *const u8 as *const ::core::ffi::c_char,
            5762 as ::core::ffi::c_int,
            b"nColumn=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            nColumn,
        );
    }

    if nColumn < 0
        || nColumn >= DPA_GetPtrCount((*infoPtr).hdpaColumns)
    {
        return FALSE;
    }

    /* While the MSDN specifically says that column zero should not be deleted,
       what actually happens is that the column itself is deleted but no items or subitems
       are removed.
     */

    LISTVIEW_GetHeaderRect(infoPtr, nColumn, core::ptr::addr_of_mut!(rcCol));

    if SendMessageW(
        (*infoPtr).hwndHeader,
        HDM_DELETEITEM as UINT,
        nColumn as WPARAM,
        0 as LPARAM,
    ) == 0
    {
        return FALSE;
    }

    Free(DPA_GetPtr((*infoPtr).hdpaColumns, nColumn) as LPVOID);
    DPA_DeletePtr((*infoPtr).hdpaColumns, nColumn);

    if ((*infoPtr).dwStyle & (LVS_OWNERDATA as DWORD)) == 0 && nColumn != 0 {
        let mut lpSubItem: *mut SUBITEM_INFO = ::core::ptr::null_mut::<SUBITEM_INFO>();
        let mut lpDelItem: *mut SUBITEM_INFO = ::core::ptr::null_mut::<SUBITEM_INFO>();
        let mut hdpaSubItems: HDPA = ::core::ptr::null_mut::<_DPA>();
        let mut nItem: INT = 0;
        let mut nSubItem: INT = 0;
        let mut i: INT = 0;

        nItem = 0 as INT;
        while nItem < (*infoPtr).nItemCount {
            hdpaSubItems = DPA_GetPtr((*infoPtr).hdpaItems, nItem) as HDPA;
            nSubItem = 0 as INT;
            lpDelItem = 0 as *mut SUBITEM_INFO;
            i = 1 as INT;
            while i < DPA_GetPtrCount(hdpaSubItems) {
                lpSubItem = DPA_GetPtr(hdpaSubItems, i) as *mut SUBITEM_INFO;
                if (*lpSubItem).iSubItem == nColumn {
                    nSubItem = i;
                    lpDelItem = lpSubItem;
                } else if (*lpSubItem).iSubItem > nColumn {
                    (*lpSubItem).iSubItem -= 1;
                }
                i += 1;
            }

            /* if we found our subitem, zap it */
            if nSubItem > 0 as INT {
                /* free string */
                if is_text((*lpDelItem).hdr.pszText as LPCWSTR) != 0 {
                    Free((*lpDelItem).hdr.pszText as LPVOID);
                }

                /* free item */
                Free(lpDelItem as LPVOID);

                /* free dpa memory */
                DPA_DeletePtr(hdpaSubItems, nSubItem);
            }
            nItem += 1;
        }
    }

    /* update the other column info */
    if DPA_GetPtrCount((*infoPtr).hdpaColumns) == 0 as INT {
        LISTVIEW_InvalidateList(infoPtr);
    } else {
        LISTVIEW_ScrollColumns(infoPtr, nColumn, -(rcCol.right - rcCol.left));
    }
    LISTVIEW_UpdateItemSize(infoPtr);

    return TRUE;
}

unsafe extern "C" fn LISTVIEW_ScrollOnInsert(infoPtr: *mut LISTVIEW_INFO, nItem: INT, dir: INT) {
    let mut nPerCol: INT = 0;
    let mut nItemCol: INT = 0;
    let mut nItemRow: INT = 0;
    let mut rcScroll: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut Origin: POINT = POINT { x: 0, y: 0 };

    /* if we don't refresh, what's the point of scrolling? */
    if is_redrawing(infoPtr) == 0 {
        return;
    }

    assert!((abs(dir) == 1) as INT);

    /* arrange icons if autoarrange is on */
    if is_autoarrange(infoPtr) != 0 {
        let mut arrange: BOOL = TRUE;
        if dir < 0 && nItem >= (*infoPtr).nItemCount {
            arrange = FALSE;
        }
        if dir > 0 && nItem == (*infoPtr).nItemCount - 1 {
            arrange = FALSE;
        }
        if arrange != 0 {
            LISTVIEW_Arrange(infoPtr, (LVA_DEFAULT) as INT);
        }
    }

    /* scrollbars need updating */
    LISTVIEW_UpdateScroll(infoPtr);

    /* figure out the item's position */
    if (*infoPtr).uView == LV_VIEW_DETAILS {
        nPerCol = (*infoPtr).nItemCount + 1;
    } else if (*infoPtr).uView == LV_VIEW_LIST {
        nPerCol = LISTVIEW_GetCountPerColumn(infoPtr);
    } else {
        /* LV_VIEW_ICON, or LV_VIEW_SMALLICON */
        return;
    }

    nItemCol = nItem / nPerCol;
    nItemRow = nItem % nPerCol;
    LISTVIEW_GetOrigin(infoPtr, core::ptr::addr_of_mut!(Origin));

    /* move the items below up a slot */
    rcScroll.left = nItemCol * (*infoPtr).nItemWidth;
    rcScroll.top = nItemRow * (*infoPtr).nItemHeight;
    rcScroll.right = rcScroll.left + (*infoPtr).nItemWidth;
    rcScroll.bottom = nPerCol * (*infoPtr).nItemHeight;
    OffsetRect(core::ptr::addr_of_mut!(rcScroll), Origin.x, Origin.y);

    if IntersectRect(
        core::ptr::addr_of_mut!(rcScroll),
        core::ptr::addr_of_mut!(rcScroll),
        core::ptr::addr_of_mut!((*infoPtr).rcList),
    ) != 0
    {
        InvalidateRect(
            (*infoPtr).hwndSelf,
            core::ptr::addr_of_mut!(rcScroll),
            TRUE,
        );
    }

    /* report has only that column, so we're done */
    if (*infoPtr).uView == LV_VIEW_DETAILS {
        return;
    }

    /* now for LISTs, we have to deal with the columns to the right */
    SetRect(
        core::ptr::addr_of_mut!(rcScroll),
        (nItemCol + 1) * (*infoPtr).nItemWidth,
        0,
        ((*infoPtr).nItemCount / nPerCol + 1) * (*infoPtr).nItemWidth,
        nPerCol * (*infoPtr).nItemHeight,
    );
    OffsetRect(core::ptr::addr_of_mut!(rcScroll), Origin.x, Origin.y);
    if IntersectRect(
        core::ptr::addr_of_mut!(rcScroll),
        core::ptr::addr_of_mut!(rcScroll),
        core::ptr::addr_of_mut!((*infoPtr).rcList),
    ) != 0
    {
        InvalidateRect(
            (*infoPtr).hwndSelf,
            core::ptr::addr_of_mut!(rcScroll),
            TRUE,
        );
    }
}

unsafe extern "C" fn LISTVIEW_DeleteItem(mut infoPtr: *mut LISTVIEW_INFO, mut nItem: INT) -> BOOL {
    let mut item: LVITEMW = core::mem::zeroed();
    let is_icon: BOOL = ((*infoPtr).uView == LV_VIEW_SMALLICON as DWORD
        || (*infoPtr).uView == LV_VIEW_ICON as DWORD) as ::core::ffi::c_int;
    let mut focus: INT = (*infoPtr).nFocusedItem;
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_DeleteItem\0" as *const u8 as *const ::core::ffi::c_char,
            5921 as ::core::ffi::c_int,
            b"(nItem=%d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            nItem,
        );
    }
    if nItem < 0 as INT || nItem >= (*infoPtr).nItemCount {
        return FALSE;
    }
    item.state = 0 as UINT;
    item.stateMask = (LVIS_SELECTED | LVIS_FOCUSED) as UINT;
    LISTVIEW_SetItemState(infoPtr, nItem, core::ptr::addr_of_mut!(item));
    if notify_deleteitem(infoPtr, nItem) == 0 {
        return FALSE;
    }
    if is_icon != 0 {
        LISTVIEW_InvalidateItem(infoPtr, nItem);
    }
    if ((*infoPtr).dwStyle & (LVS_OWNERDATA as DWORD)) == 0 {
        let mut hdpaSubItems: HDPA = core::ptr::null_mut();
        let mut hdrItem: *mut ITEMHDR = core::ptr::null_mut();
        let mut lpItem: *mut ITEM_INFO = core::ptr::null_mut();
        let mut lpID: *mut ITEM_ID = core::ptr::null_mut();
        let mut i: INT = 0;
        hdpaSubItems = DPA_DeletePtr((*infoPtr).hdpaItems, nItem) as HDPA;
        lpItem = DPA_GetPtr(hdpaSubItems, 0 as INT) as *mut ITEM_INFO;
        i = DPA_GetPtrIndex(
            (*infoPtr).hdpaItemIds,
            (*lpItem).id as *const ::core::ffi::c_void,
        ) as INT;
        lpID = DPA_GetPtr((*infoPtr).hdpaItemIds, i) as *mut ITEM_ID;
        DPA_DeletePtr((*infoPtr).hdpaItemIds, i);
        Free(lpID as LPVOID);
        i = 0 as INT;
        while i < DPA_GetPtrCount(hdpaSubItems) {
            hdrItem = DPA_GetPtr(hdpaSubItems, i) as *mut ITEMHDR;
            if is_text((*hdrItem).pszText as LPCWSTR) != 0 {
                Free((*hdrItem).pszText as LPVOID);
            }
            Free(hdrItem as LPVOID);
            i += 1;
        }
        DPA_Destroy(hdpaSubItems);
    }
    if is_icon != 0 {
        DPA_DeletePtr((*infoPtr).hdpaPosX, nItem);
        DPA_DeletePtr((*infoPtr).hdpaPosY, nItem);
    }
    (*infoPtr).nItemCount -= 1;
    LISTVIEW_ShiftIndices(infoPtr, nItem, -(1 as INT));
    LISTVIEW_ShiftFocus(infoPtr, focus, nItem, -(1 as INT));
    if is_icon == 0 {
        LISTVIEW_ScrollOnInsert(infoPtr, nItem, -(1 as INT));
    }
    return TRUE;
}

unsafe extern "C" fn LISTVIEW_EndEditLabelT(infoPtr: *mut LISTVIEW_INFO, storeText: BOOL, isW: BOOL) -> BOOL {
    let hwndSelf: HWND = (*infoPtr).hwndSelf;
    let mut szDispText: [WCHAR; (DISP_TEXT_SIZE) as usize] = [0; (DISP_TEXT_SIZE) as usize];
    let mut dispInfo: NMLVDISPINFOW = core::mem::zeroed();
    let editedItem: INT = (*infoPtr).nEditLabelItem;
    let mut same: BOOL = 0;
    let mut pszText: *mut WCHAR = core::ptr::null_mut();
    let mut res: BOOL = 0;

    'cleanup: {
        if storeText != 0 {
            let mut len: DWORD = (if isW != 0 {
                GetWindowTextLengthW((*infoPtr).hwndEdit)
            } else {
                GetWindowTextLengthA((*infoPtr).hwndEdit)
            }) as DWORD;
            let fresh19: DWORD = len;
            len = len.wrapping_add(1);
            if fresh19 != 0 {
                pszText = Alloc(
                    (len as usize).wrapping_mul(
                        if isW != 0 {
                            core::mem::size_of::<WCHAR>()
                        } else {
                            core::mem::size_of::<CHAR>() as usize
                        },
                    ) as DWORD,
                ) as *mut WCHAR;
                if pszText.is_null() {
                    return FALSE;
                }
                if isW != 0 {
                    GetWindowTextW((*infoPtr).hwndEdit, pszText, len as ::core::ffi::c_int);
                } else {
                    GetWindowTextA((*infoPtr).hwndEdit, pszText as LPSTR, len as ::core::ffi::c_int);
                }
            }
        }

        core::ptr::write_bytes(
            core::ptr::addr_of_mut!(dispInfo) as *mut u8,
            0,
            core::mem::size_of::<NMLVDISPINFOW>(),
        );
        dispInfo.item.mask = LVIF_PARAM | LVIF_STATE | LVIF_TEXT;
        dispInfo.item.iItem = editedItem;
        dispInfo.item.iSubItem = 0;
        dispInfo.item.stateMask = !0;
        dispInfo.item.pszText = szDispText.as_mut_ptr();
        dispInfo.item.cchTextMax = DISP_TEXT_SIZE;
        if LISTVIEW_GetItemT(infoPtr, core::ptr::addr_of_mut!(dispInfo.item), isW) == 0 {
            res = FALSE;
            break 'cleanup;
        }

        if isW != 0 {
            same = (lstrcmpW(dispInfo.item.pszText as LPCWSTR, pszText as LPCWSTR)
                == 0) as BOOL;
        } else {
            let tmp: LPWSTR = textdupTtoW(pszText as LPCWSTR, FALSE);
            same = (lstrcmpW(dispInfo.item.pszText as LPCWSTR, tmp as LPCWSTR)
                == 0) as BOOL;
            textfreeT(tmp, FALSE);
        }

        /* add the text from the edit in */
        dispInfo.item.mask |= LVIF_TEXT;
        dispInfo.item.pszText = (if same != 0 {
            core::ptr::null_mut::<WCHAR>()
        } else {
            pszText
        }) as LPWSTR;
        dispInfo.item.cchTextMax = textlenT(dispInfo.item.pszText as LPCWSTR, isW);

        (*infoPtr).notify_mask &= !NOTIFY_MASK_END_LABEL_EDIT;

        /* Do we need to update the Item Text */
        res = notify_dispinfoT(infoPtr, (LVN_ENDLABELEDITW) as UINT, core::ptr::addr_of_mut!(dispInfo), isW);

        (*infoPtr).notify_mask |= NOTIFY_MASK_END_LABEL_EDIT;

        (*infoPtr).nEditLabelItem = -1;
        (*infoPtr).hwndEdit = core::ptr::null_mut();

        if res == 0 {
            break 'cleanup;
        }

        if IsWindow(hwndSelf) == 0 {
            res = FALSE;
            break 'cleanup;
        }
        if pszText.is_null() {
            return TRUE;
        }
        if same != 0 {
            res = TRUE;
            break 'cleanup;
        }

        if ((*infoPtr).dwStyle & LVS_OWNERDATA) == 0 {
            let hdpaSubItems: HDPA = DPA_GetPtr((*infoPtr).hdpaItems, editedItem) as HDPA;
            let lpItem: *mut ITEM_INFO =
                DPA_GetPtr(hdpaSubItems, 0) as *mut ITEM_INFO;
            if !lpItem.is_null() && (*lpItem).hdr.pszText == LPSTR_TEXTCALLBACKW {
                LISTVIEW_InvalidateItem(infoPtr, editedItem);
                res = TRUE;
                break 'cleanup;
            }
        }

        core::ptr::write_bytes(
            core::ptr::addr_of_mut!(dispInfo) as *mut u8,
            0,
            core::mem::size_of::<NMLVDISPINFOW>(),
        );
        dispInfo.item.mask = LVIF_TEXT;
        dispInfo.item.iItem = editedItem;
        dispInfo.item.iSubItem = 0;
        dispInfo.item.pszText = pszText;
        dispInfo.item.cchTextMax = textlenT(pszText as LPCWSTR, isW);
        res = LISTVIEW_SetItemT(infoPtr, core::ptr::addr_of_mut!(dispInfo.item), isW);
    }

    Free(pszText as LPVOID);

    return res;
}

unsafe extern "C" fn EditLblWndProcT(mut hwnd: HWND, mut uMsg: UINT, mut wParam: WPARAM, mut lParam: LPARAM, mut isW: BOOL) -> LRESULT {
    let mut infoPtr: *mut LISTVIEW_INFO = GetWindowLongPtrW(GetParent(hwnd), 0) as *mut LISTVIEW_INFO;
    let mut save: BOOL = TRUE;
    's_switch: {
        if uMsg == WM_GETDLGCODE as UINT {
            return (DLGC_WANTARROWS | DLGC_WANTALLKEYS) as LRESULT;
        }
        if uMsg == WM_DESTROY as UINT {
            let mut editProc: WNDPROC = (*infoPtr).EditWndProc;
            (*infoPtr).EditWndProc = None;
            SetWindowLongPtrW(hwnd, GWLP_WNDPROC as INT, ::core::mem::transmute::<WNDPROC, DWORD_PTR>(editProc) as LONG_PTR);
            return CallWindowProcT(editProc, hwnd, uMsg, wParam, lParam, isW);
        }
        if uMsg == WM_KEYDOWN as UINT {
            if VK_ESCAPE == (wParam as INT) as u32 {
                save = FALSE;
                break 's_switch;
            } else if VK_RETURN == (wParam as INT) as u32 {
                break 's_switch;
            }
        }
        return CallWindowProcT((*infoPtr).EditWndProc, hwnd, uMsg, wParam, lParam, isW);
    }

    /* kill the edit */
    if !(*infoPtr).hwndEdit.is_null() {
        LISTVIEW_EndEditLabelT(infoPtr, save, isW);
    }

    SendMessageW(hwnd, WM_CLOSE as UINT, 0 as WPARAM, 0 as LPARAM);
    return 0 as LRESULT;
}

unsafe extern "system" fn EditLblWndProcW(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    return EditLblWndProcT(hwnd, uMsg, wParam, lParam, TRUE);
}

unsafe extern "system" fn EditLblWndProcA(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    return EditLblWndProcT(hwnd, uMsg, wParam, lParam, FALSE);
}

unsafe extern "C" fn CreateEditLabelT(infoPtr: *mut LISTVIEW_INFO, text: LPCWSTR, isW: BOOL) -> HWND {
    static style: DWORD = WS_CHILDWINDOW | WS_CLIPSIBLINGS | ES_LEFT | ES_AUTOHSCROLL | WS_BORDER | WS_VISIBLE;
    let hinst: HINSTANCE = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_HINSTANCE) as HINSTANCE;
    let mut hedit: HWND;

    // TRACE(...) debug block omitted: the function body in the C source contains no trace statement
    // and no debug channel is available in this translation unit.

    /* window will be resized and positioned after LVN_BEGINLABELEDIT */
    if isW != 0 {
        hedit = CreateWindowW!(WC_EDITW.as_ptr(), text, style, 0, 0, 0, 0, (*infoPtr).hwndSelf, core::ptr::null_mut(), hinst, core::ptr::null_mut());
    } else {
        hedit = CreateWindowA!(WC_EDITA.as_ptr() as LPCSTR, text as LPCSTR, style, 0, 0, 0, 0, (*infoPtr).hwndSelf, core::ptr::null_mut(), hinst, core::ptr::null_mut());
    }

    if hedit.is_null() {
        return core::ptr::null_mut();
    }

    (*infoPtr).EditWndProc = ::core::mem::transmute::<DWORD_PTR, WNDPROC>(
        if isW != 0 {
            SetWindowLongPtrW(hedit, GWLP_WNDPROC, EditLblWndProcW as DWORD_PTR as LONG_PTR) as DWORD_PTR
        } else {
            SetWindowLongPtrA(hedit, GWLP_WNDPROC, EditLblWndProcA as DWORD_PTR as LONG_PTR) as DWORD_PTR
        },
    );

    SendMessageW(hedit, WM_SETFONT, (*infoPtr).hFont as WPARAM, (FALSE) as LPARAM);
    SendMessageW(hedit, EM_SETLIMITTEXT, (DISP_TEXT_SIZE - 1) as WPARAM, 0);

    return hedit;
}

unsafe extern "C" fn LISTVIEW_EditLabelT(
    infoPtr: *mut LISTVIEW_INFO,
    nItem: INT,
    isW: BOOL,
) -> HWND {
    let mut disptextW: [WCHAR; (DISP_TEXT_SIZE) as usize] = [0; (DISP_TEXT_SIZE) as usize];
    let hwndSelf: HWND = (*infoPtr).hwndSelf;
    let mut dispInfo: NMLVDISPINFOW = core::mem::zeroed();
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let mut sz: SIZE = core::mem::zeroed();
    let mut hdc: HDC = core::ptr::null_mut();

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_EditLabelT\0" as *const u8 as *const ::core::ffi::c_char,
            6249 as ::core::ffi::c_int,
            b"(nItem=%d, isW=%d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            nItem,
            isW,
        );
    }

    if (!(*infoPtr).dwStyle & LVS_EDITLABELS) != 0 {
        return core::ptr::null_mut();
    }

    /* remove existing edit box */
    if !(*infoPtr).hwndEdit.is_null() {
        SetFocus((*infoPtr).hwndSelf);
        (*infoPtr).hwndEdit = core::ptr::null_mut();
    }

    if nItem < 0 || nItem >= (*infoPtr).nItemCount {
        return core::ptr::null_mut();
    }

    (*infoPtr).nEditLabelItem = nItem;

    LISTVIEW_SetSelection(infoPtr, nItem);
    LISTVIEW_SetItemFocus(infoPtr, nItem);
    LISTVIEW_InvalidateItem(infoPtr, nItem);

    rect.left = (LVIR_LABEL as i32);
    if LISTVIEW_GetItemRect(infoPtr, nItem, &raw mut rect) == 0 {
        return core::ptr::null_mut();
    }

    ZeroMemory!(
        &raw mut dispInfo as *mut ::core::ffi::c_void,
        core::mem::size_of::<NMLVDISPINFOW>(),
    );
    dispInfo.item.mask = LVIF_PARAM | LVIF_STATE | LVIF_TEXT;
    dispInfo.item.iItem = nItem;
    dispInfo.item.iSubItem = 0;
    dispInfo.item.stateMask = !0;
    dispInfo.item.pszText = &raw mut disptextW as *mut WCHAR as LPWSTR;
    dispInfo.item.cchTextMax = DISP_TEXT_SIZE;
    if LISTVIEW_GetItemT(infoPtr, &raw mut dispInfo.item, isW) == 0 {
        return core::ptr::null_mut();
    }

    (*infoPtr).hwndEdit = CreateEditLabelT(infoPtr, dispInfo.item.pszText as LPCWSTR, isW);
    if (*infoPtr).hwndEdit.is_null() {
        return core::ptr::null_mut();
    }

    if notify_dispinfoT(infoPtr, (LVN_BEGINLABELEDITW) as UINT, &raw mut dispInfo, isW) != 0 {
        if IsWindow(hwndSelf) == 0 {
            return core::ptr::null_mut();
        }
        SendMessageW(
            (*infoPtr).hwndEdit,
            WM_CLOSE,
            0,
            0,
        );
        (*infoPtr).hwndEdit = core::ptr::null_mut();
        return core::ptr::null_mut();
    }

    /* position and display edit box */
    hdc = GetDC((*infoPtr).hwndSelf);

    /* select the font to get appropriate metric dimensions */
    if !(*infoPtr).hFont.is_null() {
        hOldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;
    }

    /* use real edit box content, it could be altered during LVN_BEGINLABELEDIT notification */
    GetWindowTextW(
        (*infoPtr).hwndEdit,
        &raw mut disptextW as LPWSTR,
        DISP_TEXT_SIZE,
    );

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_EditLabelT\0" as *const u8 as *const ::core::ffi::c_char,
            6303 as ::core::ffi::c_int,
            b"edit box text=%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(&raw mut disptextW as *mut WCHAR),
        );
    }

    /* get string length in pixels */
    GetTextExtentPoint32W(
        hdc,
        &raw mut disptextW as *mut WCHAR as LPCWSTR,
        lstrlenW(&raw mut disptextW as *mut WCHAR as LPCWSTR),
        &raw mut sz,
    );

    /* add extra spacing for the next character */
    GetTextMetricsW(hdc, &raw mut tm);
    sz.cx += tm.tmMaxCharWidth * 2;

    if !(*infoPtr).hFont.is_null() {
        SelectObject(hdc, hOldFont as HGDIOBJ);
    }

    ReleaseDC((*infoPtr).hwndSelf, hdc);

    sz.cy = rect.bottom - rect.top + 2;
    rect.left -= 2;
    rect.top -= 1;

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
        != 0
    {
        let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_2,
            __dbch_2,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_EditLabelT\0" as *const u8 as *const ::core::ffi::c_char,
            6320 as ::core::ffi::c_int,
            b"moving edit=(%d,%d)-(%d,%d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            rect.left,
            rect.top,
            sz.cx,
            sz.cy,
        );
    }

    MoveWindow(
        (*infoPtr).hwndEdit,
        rect.left,
        rect.top,
        sz.cx,
        sz.cy,
        FALSE,
    );
    ShowWindow((*infoPtr).hwndEdit, (SW_NORMAL) as i32);
    SetFocus((*infoPtr).hwndEdit);
    SendMessageW(
        (*infoPtr).hwndEdit,
        EM_SETSEL,
        0,
        -1,
    );
    return (*infoPtr).hwndEdit;
}

unsafe extern "C" fn LISTVIEW_EnsureVisible(
    infoPtr: *mut LISTVIEW_INFO,
    nItem: INT,
    bPartial: BOOL,
) -> BOOL {
    let mut nScrollPosHeight: INT = 0;
    let mut nScrollPosWidth: INT = 0;
    let mut nHorzAdjust: INT = 0;
    let mut nVertAdjust: INT = 0;
    let mut nHorzDiff: INT = 0;
    let mut nVertDiff: INT = 0;
    let mut rcItem: RECT = core::mem::zeroed();
    let mut rcTemp: RECT = core::mem::zeroed();

    rcItem.left = (LVIR_BOUNDS as i32);
    if LISTVIEW_GetItemRect(infoPtr, nItem, &raw mut rcItem) == 0 {
        return FALSE;
    }

    if bPartial != 0
        && IntersectRect(
            &raw mut rcTemp,
            &raw mut (*infoPtr).rcList as LPCRECT,
            &raw mut rcItem as LPCRECT,
        ) != 0
    {
        return TRUE;
    }

    if rcItem.left < (*infoPtr).rcList.left || rcItem.right > (*infoPtr).rcList.right {
        if (*infoPtr).uView == LV_VIEW_LIST {
            nScrollPosWidth = (*infoPtr).nItemWidth;
        } else if (*infoPtr).uView == LV_VIEW_SMALLICON
            || (*infoPtr).uView == LV_VIEW_ICON
        {
            nScrollPosWidth = 1;
        }

        if rcItem.left < (*infoPtr).rcList.left {
            nHorzAdjust = -1;
            if (*infoPtr).uView != LV_VIEW_DETAILS {
                nHorzDiff = rcItem.left - (*infoPtr).rcList.left;
            }
        } else {
            nHorzAdjust = 1;
            if (*infoPtr).uView != LV_VIEW_DETAILS {
                nHorzDiff = rcItem.right - (*infoPtr).rcList.right;
            }
        }
    }

    if rcItem.top < (*infoPtr).rcList.top || rcItem.bottom > (*infoPtr).rcList.bottom {
        if (*infoPtr).uView == LV_VIEW_DETAILS {
            nScrollPosHeight = (*infoPtr).nItemHeight;
        } else if (*infoPtr).uView == LV_VIEW_ICON
            || (*infoPtr).uView == LV_VIEW_SMALLICON
        {
            nScrollPosHeight = 1;
        }

        if rcItem.top < (*infoPtr).rcList.top {
            nVertAdjust = -1;
            if (*infoPtr).uView != LV_VIEW_LIST {
                nVertDiff = rcItem.top - (*infoPtr).rcList.top;
            }
        } else {
            nVertAdjust = 1;
            if (*infoPtr).uView != LV_VIEW_LIST {
                nVertDiff = rcItem.bottom - (*infoPtr).rcList.bottom;
            }
        }
    }

    if nScrollPosWidth == 0 && nScrollPosHeight == 0 {
        return TRUE;
    }

    if nScrollPosWidth != 0 {
        let mut diff: INT = nHorzDiff / nScrollPosWidth;
        if nHorzDiff % nScrollPosWidth != 0 {
            diff += nHorzAdjust;
        }
        LISTVIEW_HScroll(infoPtr, SB_INTERNAL, diff);
    }

    if nScrollPosHeight != 0 {
        let mut diff_0: INT = nVertDiff / nScrollPosHeight;
        if nVertDiff % nScrollPosHeight != 0 {
            diff_0 += nVertAdjust;
        }
        LISTVIEW_VScroll(infoPtr, SB_INTERNAL, diff_0);
    }

    return TRUE;
}

unsafe extern "C" fn LISTVIEW_FindItemW(
    infoPtr: *const LISTVIEW_INFO,
    nStart: INT,
    lpFindInfo: *const LVFINDINFOW,
) -> INT {
    let mut szDispText: [WCHAR; DISP_TEXT_SIZE as usize] = [0 as WCHAR; DISP_TEXT_SIZE as usize];
    let mut bWrap: BOOL = FALSE as BOOL;
    let mut bNearest: BOOL = FALSE as BOOL;
    let mut nItem: INT = nStart + 1 as INT;
    let mut nLast: INT = (*infoPtr).nItemCount;
    let mut nNearestItem: INT = -(1 as INT);
    let mut xdist: ULONG = 0;
    let mut ydist: ULONG = 0;
    let mut dist: ULONG = 0;
    let mut mindist: ULONG = 0x7fffffff as ULONG;
    let mut Position: POINT = POINT { x: 0, y: 0 };
    let mut Destination: POINT = POINT { x: 0, y: 0 };
    let mut lvItem: LVITEMW = core::mem::zeroed::<LVITEMW>();

    if ((*infoPtr).dwStyle & (LVS_OWNERDATA as DWORD)) != 0 {
        let mut nmlv: NMLVFINDITEMW = core::mem::zeroed::<NMLVFINDITEMW>();
        nmlv.iStart = nStart;
        nmlv.lvfi = *lpFindInfo;
        return notify_hdr(infoPtr, LVN_ODFINDITEMW as INT, &raw mut nmlv.hdr as LPNMHDR) as INT;
    }

    if lpFindInfo.is_null() || nItem < 0 as INT {
        return -(1 as INT);
    }

    lvItem.mask = 0 as UINT;
    if ((*lpFindInfo).flags & ((LVFI_STRING as UINT) | (LVFI_PARTIAL as UINT) | (LVFI_SUBSTRING as UINT))) != 0 {
        lvItem.mask |= LVIF_TEXT as UINT;
        lvItem.pszText = &raw mut szDispText as *mut WCHAR as LPWSTR;
        lvItem.cchTextMax = DISP_TEXT_SIZE as INT;
    }

    if ((*lpFindInfo).flags & (LVFI_WRAP as UINT)) != 0 {
        bWrap = TRUE as BOOL;
    }

    if ((*lpFindInfo).flags & (LVFI_NEARESTXY as UINT)) != 0
        && ((*infoPtr).uView == LV_VIEW_ICON as DWORD
            || (*infoPtr).uView == LV_VIEW_SMALLICON as DWORD)
    {
        let mut Origin: POINT = POINT { x: 0, y: 0 };
        let mut rcArea: RECT = core::mem::zeroed::<RECT>();

        LISTVIEW_GetOrigin(infoPtr, &raw mut Origin);
        Destination.x = (*lpFindInfo).pt.x - Origin.x;
        Destination.y = (*lpFindInfo).pt.y - Origin.y;
        match (*lpFindInfo).vkDirection {
            VK_DOWN => {
                Destination.y += (*infoPtr).nItemHeight as LONG;
            }
            VK_UP => {
                Destination.y -= (*infoPtr).nItemHeight as LONG;
            }
            VK_RIGHT => {
                Destination.x += (*infoPtr).nItemWidth as LONG;
            }
            VK_LEFT => {
                Destination.x -= (*infoPtr).nItemWidth as LONG;
            }
            VK_HOME => {
                Destination.y = 0 as LONG;
                Destination.x = Destination.y;
            }
            VK_NEXT => {
                Destination.y += (*infoPtr).rcList.bottom - (*infoPtr).rcList.top;
            }
            VK_PRIOR => {
                Destination.y -= (*infoPtr).rcList.bottom - (*infoPtr).rcList.top;
            }
            VK_END => {
                LISTVIEW_GetAreaRect(infoPtr, &raw mut rcArea);
                Destination.x = rcArea.right;
                Destination.y = rcArea.bottom;
            }
            _ => {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/comctl32/listview.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"LISTVIEW_FindItemW\0" as *const u8
                            as *const ::core::ffi::c_char,
                        6486 as ::core::ffi::c_int,
                        b"Unknown vkDirection=%d\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        (*lpFindInfo).vkDirection,
                    );
                }
            }
        }
        bNearest = TRUE as BOOL;
    } else {
        Destination.y = 0 as LONG;
        Destination.x = Destination.y;
    }

    if ((*lpFindInfo).flags & (LVFI_PARAM as UINT)) != 0 {
        lvItem.mask |= LVIF_PARAM as UINT;
        bNearest = FALSE as BOOL;
        lvItem.mask &= !(LVIF_TEXT as UINT);
    }

    nItem = if bNearest != 0 {
        -(1 as INT)
    } else {
        nStart + 1 as INT
    };

    loop {
        while nItem < nLast {
            'next: {
                lvItem.iItem = nItem;
                lvItem.iSubItem = 0 as INT;
                lvItem.pszText = &raw mut szDispText as *mut WCHAR as LPWSTR;
                if LISTVIEW_GetItemW(infoPtr, &raw mut lvItem) == 0 {
                    break 'next;
                }

                if (lvItem.mask & (LVIF_PARAM as UINT)) != 0 {
                    if (*lpFindInfo).lParam == lvItem.lParam {
                        return nItem;
                    } else {
                        break 'next;
                    }
                }

                if (lvItem.mask & (LVIF_TEXT as UINT)) != 0 {
                    if ((*lpFindInfo).flags & ((LVFI_PARTIAL as UINT) | (LVFI_SUBSTRING as UINT))) != 0 {
                        let p: *mut WCHAR = wcsstr(
                            lvItem.pszText as *const wchar_t,
                            (*lpFindInfo).psz as *const wchar_t,
                        ) as *mut WCHAR;
                        if p.is_null() || p != lvItem.pszText {
                            break 'next;
                        }
                    } else {
                        if lstrcmpW(lvItem.pszText as LPCWSTR, (*lpFindInfo).psz)
                            != 0 as INT
                        {
                            break 'next;
                        }
                    }
                }

                if bNearest == 0 {
                    return nItem;
                }

                LISTVIEW_GetItemOrigin(infoPtr, nItem, &raw mut Position);

                xdist = (Destination.x - Position.x) as ULONG;
                ydist = (Destination.y - Position.y) as ULONG;
                dist = xdist.wrapping_mul(xdist).wrapping_add(ydist.wrapping_mul(ydist));

                if dist < mindist {
                    mindist = dist;
                    nNearestItem = nItem;
                }
            }
            nItem += 1;
        }

        if bWrap == 0 {
            break;
        }

        nItem = 0 as INT;
        let wrapEnd: INT = nStart + 1 as INT;
        nLast = if wrapEnd < (*infoPtr).nItemCount {
            wrapEnd
        } else {
            (*infoPtr).nItemCount
        };
        bWrap = FALSE as BOOL;
    }

    return nNearestItem;
}

unsafe extern "C" fn LISTVIEW_FindItemA(
    infoPtr: *const LISTVIEW_INFO,
    nStart: INT,
    lpFindInfo: *const LVFINDINFOA,
) -> INT {
    let mut fiw: LVFINDINFOW = tagLVFINDINFOW {
        flags: 0,
        psz: ::core::ptr::null::<WCHAR>(),
        lParam: 0,
        pt: tagPOINT { x: 0, y: 0 },
        vkDirection: 0,
    };
    let mut res: INT = 0;
    let mut strW: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    memcpy(
        &raw mut fiw as *mut ::core::ffi::c_void,
        lpFindInfo as *const ::core::ffi::c_void,
        ::core::mem::size_of::<LVFINDINFOW>() as size_t,
    );
    if ((*lpFindInfo).flags & ((LVFI_STRING | LVFI_PARTIAL | LVFI_SUBSTRING) as UINT)) != 0 {
        strW = textdupTtoW((*lpFindInfo).psz as LPCWSTR, FALSE);
        fiw.psz = strW as LPCWSTR;
    }
    res = LISTVIEW_FindItemW(infoPtr, nStart, &raw mut fiw);
    textfreeT(strW, FALSE);
    return res;
}

unsafe extern "C" fn LISTVIEW_GetColumnT(
    mut infoPtr: *const LISTVIEW_INFO,
    mut nColumn: INT,
    mut lpColumn: LPLVCOLUMNW,
    mut isW: BOOL,
) -> BOOL {
    let mut lpColumnInfo: *mut COLUMN_INFO = ::core::ptr::null_mut::<COLUMN_INFO>();
    let mut hdi: HDITEMW = _HD_ITEMW {
        mask: 0,
        cxy: 0,
        pszText: ::core::ptr::null_mut::<WCHAR>(),
        hbm: ::core::ptr::null_mut::<HBITMAP__>(),
        cchTextMax: 0,
        fmt: 0,
        lParam: 0,
        iImage: 0,
        iOrder: 0,
        type_: 0,
        pvFilter: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    if lpColumn.is_null()
        || nColumn < 0
        || nColumn >= DPA_GetPtrCount((*infoPtr).hdpaColumns)
    {
        return FALSE;
    }
    lpColumnInfo = LISTVIEW_GetColumnInfo(infoPtr, nColumn);
    ZeroMemory!(
        &raw mut hdi as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<HDITEMW>() as size_t,
    );
    if ((*lpColumn).mask & (LVCF_TEXT as UINT)) != 0 {
        hdi.mask |= HDI_TEXT as UINT;
        hdi.pszText = (*lpColumn).pszText;
        hdi.cchTextMax = (*lpColumn).cchTextMax;
    }
    if ((*lpColumn).mask & (LVCF_IMAGE as UINT)) != 0 {
        hdi.mask |= HDI_IMAGE as UINT;
    }
    if ((*lpColumn).mask & (LVCF_ORDER as UINT)) != 0 {
        hdi.mask |= HDI_ORDER as UINT;
    }
    if ((*lpColumn).mask & (LVCF_SUBITEM as UINT)) != 0 {
        hdi.mask |= HDI_LPARAM as UINT;
    }
    if SendMessageW(
        (*infoPtr).hwndHeader,
        (if isW != 0 { HDM_GETITEMW } else { HDM_GETITEMA }) as UINT,
        nColumn as WPARAM,
        &raw mut hdi as LPARAM,
    ) == 0
    {
        return FALSE;
    }
    if ((*lpColumn).mask & (LVCF_FMT as UINT)) != 0 {
        (*lpColumn).fmt = (*lpColumnInfo).fmt as ::core::ffi::c_int;
    }
    if ((*lpColumn).mask & (LVCF_WIDTH as UINT)) != 0 {
        (*lpColumn).cx =
            ((*lpColumnInfo).rcHeader.right - (*lpColumnInfo).rcHeader.left) as ::core::ffi::c_int;
    }
    if ((*lpColumn).mask & (LVCF_IMAGE as UINT)) != 0 {
        (*lpColumn).iImage = hdi.iImage;
    }
    if ((*lpColumn).mask & (LVCF_ORDER as UINT)) != 0 {
        (*lpColumn).iOrder = hdi.iOrder;
    }
    if ((*lpColumn).mask & (LVCF_SUBITEM as UINT)) != 0 {
        (*lpColumn).iSubItem = hdi.lParam as ::core::ffi::c_int;
    }
    if ((*lpColumn).mask & (LVCF_MINWIDTH as UINT)) != 0 {
        (*lpColumn).cxMin = (*lpColumnInfo).cxMin as ::core::ffi::c_int;
    }
    return TRUE;
}

unsafe extern "C" fn LISTVIEW_GetColumnOrderArray(infoPtr: *const LISTVIEW_INFO, iCount: INT, lpiArray: LPINT) -> BOOL {
    if (*infoPtr).hwndHeader.is_null() {
        return FALSE;
    }
    return SendMessageW(
        (*infoPtr).hwndHeader,
        HDM_GETORDERARRAY as UINT,
        iCount as WPARAM,
        lpiArray as LPARAM,
    ) as BOOL;
}

unsafe extern "C" fn LISTVIEW_GetColumnWidth(infoPtr: *const LISTVIEW_INFO, nColumn: INT) -> INT {
    let mut nColumnWidth: INT = 0;
    let mut hdItem: HDITEMW = core::mem::zeroed();

    /* we have a 'column' in LIST and REPORT mode only */
    match (*infoPtr).uView {
        LV_VIEW_LIST => {
            nColumnWidth = (*infoPtr).nItemWidth;
        }
        LV_VIEW_DETAILS => {
            /* We are not using LISTVIEW_GetHeaderRect as this data is updated only after a HDN_ITEMCHANGED.
             * There is an application that subclasses the listview, calls LVM_GETCOLUMNWIDTH in the
             * HDN_ITEMCHANGED handler and goes into infinite recursion if it receives old data.
             */
            hdItem.mask = HDI_WIDTH;
            if SendMessageW(
                (*infoPtr).hwndHeader,
                HDM_GETITEMW,
                nColumn as WPARAM,
                core::ptr::addr_of_mut!(hdItem) as LPARAM,
            ) == 0
            {
                return 0;
            }
            nColumnWidth = hdItem.cxy;
        }
        _ => {}
    }

    return nColumnWidth;
}

unsafe extern "C" fn LISTVIEW_GetCountPerPage(infoPtr: *const LISTVIEW_INFO) -> INT {
    match (*infoPtr).uView {
        LV_VIEW_ICON | LV_VIEW_SMALLICON => return (*infoPtr).nItemCount,
        LV_VIEW_DETAILS => return LISTVIEW_GetCountPerColumn(infoPtr),
        LV_VIEW_LIST => {
            return LISTVIEW_GetCountPerRow(infoPtr) * LISTVIEW_GetCountPerColumn(infoPtr);
        }
        _ => {}
    }
    assert!(FALSE);
    return 0;
}

unsafe extern "C" fn LISTVIEW_GetImageList(infoPtr: *const LISTVIEW_INFO, nImageList: INT) -> HIMAGELIST {
    match (nImageList) as u32 {
        LVSIL_NORMAL => return (*infoPtr).himlNormal,
        LVSIL_SMALL => return (*infoPtr).himlSmall,
        LVSIL_STATE => return (*infoPtr).himlState,
        LVSIL_GROUPHEADER => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/listview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"LISTVIEW_GetImageList\0" as *const u8 as *const ::core::ffi::c_char,
                    6752 as ::core::ffi::c_int,
                    b"LVSIL_GROUPHEADER not supported\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/listview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"LISTVIEW_GetImageList\0" as *const u8 as *const ::core::ffi::c_char,
                    6755 as ::core::ffi::c_int,
                    b"got unknown imagelist index - %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                    nImageList,
                );
            }
        }
    }
    return core::ptr::null_mut::<_IMAGELIST>();
}

unsafe extern "C" fn LISTVIEW_GetItemT(infoPtr: *const LISTVIEW_INFO, lpLVItem: LPLVITEMW, isW: BOOL) -> BOOL {
    let mut callbackHdr: ITEMHDR = ITEMHDR { pszText: LPSTR_TEXTCALLBACKW, iImage: I_IMAGECALLBACK };
    let mut is_subitem_invalid: BOOL = FALSE;
    let mut dispInfo: NMLVDISPINFOW = core::mem::zeroed();
    let mut lpItem: *mut ITEM_INFO = core::ptr::null_mut();
    let mut pItemHdr: *mut ITEMHDR = core::ptr::null_mut();
    let mut hdpaSubItems: HDPA = core::ptr::null_mut();

    if lpLVItem.is_null() || (*lpLVItem).iItem < 0 || (*lpLVItem).iItem >= (*infoPtr).nItemCount {
        return FALSE;
    }

    if (*lpLVItem).mask == 0 {
        return TRUE;
    }

    if (*lpLVItem).iSubItem != 0 && ((*lpLVItem).mask & LVIF_STATE) != 0 {
        (*lpLVItem).state = 0;
    }

    /* a quick optimization if all we're asked is the focus state
     * these queries are worth optimising since they are common,
     * and can be answered in constant time, without the heavy accesses */
    if ((*lpLVItem).mask == LVIF_STATE)
        && ((*lpLVItem).stateMask == LVIS_FOCUSED)
        && (((*infoPtr).uCallbackMask & LVIS_FOCUSED) == 0)
    {
        (*lpLVItem).state = 0;
        if (*infoPtr).nFocusedItem == (*lpLVItem).iItem && (*lpLVItem).iSubItem == 0 {
            (*lpLVItem).state |= LVIS_FOCUSED;
        }
        return TRUE;
    }

    ZeroMemory!(&raw mut dispInfo as *mut ::core::ffi::c_void, core::mem::size_of::<NMLVDISPINFOW>());

    /* if the app stores all the data, handle it separately */
    if ((*infoPtr).dwStyle & LVS_OWNERDATA) != 0 {
        dispInfo.item.state = 0;

        /* apparently, we should not callback for lParam in LVS_OWNERDATA */
        if ((*lpLVItem).mask & !(LVIF_STATE | LVIF_PARAM)) != 0
            || (((*lpLVItem).mask & LVIF_STATE) != 0
                && ((*infoPtr).uCallbackMask & (*lpLVItem).stateMask) != 0)
        {
            let mut mask: UINT = (*lpLVItem).mask;

            /* NOTE: copy only fields which we _know_ are initialized, some apps
             *       depend on the uninitialized fields being 0 */
            dispInfo.item.mask = (*lpLVItem).mask & !LVIF_PARAM;
            dispInfo.item.iItem = (*lpLVItem).iItem;
            dispInfo.item.iSubItem = (*lpLVItem).iSubItem;
            if ((*lpLVItem).mask & LVIF_TEXT) != 0 {
                if ((*lpLVItem).mask & LVIF_NORECOMPUTE) != 0 {
                    /* reset mask */
                    dispInfo.item.mask &= !(LVIF_TEXT | LVIF_NORECOMPUTE);
                } else {
                    dispInfo.item.pszText = (*lpLVItem).pszText;
                    dispInfo.item.cchTextMax = (*lpLVItem).cchTextMax;
                }
            }
            if ((*lpLVItem).mask & LVIF_STATE) != 0 {
                dispInfo.item.stateMask = (*lpLVItem).stateMask & (*infoPtr).uCallbackMask;
            }
            /* could be zeroed on LVIF_NORECOMPUTE case */
            if dispInfo.item.mask != 0 {
                notify_dispinfoT(infoPtr, (LVN_GETDISPINFOW) as UINT, &raw mut dispInfo, isW);
                dispInfo.item.stateMask = (*lpLVItem).stateMask;
                if ((*lpLVItem).mask & (LVIF_GROUPID | LVIF_COLUMNS)) != 0 {
                    /* full size structure expected - _WIN32IE >= 0x560 */
                    *lpLVItem = dispInfo.item;
                } else if ((*lpLVItem).mask & LVIF_INDENT) != 0 {
                    /* indent member expected - _WIN32IE >= 0x300 */
                    memcpy(
                        lpLVItem as *mut ::core::ffi::c_void,
                        &raw mut dispInfo.item as *const ::core::ffi::c_void,
                        core::mem::offset_of!(LVITEMW, iGroupId),
                    );
                } else {
                    /* minimal structure expected */
                    memcpy(
                        lpLVItem as *mut ::core::ffi::c_void,
                        &raw mut dispInfo.item as *const ::core::ffi::c_void,
                        core::mem::offset_of!(LVITEMW, iIndent),
                    );
                }
                (*lpLVItem).mask = mask;
            }
        }

        /* make sure lParam is zeroed out */
        if ((*lpLVItem).mask & LVIF_PARAM) != 0 {
            (*lpLVItem).lParam = 0;
        }

        /* callback marked pointer required here */
        if ((*lpLVItem).mask & LVIF_TEXT) != 0 && ((*lpLVItem).mask & LVIF_NORECOMPUTE) != 0 {
            (*lpLVItem).pszText = LPSTR_TEXTCALLBACKW;
        }

        /* we store only a little state, so if we're not asked, we're done */
        if ((*lpLVItem).mask & LVIF_STATE) == 0 || (*lpLVItem).iSubItem != 0 {
            return TRUE;
        }

        /* if focus is handled by us, report it */
        if ((*lpLVItem).stateMask & !(*infoPtr).uCallbackMask & LVIS_FOCUSED) != 0 {
            (*lpLVItem).state &= !LVIS_FOCUSED;
            if (*infoPtr).nFocusedItem == (*lpLVItem).iItem {
                (*lpLVItem).state |= LVIS_FOCUSED;
            }
        }

        /* and do the same for selection, if we handle it */
        if ((*lpLVItem).stateMask & !(*infoPtr).uCallbackMask & LVIS_SELECTED) != 0 {
            (*lpLVItem).state &= !LVIS_SELECTED;
            if ranges_contain((*infoPtr).selectionRanges, (*lpLVItem).iItem) != 0 {
                (*lpLVItem).state |= LVIS_SELECTED;
            }
        }

        return TRUE;
    }

    /* find the item and subitem structures before we proceed */
    hdpaSubItems = DPA_GetPtr((*infoPtr).hdpaItems, (*lpLVItem).iItem) as HDPA;
    lpItem = DPA_GetPtr(hdpaSubItems, 0) as *mut ITEM_INFO;
    assert!((!lpItem.is_null()) as ::core::ffi::c_int);

    if (*lpLVItem).iSubItem != 0 {
        let mut lpSubItem: *mut SUBITEM_INFO = LISTVIEW_GetSubItemPtr(hdpaSubItems, (*lpLVItem).iSubItem);
        if !lpSubItem.is_null() {
            pItemHdr = &raw mut (*lpSubItem).hdr;
        } else {
            pItemHdr = &raw mut callbackHdr;
            is_subitem_invalid = TRUE;
        }
    } else {
        pItemHdr = &raw mut (*lpItem).hdr;
    }

    /* Do we need to query the state from the app? */
    if ((*lpLVItem).mask & LVIF_STATE) != 0
        && (*infoPtr).uCallbackMask != 0
        && ((*lpLVItem).iSubItem == 0 || is_subitem_invalid != 0)
    {
        dispInfo.item.mask |= LVIF_STATE;
        dispInfo.item.stateMask = (*infoPtr).uCallbackMask;
    }

    /* Do we need to enquire about the image? */
    if ((*lpLVItem).mask & LVIF_IMAGE) != 0
        && (*pItemHdr).iImage == I_IMAGECALLBACK
        && ((*lpLVItem).iSubItem == 0 || ((*infoPtr).dwLvExStyle & LVS_EX_SUBITEMIMAGES) != 0)
    {
        dispInfo.item.mask |= LVIF_IMAGE;
        dispInfo.item.iImage = I_IMAGECALLBACK;
    }

    /* Only items support indentation */
    if ((*lpLVItem).mask & LVIF_INDENT) != 0
        && (*lpItem).iIndent == I_INDENTCALLBACK
        && (*lpLVItem).iSubItem == 0
    {
        dispInfo.item.mask |= LVIF_INDENT;
        dispInfo.item.iIndent = I_INDENTCALLBACK;
    }

    /* Apps depend on calling back for text if it is NULL or LPSTR_TEXTCALLBACKW */
    if ((*lpLVItem).mask & LVIF_TEXT) != 0
        && ((*lpLVItem).mask & LVIF_NORECOMPUTE) == 0
        && is_text((*pItemHdr).pszText as LPCWSTR) == 0
    {
        dispInfo.item.mask |= LVIF_TEXT;
        dispInfo.item.pszText = (*lpLVItem).pszText;
        dispInfo.item.cchTextMax = (*lpLVItem).cchTextMax;
        if !dispInfo.item.pszText.is_null() && dispInfo.item.cchTextMax > 0 {
            *dispInfo.item.pszText = '\0' as u16;
        }
    }

    /* If we don't have all the requested info, query the application */
    if dispInfo.item.mask != 0 {
        dispInfo.item.iItem = (*lpLVItem).iItem;
        dispInfo.item.iSubItem = (*lpLVItem).iSubItem;
        dispInfo.item.lParam = (*lpItem).lParam;
        notify_dispinfoT(infoPtr, (LVN_GETDISPINFOW) as UINT, &raw mut dispInfo, isW);
    }

    /* we should not store values for subitems */
    if (*lpLVItem).iSubItem != 0 {
        dispInfo.item.mask &= !LVIF_DI_SETITEM;
    }

    /* Now, handle the iImage field */
    if (dispInfo.item.mask & LVIF_IMAGE) != 0 {
        (*lpLVItem).iImage = dispInfo.item.iImage;
        if (dispInfo.item.mask & LVIF_DI_SETITEM) != 0 && (*pItemHdr).iImage == I_IMAGECALLBACK {
            (*pItemHdr).iImage = dispInfo.item.iImage;
        }
    } else if ((*lpLVItem).mask & LVIF_IMAGE) != 0 {
        if (*lpLVItem).iSubItem == 0 || ((*infoPtr).dwLvExStyle & LVS_EX_SUBITEMIMAGES) != 0 {
            (*lpLVItem).iImage = (*pItemHdr).iImage;
        } else {
            (*lpLVItem).iImage = 0;
        }
    }

    /* The pszText field */
    if (dispInfo.item.mask & LVIF_TEXT) != 0 {
        if (dispInfo.item.mask & LVIF_DI_SETITEM) != 0 && !(*pItemHdr).pszText.is_null() {
            textsetptrT(&raw mut (*pItemHdr).pszText, dispInfo.item.pszText as LPCWSTR, isW);
        }

        (*lpLVItem).pszText = dispInfo.item.pszText;
    } else if ((*lpLVItem).mask & LVIF_TEXT) != 0 {
        /* if LVN_GETDISPINFO's disabled with LVIF_NORECOMPUTE return callback placeholder */
        if isW != 0 || is_text((*pItemHdr).pszText as LPCWSTR) == 0 {
            (*lpLVItem).pszText = (*pItemHdr).pszText;
        } else {
            textcpynT(
                (*lpLVItem).pszText,
                isW,
                (*pItemHdr).pszText as LPCWSTR,
                TRUE,
                (*lpLVItem).cchTextMax as INT,
            );
        }
    }

    /* Next is the lParam field */
    if (dispInfo.item.mask & LVIF_PARAM) != 0 {
        (*lpLVItem).lParam = dispInfo.item.lParam;
        if (dispInfo.item.mask & LVIF_DI_SETITEM) != 0 {
            (*lpItem).lParam = dispInfo.item.lParam;
        }
    } else if ((*lpLVItem).mask & LVIF_PARAM) != 0 {
        (*lpLVItem).lParam = (*lpItem).lParam;
    }

    /* if this is a subitem, we're done */
    if (*lpLVItem).iSubItem != 0 {
        return TRUE;
    }

    /* ... the state field (this one is different due to uCallbackmask) */
    if ((*lpLVItem).mask & LVIF_STATE) != 0 {
        (*lpLVItem).state = (*lpItem).state & (*lpLVItem).stateMask;
        if (dispInfo.item.mask & LVIF_STATE) != 0 {
            (*lpLVItem).state &= !dispInfo.item.stateMask;
            (*lpLVItem).state |= dispInfo.item.state & dispInfo.item.stateMask;
        }
        if ((*lpLVItem).stateMask & !(*infoPtr).uCallbackMask & LVIS_FOCUSED) != 0 {
            (*lpLVItem).state &= !LVIS_FOCUSED;
            if (*infoPtr).nFocusedItem == (*lpLVItem).iItem {
                (*lpLVItem).state |= LVIS_FOCUSED;
            }
        }
        if ((*lpLVItem).stateMask & !(*infoPtr).uCallbackMask & LVIS_SELECTED) != 0 {
            (*lpLVItem).state &= !LVIS_SELECTED;
            if ranges_contain((*infoPtr).selectionRanges, (*lpLVItem).iItem) != 0 {
                (*lpLVItem).state |= LVIS_SELECTED;
            }
        }
    }

    /* and last, but not least, the indent field */
    if (dispInfo.item.mask & LVIF_INDENT) != 0 {
        (*lpLVItem).iIndent = dispInfo.item.iIndent;
        if (dispInfo.item.mask & LVIF_DI_SETITEM) != 0 && (*lpItem).iIndent == I_INDENTCALLBACK {
            (*lpItem).iIndent = dispInfo.item.iIndent;
        }
    } else if ((*lpLVItem).mask & LVIF_INDENT) != 0 {
        (*lpLVItem).iIndent = (*lpItem).iIndent;
    }

    return TRUE;
}

unsafe extern "C" fn LISTVIEW_GetItemExtT(infoPtr: *const LISTVIEW_INFO, lpLVItem: LPLVITEMW, isW: BOOL) -> BOOL {
    let mut pszText: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut bResult: BOOL = 0;

    if lpLVItem.is_null()
        || (*lpLVItem).iItem < 0
        || (*lpLVItem).iItem >= (*infoPtr).nItemCount
    {
        return FALSE;
    }

    pszText = (*lpLVItem).pszText;
    bResult = LISTVIEW_GetItemT(infoPtr, lpLVItem, isW);
    if bResult != 0 && (*lpLVItem).mask & LVIF_TEXT as UINT != 0 && (*lpLVItem).pszText != pszText {
        if (*lpLVItem).pszText != LPSTR_TEXTCALLBACKW {
            textcpynT(
                pszText,
                isW,
                (*lpLVItem).pszText as LPCWSTR,
                isW,
                (*lpLVItem).cchTextMax as INT,
            );
        } else {
            pszText = LPSTR_TEXTCALLBACKW;
        }
    }
    (*lpLVItem).pszText = pszText;

    return bResult;
}

unsafe extern "C" fn LISTVIEW_GetItemPosition(infoPtr: *const LISTVIEW_INFO, nItem: INT, lpptPosition: LPPOINT) -> BOOL {
    let mut Origin: POINT = POINT { x: 0, y: 0 };

    if lpptPosition.is_null() || nItem < 0 || nItem >= (*infoPtr).nItemCount {
        return FALSE;
    }

    LISTVIEW_GetOrigin(infoPtr, core::ptr::addr_of_mut!(Origin));
    LISTVIEW_GetItemOrigin(infoPtr, nItem, lpptPosition);

    if (*infoPtr).uView == LV_VIEW_ICON {
        (*lpptPosition).x += ((*infoPtr).nItemWidth as INT - (*infoPtr).iconSize.cx as INT) / 2;
        (*lpptPosition).y += ICON_TOP_PADDING;
    }
    (*lpptPosition).x += Origin.x;
    (*lpptPosition).y += Origin.y;

    return TRUE;
}

unsafe extern "C" fn LISTVIEW_GetItemRect(
    mut infoPtr: *const LISTVIEW_INFO,
    mut nItem: INT,
    mut lprc: LPRECT,
) -> BOOL {
    let mut szDispText: [WCHAR; (DISP_TEXT_SIZE) as usize] = [0; (DISP_TEXT_SIZE) as usize];
    let mut doLabel: BOOL = TRUE;
    let mut oversizedBox: BOOL = FALSE;
    let mut Position: POINT = tagPOINT { x: 0, y: 0 };
    let mut Origin: POINT = tagPOINT { x: 0, y: 0 };
    let mut lvItem: LVITEMW = tagLVITEMW {
        mask: 0,
        iItem: 0,
        iSubItem: 0,
        state: 0,
        stateMask: 0,
        pszText: ::core::ptr::null_mut::<WCHAR>(),
        cchTextMax: 0,
        iImage: 0,
        lParam: 0,
        iIndent: 0,
        iGroupId: 0,
        cColumns: 0,
        puColumns: ::core::ptr::null_mut::<::core::ffi::c_uint>(),
    };
    let mut mode: LONG = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_GetItemRect\0" as *const u8 as *const ::core::ffi::c_char,
            7201 as ::core::ffi::c_int,
            b"(hwnd=%p, nItem=%d, lprc=%p)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*infoPtr).hwndSelf,
            nItem,
            lprc,
        );
    }
    if lprc.is_null() || nItem < 0 || nItem >= (*infoPtr).nItemCount {
        return FALSE;
    }
    LISTVIEW_GetOrigin(infoPtr, &raw mut Origin);
    LISTVIEW_GetItemOrigin(infoPtr, nItem, &raw mut Position);

    /* Be smart and try to figure out the minimum we have to do */
    if (*lprc).left == (LVIR_ICON as i32) {
        doLabel = FALSE;
    }
    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD && (*lprc).left == (LVIR_BOUNDS as i32) {
        doLabel = FALSE;
    }
    if (*infoPtr).uView == LV_VIEW_ICON as DWORD
        && (*lprc).left != (LVIR_ICON as i32)
        && (*infoPtr).bFocus != 0
        && LISTVIEW_GetItemState(infoPtr, nItem, LVIS_FOCUSED) != 0
    {
        oversizedBox = TRUE;
    }

    /* get what we need from the item before hand, so we make
     * only one request. This can speed up things, if data
     * is stored on the app side */
    lvItem.mask = 0 as UINT;
    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
        lvItem.mask |= LVIF_INDENT as UINT;
    }
    if doLabel != 0 {
        lvItem.mask |= LVIF_TEXT as UINT;
    }
    lvItem.iItem = nItem as ::core::ffi::c_int;
    lvItem.iSubItem = 0 as ::core::ffi::c_int;
    lvItem.pszText = &raw mut szDispText as *mut WCHAR as LPWSTR;
    lvItem.cchTextMax = DISP_TEXT_SIZE as ::core::ffi::c_int;
    if lvItem.mask != 0 && LISTVIEW_GetItemW(infoPtr, &raw mut lvItem) == 0 {
        return FALSE;
    }
    /* we got the state already up, simulate it here, to avoid a reget */
    if (*infoPtr).uView == LV_VIEW_ICON as DWORD && ((*lprc).left != (LVIR_ICON as i32)) {
        lvItem.mask |= LVIF_STATE as UINT;
        lvItem.stateMask = LVIS_FOCUSED as UINT;
        lvItem.state = (if oversizedBox != 0 {
            LVIS_FOCUSED
        } else {
            0
        }) as UINT;
    }

    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD
        && (*infoPtr).dwLvExStyle & LVS_EX_FULLROWSELECT as DWORD != 0
        && (*lprc).left == (LVIR_SELECTBOUNDS as i32)
    {
        (*lprc).left = LVIR_BOUNDS as LONG;
    }

    mode = (*lprc).left;
    match ((*lprc).left) as u32 {
        LVIR_ICON => {
            LISTVIEW_GetItemMetrics(
                infoPtr,
                &raw mut lvItem,
                ::core::ptr::null_mut::<tagRECT>(),
                ::core::ptr::null_mut::<tagRECT>(),
                lprc,
                ::core::ptr::null_mut::<tagRECT>(),
                ::core::ptr::null_mut::<tagRECT>(),
            );
        }
        LVIR_LABEL => {
            LISTVIEW_GetItemMetrics(
                infoPtr,
                &raw mut lvItem,
                ::core::ptr::null_mut::<tagRECT>(),
                ::core::ptr::null_mut::<tagRECT>(),
                ::core::ptr::null_mut::<tagRECT>(),
                ::core::ptr::null_mut::<tagRECT>(),
                lprc,
            );
        }
        LVIR_BOUNDS => {
            LISTVIEW_GetItemMetrics(
                infoPtr,
                &raw mut lvItem,
                lprc,
                ::core::ptr::null_mut::<tagRECT>(),
                ::core::ptr::null_mut::<tagRECT>(),
                ::core::ptr::null_mut::<tagRECT>(),
                ::core::ptr::null_mut::<tagRECT>(),
            );
        }
        LVIR_SELECTBOUNDS => {
            LISTVIEW_GetItemMetrics(
                infoPtr,
                &raw mut lvItem,
                ::core::ptr::null_mut::<tagRECT>(),
                lprc,
                ::core::ptr::null_mut::<tagRECT>(),
                ::core::ptr::null_mut::<tagRECT>(),
                ::core::ptr::null_mut::<tagRECT>(),
            );
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/listview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"LISTVIEW_GetItemRect\0" as *const u8 as *const ::core::ffi::c_char,
                    7257 as ::core::ffi::c_int,
                    b"Unknown value: %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                    (*lprc).left,
                );
            }
            return FALSE;
        }
    }

    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
        if mode != (LVIR_BOUNDS as i32) {
            OffsetRect(
                lprc,
                Origin.x as ::core::ffi::c_int
                    + (*LISTVIEW_GetColumnInfo(infoPtr, 0 as INT)).rcHeader.left
                        as ::core::ffi::c_int,
                Position.y as ::core::ffi::c_int + Origin.y as ::core::ffi::c_int,
            );
        } else {
            OffsetRect(
                lprc,
                Origin.x as ::core::ffi::c_int,
                Position.y as ::core::ffi::c_int + Origin.y as ::core::ffi::c_int,
            );
        }
    } else {
        OffsetRect(
            lprc,
            Position.x as ::core::ffi::c_int + Origin.x as ::core::ffi::c_int,
            Position.y as ::core::ffi::c_int + Origin.y as ::core::ffi::c_int,
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
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_GetItemRect\0" as *const u8 as *const ::core::ffi::c_char,
            7272 as ::core::ffi::c_int,
            b" rect=%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_rect(lprc as *const RECT),
        );
    }
    return TRUE;
}

unsafe extern "C" fn LISTVIEW_GetSubItemRect(infoPtr: *const LISTVIEW_INFO, item: INT, lprc: LPRECT) -> BOOL {
    let mut rect: RECT = RECT {
        left: 0 as LONG,
        top: 0 as LONG,
        right: 0 as LONG,
        bottom: 0 as LONG,
    };
    let mut origin: POINT = POINT { x: 0, y: 0 };
    let mut y: INT = 0;

    if lprc.is_null() {
        return FALSE;
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
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_GetSubItemRect\0" as *const u8 as *const ::core::ffi::c_char,
            7302 as ::core::ffi::c_int,
            b"(item=%d, subitem=%d, type=%d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            item,
            (*lprc).top,
            (*lprc).left,
        );
    }

    /* Subitem of '0' means item itself, and this works for all control view modes */
    if (*lprc).top == 0 as LONG {
        return LISTVIEW_GetItemRect(infoPtr, item, lprc);
    }

    if (*infoPtr).uView != LV_VIEW_DETAILS as DWORD {
        return FALSE;
    }

    LISTVIEW_GetOrigin(infoPtr, core::ptr::addr_of_mut!(origin));
    /* this works for any item index, no matter if it exists or not */
    y = (item as LONG * (*infoPtr).nItemHeight as LONG + origin.y) as INT;

    if !(*infoPtr).hwndHeader.is_null()
        && SendMessageW(
            (*infoPtr).hwndHeader,
            HDM_GETITEMRECT as UINT,
            (*lprc).top as WPARAM,
            core::ptr::addr_of_mut!(rect) as LPARAM,
        ) != 0
    {
        rect.top = 0 as LONG;
        rect.bottom = (*infoPtr).nItemHeight as LONG;
    } else {
        /* Native implementation is broken for this case and garbage is left for left and right fields,
           we zero them to get predictable output */
        (*lprc).left = 0 as LONG;
        (*lprc).right = 0 as LONG;
        (*lprc).top = 0 as LONG;
        (*lprc).bottom = (*infoPtr).nItemHeight as LONG;
        OffsetRect(lprc, origin.x as ::core::ffi::c_int, y as ::core::ffi::c_int);

        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/listview.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"LISTVIEW_GetSubItemRect\0" as *const u8 as *const ::core::ffi::c_char,
                7325 as ::core::ffi::c_int,
                b"return rect %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                wine_dbgstr_rect(lprc as *const RECT),
            );
        }
        return TRUE;
    }

    match ((*lprc).left) as u32 {
        LVIR_ICON => {
            /* it doesn't matter if main item actually has an icon, if imagelist is set icon width is returned */
            if !(*infoPtr).himlSmall.is_null() {
                rect.right = rect.left + (*infoPtr).iconSize.cx;
            } else {
                rect.right = rect.left;
            }
            rect.bottom = rect.top + (*infoPtr).iconSize.cy;
        }
        LVIR_LABEL | LVIR_BOUNDS => {}
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/listview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"LISTVIEW_GetSubItemRect\0" as *const u8 as *const ::core::ffi::c_char,
                    7347 as ::core::ffi::c_int,
                    b"Unknown bounds=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
                    (*lprc).left,
                );
            }
            return FALSE;
        }
    }

    OffsetRect(
        core::ptr::addr_of_mut!(rect),
        origin.x as ::core::ffi::c_int,
        y as ::core::ffi::c_int,
    );
    *lprc = rect;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_2,
            __dbch_2,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_GetSubItemRect\0" as *const u8 as *const ::core::ffi::c_char,
            7353 as ::core::ffi::c_int,
            b"return rect %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_rect(lprc as *const RECT),
        );
    }

    return TRUE;
}

unsafe extern "C" fn LISTVIEW_GetItemSpacing(infoPtr: *const LISTVIEW_INFO, bSmall: BOOL) -> LONG {
    let mut lResult: LONG = 0;

    if bSmall == 0 {
        lResult = MAKELONG!((*infoPtr).iconSpacing.cx as WORD, (*infoPtr).iconSpacing.cy as WORD);
    } else {
        if (*infoPtr).uView == LV_VIEW_ICON {
            lResult = MAKELONG!(
                DEFAULT_COLUMN_WIDTH as WORD,
                (GetSystemMetrics(49 /* SM_CXSMICON */) + HEIGHT_PADDING) as WORD,
            );
        } else {
            lResult = MAKELONG!((*infoPtr).nItemWidth as WORD, (*infoPtr).nItemHeight as WORD);
        }
    }

    return lResult;
}

unsafe extern "C" fn LISTVIEW_GetItemState(infoPtr: *const LISTVIEW_INFO, nItem: INT, uMask: UINT) -> UINT {
    let mut lvItem: LVITEMW = core::mem::zeroed();
    if nItem < 0 || nItem >= (*infoPtr).nItemCount {
        return 0;
    }

    lvItem.iItem = nItem;
    lvItem.iSubItem = 0;
    lvItem.mask = LVIF_STATE;
    lvItem.stateMask = uMask;
    if LISTVIEW_GetItemW(infoPtr, core::ptr::addr_of_mut!(lvItem)) == 0 {
        return 0;
    }

    return lvItem.state & uMask;
}

unsafe extern "C" fn LISTVIEW_GetItemTextT(infoPtr: *const LISTVIEW_INFO, nItem: INT, lpLVItem: LPLVITEMW, isW: BOOL) -> INT {
    if lpLVItem.is_null() || nItem < 0 || nItem >= (*infoPtr).nItemCount {
        return 0;
    }

    (*lpLVItem).mask = LVIF_TEXT;
    (*lpLVItem).iItem = nItem;
    if LISTVIEW_GetItemExtT(infoPtr, lpLVItem, isW) == 0 {
        return 0;
    }

    return textlenT((*lpLVItem).pszText as LPCWSTR, isW) as INT;
}

unsafe extern "C" fn LISTVIEW_GetNextItem(infoPtr: *const LISTVIEW_INFO, mut nItem: INT, mut uFlags: UINT) -> INT {
    let mut uMask: UINT = 0 as UINT;
    let mut lvFindInfo: LVFINDINFOW = core::mem::zeroed();
    let mut nCountPerColumn: INT = 0;
    let mut i: INT;

    if nItem < -(1 as INT) || nItem >= (*infoPtr).nItemCount {
        return -(1 as INT);
    }

    core::ptr::write_bytes(
        &raw mut lvFindInfo as *mut u8,
        0,
        core::mem::size_of::<LVFINDINFOW>(),
    );

    if uFlags & (LVNI_CUT as UINT) != 0 {
        uMask |= LVIS_CUT as UINT;
    }

    if uFlags & (LVNI_DROPHILITED as UINT) != 0 {
        uMask |= LVIS_DROPHILITED as UINT;
    }

    if uFlags & (LVNI_FOCUSED as UINT) != 0 {
        uMask |= LVIS_FOCUSED as UINT;
    }

    if uFlags & (LVNI_SELECTED as UINT) != 0 {
        uMask |= LVIS_SELECTED as UINT;
    }

    if uFlags & (LVNI_FOCUSED as UINT) != 0 {
        if (LISTVIEW_GetItemState(infoPtr, (*infoPtr).nFocusedItem, uMask) & uMask) != uMask {
            return -(1 as INT);
        }
        return if (*infoPtr).nFocusedItem == nItem {
            -(1 as INT)
        } else {
            (*infoPtr).nFocusedItem
        };
    }

    if uFlags & (LVNI_ABOVE as UINT) != 0 {
        if (*infoPtr).uView == (LV_VIEW_LIST as DWORD) || (*infoPtr).uView == (LV_VIEW_DETAILS as DWORD) {
            while nItem >= (0 as INT) {
                nItem -= 1;
                if (LISTVIEW_GetItemState(infoPtr, nItem, uMask) & uMask) == uMask {
                    return nItem;
                }
            }
        } else {
            lvFindInfo.flags = LVFI_NEARESTXY as UINT;
            lvFindInfo.vkDirection = VK_UP as UINT;
            LISTVIEW_GetItemPosition(infoPtr, nItem, &raw mut lvFindInfo.pt);
            loop {
                nItem = LISTVIEW_FindItemW(infoPtr, nItem, &raw mut lvFindInfo);
                if !(nItem != -(1 as INT)) {
                    break;
                }
                if (LISTVIEW_GetItemState(infoPtr, nItem, uMask) & uMask) == uMask {
                    return nItem;
                }
            }
        }
    } else if uFlags & (LVNI_BELOW as UINT) != 0 {
        if (*infoPtr).uView == (LV_VIEW_LIST as DWORD) || (*infoPtr).uView == (LV_VIEW_DETAILS as DWORD) {
            while nItem < (*infoPtr).nItemCount {
                nItem += 1;
                if (LISTVIEW_GetItemState(infoPtr, nItem, uMask) & uMask) == uMask {
                    return nItem;
                }
            }
        } else {
            lvFindInfo.flags = LVFI_NEARESTXY as UINT;
            lvFindInfo.vkDirection = VK_DOWN as UINT;
            LISTVIEW_GetItemPosition(infoPtr, nItem, &raw mut lvFindInfo.pt);
            loop {
                nItem = LISTVIEW_FindItemW(infoPtr, nItem, &raw mut lvFindInfo);
                if !(nItem != -(1 as INT)) {
                    break;
                }
                if (LISTVIEW_GetItemState(infoPtr, nItem, uMask) & uMask) == uMask {
                    return nItem;
                }
            }
        }
    } else if uFlags & (LVNI_TOLEFT as UINT) != 0 {
        if (*infoPtr).uView == (LV_VIEW_LIST as DWORD) {
            nCountPerColumn = LISTVIEW_GetCountPerColumn(infoPtr);
            while nItem - nCountPerColumn >= (0 as INT) {
                nItem -= nCountPerColumn;
                if (LISTVIEW_GetItemState(infoPtr, nItem, uMask) & uMask) == uMask {
                    return nItem;
                }
            }
        } else if (*infoPtr).uView == (LV_VIEW_SMALLICON as DWORD) || (*infoPtr).uView == (LV_VIEW_ICON as DWORD) {
            lvFindInfo.flags = LVFI_NEARESTXY as UINT;
            lvFindInfo.vkDirection = VK_LEFT as UINT;
            LISTVIEW_GetItemPosition(infoPtr, nItem, &raw mut lvFindInfo.pt);
            loop {
                nItem = LISTVIEW_FindItemW(infoPtr, nItem, &raw mut lvFindInfo);
                if !(nItem != -(1 as INT)) {
                    break;
                }
                if (LISTVIEW_GetItemState(infoPtr, nItem, uMask) & uMask) == uMask {
                    return nItem;
                }
            }
        }
    } else if uFlags & (LVNI_TORIGHT as UINT) != 0 {
        if (*infoPtr).uView == (LV_VIEW_LIST as DWORD) {
            nCountPerColumn = LISTVIEW_GetCountPerColumn(infoPtr);
            while nItem + nCountPerColumn < (*infoPtr).nItemCount {
                nItem += nCountPerColumn;
                if (LISTVIEW_GetItemState(infoPtr, nItem, uMask) & uMask) == uMask {
                    return nItem;
                }
            }
        } else if (*infoPtr).uView == (LV_VIEW_SMALLICON as DWORD) || (*infoPtr).uView == (LV_VIEW_ICON as DWORD) {
            lvFindInfo.flags = LVFI_NEARESTXY as UINT;
            lvFindInfo.vkDirection = VK_RIGHT as UINT;
            LISTVIEW_GetItemPosition(infoPtr, nItem, &raw mut lvFindInfo.pt);
            loop {
                nItem = LISTVIEW_FindItemW(infoPtr, nItem, &raw mut lvFindInfo);
                if !(nItem != -(1 as INT)) {
                    break;
                }
                if (LISTVIEW_GetItemState(infoPtr, nItem, uMask) & uMask) == uMask {
                    return nItem;
                }
            }
        }
    } else {
        nItem += 1;
        i = nItem;
        while i < (*infoPtr).nItemCount {
            if (LISTVIEW_GetItemState(infoPtr, i, uMask) & uMask) == uMask {
                return i;
            }
            i += 1;
        }
    }

    return -(1 as INT);
}

unsafe extern "C" fn LISTVIEW_GetOrigin(infoPtr: *const LISTVIEW_INFO, lpptOrigin: LPPOINT) {
    let mut nHorzPos: INT = 0 as INT;
    let mut nVertPos: INT = 0 as INT;
    let mut scrollInfo: SCROLLINFO = core::mem::zeroed();

    scrollInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    scrollInfo.fMask = SIF_POS as UINT;

    if GetScrollInfo(
        (*infoPtr).hwndSelf,
        SB_HORZ as INT,
        core::ptr::addr_of_mut!(scrollInfo),
    ) != 0
    {
        nHorzPos = scrollInfo.nPos as INT;
    }
    if GetScrollInfo(
        (*infoPtr).hwndSelf,
        SB_VERT as INT,
        core::ptr::addr_of_mut!(scrollInfo),
    ) != 0
    {
        nVertPos = scrollInfo.nPos as INT;
    }

    (*lpptOrigin).x = (*infoPtr).rcList.left;
    (*lpptOrigin).y = (*infoPtr).rcList.top;
    if (*infoPtr).uView == LV_VIEW_LIST as DWORD {
        nHorzPos *= (*infoPtr).nItemWidth;
    } else if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
        nVertPos *= (*infoPtr).nItemHeight;
    }

    (*lpptOrigin).x -= nHorzPos;
    (*lpptOrigin).y -= nVertPos;
}

unsafe extern "C" fn LISTVIEW_GetStringWidthT(
    mut infoPtr: *const LISTVIEW_INFO,
    mut lpszText: LPCWSTR,
    mut isW: BOOL,
) -> INT {
    let mut stringSize: SIZE = SIZE { cx: 0, cy: 0 };
    stringSize.cx = 0;
    if is_text(lpszText) != 0 {
        let mut hFont: HFONT = if !(*infoPtr).hFont.is_null() {
            (*infoPtr).hFont
        } else {
            (*infoPtr).hDefaultFont
        };
        let mut hdc: HDC = GetDC((*infoPtr).hwndSelf);
        let mut hOldFont: HFONT = SelectObject(hdc, hFont as HGDIOBJ) as HFONT;
        if isW != 0 {
            GetTextExtentPointW(
                hdc,
                lpszText,
                lstrlenW(lpszText),
                core::ptr::addr_of_mut!(stringSize),
            );
        } else {
            GetTextExtentPointA(
                hdc,
                lpszText as LPCSTR,
                lstrlenA(lpszText as LPCSTR),
                core::ptr::addr_of_mut!(stringSize),
            );
        }
        SelectObject(hdc, hOldFont as HGDIOBJ);
        ReleaseDC((*infoPtr).hwndSelf, hdc);
    }
    return stringSize.cx as INT;
}

unsafe extern "C" fn LISTVIEW_HitTest(infoPtr: *const LISTVIEW_INFO, lpht: LPLVHITTESTINFO, subitem: BOOL, select: BOOL) -> INT {
    let mut szDispText: [WCHAR; (DISP_TEXT_SIZE) as usize] = [0 as WCHAR; (DISP_TEXT_SIZE) as usize];
    let mut rcBox: RECT = core::mem::zeroed::<RECT>();
    let mut rcBounds: RECT = core::mem::zeroed::<RECT>();
    let mut rcState: RECT = core::mem::zeroed::<RECT>();
    let mut rcIcon: RECT = core::mem::zeroed::<RECT>();
    let mut rcLabel: RECT = core::mem::zeroed::<RECT>();
    let mut rcSearch: RECT = core::mem::zeroed::<RECT>();
    let mut Origin: POINT = core::mem::zeroed::<POINT>();
    let mut Position: POINT = core::mem::zeroed::<POINT>();
    let mut opt: POINT = core::mem::zeroed::<POINT>();
    let mut is_fullrow: BOOL = 0;
    let mut lvItem: LVITEMW = core::mem::zeroed::<LVITEMW>();
    let mut i: ITERATOR = core::mem::zeroed::<ITERATOR>();
    let mut iItem: INT = 0;

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/comctl32/listview.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_HitTest\0" as *const u8 as *const ::core::ffi::c_char,
            7756 as ::core::ffi::c_int,
            b"(pt=%s, subitem=%d, select=%d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_point(&raw mut (*lpht).pt),
            subitem,
            select,
        );
    }

    (*lpht).flags = 0 as UINT;
    (*lpht).iItem = -(1 as INT);
    if subitem != 0 {
        (*lpht).iSubItem = 0 as INT;
    }

    LISTVIEW_GetOrigin(infoPtr, &raw mut Origin);

    /* set whole list relation flags */
    if subitem != 0 && (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
        /* LVM_SUBITEMHITTEST checks left bound of possible client area */
        if (*infoPtr).rcList.left > (*lpht).pt.x && Origin.x < (*lpht).pt.x {
            (*lpht).flags |= LVHT_TOLEFT as UINT;
        }

        if (*lpht).pt.y < (*infoPtr).rcList.top && (*lpht).pt.y >= 0 as INT {
            opt.y = (*lpht).pt.y + (*infoPtr).rcList.top;
        } else {
            opt.y = (*lpht).pt.y;
        }

        if (*infoPtr).rcList.bottom < opt.y {
            (*lpht).flags |= LVHT_BELOW as UINT;
        }
    } else {
        if (*infoPtr).rcList.left > (*lpht).pt.x {
            (*lpht).flags |= LVHT_TOLEFT as UINT;
        } else if (*infoPtr).rcList.right < (*lpht).pt.x {
            (*lpht).flags |= LVHT_TORIGHT as UINT;
        }

        if (*infoPtr).rcList.top > (*lpht).pt.y {
            (*lpht).flags |= LVHT_ABOVE as UINT;
        } else if (*infoPtr).rcList.bottom < (*lpht).pt.y {
            (*lpht).flags |= LVHT_BELOW as UINT;
        }
    }

    /* even if item is invalid try to find subitem */
    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD && subitem != 0 {
        let mut pRect: *mut RECT = core::ptr::null_mut::<RECT>();
        let mut j: INT = 0;

        opt.x = (*lpht).pt.x - Origin.x;

        (*lpht).iSubItem = -(1 as INT);
        j = 0 as INT;
        while j < DPA_GetPtrCount((*infoPtr).hdpaColumns) {
            pRect = &raw mut (*LISTVIEW_GetColumnInfo(infoPtr, j)).rcHeader;

            if opt.x >= (*pRect).left && opt.x < (*pRect).right {
                (*lpht).iSubItem = j;
                break;
            }
            j += 1;
        }

        if ((*__wine_dbch___default).flags as ::core::ffi::c_int & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))) != 0 {
            ros_dbg_log(
                __WINE_DBCL_TRACE,
                __wine_dbch___default,
                b"dll/win32/comctl32/listview.c\0" as *const u8 as *const ::core::ffi::c_char,
                b"LISTVIEW_HitTest\0" as *const u8 as *const ::core::ffi::c_char,
                7811 as ::core::ffi::c_int,
                b"lpht->iSubItem=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
                (*lpht).iSubItem,
            );
        }

        /* if we're outside horizontal columns bounds there's nothing to test further */
        if (*lpht).iSubItem == -(1 as INT) {
            (*lpht).iItem = -(1 as INT);
            (*lpht).flags = LVHT_NOWHERE as UINT;
            return -(1 as INT);
        }
    }

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/comctl32/listview.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_HitTest\0" as *const u8 as *const ::core::ffi::c_char,
            7822 as ::core::ffi::c_int,
            b"lpht->flags=0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*lpht).flags,
        );
    }
    if (*lpht).flags != 0 {
        return -(1 as INT);
    }

    (*lpht).flags |= LVHT_NOWHERE as UINT;

    /* first deal with the large items */
    rcSearch.left = (*lpht).pt.x;
    rcSearch.top = (*lpht).pt.y;
    rcSearch.right = rcSearch.left + 1 as LONG;
    rcSearch.bottom = rcSearch.top + 1 as LONG;

    iterator_frameditems(&raw mut i, infoPtr, &raw mut rcSearch);
    iterator_next(&raw mut i); /* go to first item in the sequence */
    iItem = i.nItem;
    iterator_destroy(&raw mut i);

    if iItem == -(1 as INT) {
        return -(1 as INT);
    }

    lvItem.mask = (LVIF_STATE | LVIF_TEXT) as UINT;
    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
        lvItem.mask |= LVIF_INDENT as UINT;
    }
    lvItem.stateMask = LVIS_STATEIMAGEMASK as UINT;
    if (*infoPtr).uView == LV_VIEW_ICON as DWORD {
        lvItem.stateMask |= LVIS_FOCUSED as UINT;
    }
    lvItem.iItem = iItem;
    lvItem.iSubItem = if subitem != 0 {
        (*lpht).iSubItem
    } else {
        0 as INT
    };
    lvItem.pszText = &raw mut szDispText as *mut WCHAR as LPWSTR;
    lvItem.cchTextMax = DISP_TEXT_SIZE as INT;
    if LISTVIEW_GetItemW(infoPtr, &raw mut lvItem) == 0 {
        return -(1 as INT);
    }
    if (*infoPtr).bFocus == 0 {
        lvItem.state &= !(LVIS_FOCUSED as UINT);
    }

    LISTVIEW_GetItemMetrics(
        infoPtr,
        &raw mut lvItem,
        &raw mut rcBox,
        core::ptr::null_mut::<RECT>(),
        &raw mut rcIcon,
        &raw mut rcState,
        &raw mut rcLabel,
    );
    LISTVIEW_GetItemOrigin(infoPtr, iItem, &raw mut Position);
    opt.x = (*lpht).pt.x - Position.x - Origin.x;

    if (*lpht).pt.y < (*infoPtr).rcList.top && (*lpht).pt.y >= 0 as INT {
        opt.y = (*lpht).pt.y - Position.y - Origin.y + (*infoPtr).rcList.top;
    } else {
        opt.y = (*lpht).pt.y - Position.y - Origin.y;
    }

    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
        rcBounds = rcBox;
        if ((*infoPtr).dwLvExStyle & (LVS_EX_FULLROWSELECT as DWORD)) != 0 {
            opt.x = (*lpht).pt.x - Origin.x;
        }
    } else {
        UnionRect(
            &raw mut rcBounds,
            &raw mut rcIcon as LPCRECT,
            &raw mut rcLabel as LPCRECT,
        );
        UnionRect(
            &raw mut rcBounds,
            &raw mut rcBounds as LPCRECT,
            &raw mut rcState as LPCRECT,
        );
    }

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/comctl32/listview.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_HitTest\0" as *const u8 as *const ::core::ffi::c_char,
            7872 as ::core::ffi::c_int,
            b"rcBounds=%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_rect(&raw mut rcBounds),
        );
    }

    /* the item must be hit within its bounds */
    if PtInRect(&raw mut rcBounds as LPCRECT, opt) == 0 {
        return -(1 as INT);
    }

    /* That's a special case - row rectangle is used as item rectangle and
       returned flags contain all item parts. */
    is_fullrow = ((*infoPtr).uView == LV_VIEW_DETAILS as DWORD
        && (((*infoPtr).dwLvExStyle & (LVS_EX_FULLROWSELECT as DWORD)) != 0
            || ((*infoPtr).dwStyle & (LVS_OWNERDRAWFIXED as DWORD)) != 0)) as BOOL;

    if PtInRect(&raw mut rcIcon as LPCRECT, opt) != 0 {
        (*lpht).flags |= LVHT_ONITEMICON as UINT;
    } else if PtInRect(&raw mut rcLabel as LPCRECT, opt) != 0 {
        (*lpht).flags |= LVHT_ONITEMLABEL as UINT;
    } else if !(*infoPtr).himlState.is_null() && PtInRect(&raw mut rcState as LPCRECT, opt) != 0 {
        (*lpht).flags |= LVHT_ONITEMSTATEICON as UINT;
    }
    if is_fullrow != 0 && ((*lpht).flags & (LVHT_ONITEM as UINT)) == 0 {
        (*lpht).flags = (LVHT_ONITEM | LVHT_ABOVE) as UINT;
    }
    if ((*lpht).flags & (LVHT_ONITEM as UINT)) != 0 {
        (*lpht).flags &= !(LVHT_NOWHERE as UINT);
    }

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int))) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/comctl32/listview.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_HitTest\0" as *const u8 as *const ::core::ffi::c_char,
            7891 as ::core::ffi::c_int,
            b"lpht->flags=0x%x\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*lpht).flags,
        );
    }

    if select != 0 && is_fullrow == 0 {
        if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
            /* get main item bounds */
            lvItem.iSubItem = 0 as INT;
            LISTVIEW_GetItemMetrics(
                infoPtr,
                &raw mut lvItem,
                &raw mut rcBox,
                core::ptr::null_mut::<RECT>(),
                &raw mut rcIcon,
                &raw mut rcState,
                &raw mut rcLabel,
            );
            UnionRect(
                &raw mut rcBounds,
                &raw mut rcIcon as LPCRECT,
                &raw mut rcLabel as LPCRECT,
            );
            UnionRect(
                &raw mut rcBounds,
                &raw mut rcBounds as LPCRECT,
                &raw mut rcState as LPCRECT,
            );
        }
        if PtInRect(&raw mut rcBounds as LPCRECT, opt) == 0 {
            iItem = -(1 as INT);
        }
    }
    (*lpht).iItem = iItem;
    return (*lpht).iItem;
}

unsafe extern "C" fn LISTVIEW_InsertItemT(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut lpLVItem: *const LVITEMW,
    mut isW: BOOL,
) -> INT {
    let mut nItem: INT = 0;
    let mut hdpaSubItems: HDPA = core::ptr::null_mut();
    let mut nmlv: NMLISTVIEW = core::mem::zeroed();
    let mut lpItem: *mut ITEM_INFO = core::ptr::null_mut();
    let mut lpID: *mut ITEM_ID = core::ptr::null_mut();
    let mut is_sorted: BOOL = 0;
    let mut has_changed: BOOL = 0;
    let mut item: LVITEMW = core::mem::zeroed();
    let mut hwndSelf: HWND = (*infoPtr).hwndSelf;

    if (*infoPtr).dwStyle & LVS_OWNERDATA != 0 {
        let fresh = (*infoPtr).nItemCount;
        (*infoPtr).nItemCount += 1;
        return fresh;
    }

    /* make sure it's an item, and not a subitem; cannot insert a subitem */
    if lpLVItem.is_null() || (*lpLVItem).iSubItem != 0 {
        return -(1);
    }

    if is_assignable_item(lpLVItem, (*infoPtr).dwStyle as LONG) == 0 {
        return -(1);
    }

    lpItem = Alloc(core::mem::size_of::<ITEM_INFO>() as DWORD)
        as *mut ITEM_INFO;
    if lpItem.is_null() {
        return -(1);
    }

    'fail: {
        'undo: {
            /* insert item in listview control data structure */
            hdpaSubItems = DPA_Create(8);
            if hdpaSubItems.is_null() {
                break 'fail;
            }
            if DPA_SetPtr(hdpaSubItems, 0, lpItem as *mut core::ffi::c_void) == 0 {
                assert!(FALSE);
            }

            /* link with id struct */
            lpID = Alloc(core::mem::size_of::<ITEM_ID>() as DWORD)
                as *mut ITEM_ID;
            if lpID.is_null() {
                break 'fail;
            }
            (*lpItem).id = lpID;
            (*lpID).item = hdpaSubItems;
            (*lpID).id = get_next_itemid(infoPtr) as UINT;
            if DPA_InsertPtr(
                (*infoPtr).hdpaItemIds,
                (*infoPtr).nItemCount as core::ffi::c_int,
                lpID as *mut core::ffi::c_void,
            ) == -(1)
            {
                break 'fail;
            }

            is_sorted = (((*infoPtr).dwStyle & (LVS_SORTASCENDING | LVS_SORTDESCENDING) != 0)
                && ((*infoPtr).dwStyle & LVS_OWNERDRAWFIXED == 0)
                && (LPSTR_TEXTCALLBACKW != (*lpLVItem).pszText)) as core::ffi::c_int as BOOL;

            if (*lpLVItem).iItem < 0 && is_sorted == 0 {
                return -(1);
            }

            /* calculate new item index */
            if is_sorted != 0 {
                let mut hItem: HDPA = core::ptr::null_mut();
                let mut item_s: *mut ITEM_INFO = core::ptr::null_mut();
                let mut i: INT = 0;
                let mut cmpv: INT = 0;
                let mut textW: LPWSTR = core::ptr::null_mut();

                textW = textdupTtoW((*lpLVItem).pszText as LPCWSTR, isW);

                while i < (*infoPtr).nItemCount {
                    hItem = DPA_GetPtr((*infoPtr).hdpaItems, i) as HDPA;
                    item_s = DPA_GetPtr(hItem, 0) as *mut ITEM_INFO;

                    cmpv = textcmpWT(
                        (*item_s).hdr.pszText as LPCWSTR,
                        textW as LPCWSTR,
                        TRUE,
                    ) as INT;
                    if (*infoPtr).dwStyle & LVS_SORTDESCENDING != 0 {
                        cmpv *= -(1);
                    }

                    if cmpv >= 0 {
                        break;
                    }
                    i += 1;
                }

                textfreeT(textW, isW);

                nItem = i;
            } else {
                nItem = min((*lpLVItem).iItem, (*infoPtr).nItemCount);
            }

            nItem = DPA_InsertPtr((*infoPtr).hdpaItems, nItem, (hdpaSubItems).cast()) as INT;
            if nItem == -(1) {
                break 'fail;
            }
            (*infoPtr).nItemCount += 1;

            /* shift indices first so they don't get tangled */
            LISTVIEW_ShiftIndices(infoPtr, nItem, 1);

            /* set the item attributes */
            if (*lpLVItem).mask & (LVIF_GROUPID | LVIF_COLUMNS) != 0 {
                /* full size structure expected - _WIN32IE >= 0x560 */
                item = *lpLVItem;
            } else if (*lpLVItem).mask & LVIF_INDENT != 0 {
                /* indent member expected - _WIN32IE >= 0x300 */
                memcpy(
                    &raw mut item as *mut core::ffi::c_void,
                    lpLVItem as *const core::ffi::c_void,
                    core::mem::offset_of!(LVITEMW, iGroupId),
                );
            } else {
                /* minimal structure expected */
                memcpy(
                    &raw mut item as *mut core::ffi::c_void,
                    lpLVItem as *const core::ffi::c_void,
                    core::mem::offset_of!(LVITEMW, iIndent),
                );
            }
            item.iItem = nItem;
            if (*infoPtr).dwLvExStyle & LVS_EX_CHECKBOXES != 0 {
                if item.mask & LVIF_STATE != 0 {
                    item.stateMask |= LVIS_STATEIMAGEMASK;
                    item.state &= !(LVIS_STATEIMAGEMASK);
                    item.state |= INDEXTOSTATEIMAGEMASK!(1);
                } else {
                    item.mask |= LVIF_STATE;
                    item.stateMask = LVIS_STATEIMAGEMASK;
                    item.state = INDEXTOSTATEIMAGEMASK!(1);
                }
            }

            if set_main_item(
                infoPtr,
                &raw mut item,
                TRUE,
                isW,
                &raw mut has_changed,
            ) == 0
            {
                break 'undo;
            }

            /* make room for the position, if we are in the right mode */
            if (*infoPtr).uView == LV_VIEW_SMALLICON || (*infoPtr).uView == LV_VIEW_ICON {
                if DPA_InsertPtr((*infoPtr).hdpaPosX, nItem, core::ptr::null_mut())
                    == -(1)
                {
                    break 'undo;
                }
                if DPA_InsertPtr((*infoPtr).hdpaPosY, nItem, core::ptr::null_mut())
                    == -(1)
                {
                    DPA_DeletePtr((*infoPtr).hdpaPosX, nItem);
                    break 'undo;
                }
            }

            /* send LVN_INSERTITEM notification */
            memset(
                &raw mut nmlv as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<NMLISTVIEW>() as size_t,
            );
            nmlv.iItem = nItem;
            nmlv.lParam = (*lpItem).lParam;
            notify_listview(infoPtr, LVN_INSERTITEM, &raw mut nmlv);
            if IsWindow(hwndSelf) == 0 {
                return -(1);
            }

            /* align items (set position of each item) */
            if (*infoPtr).uView == LV_VIEW_SMALLICON || (*infoPtr).uView == LV_VIEW_ICON {
                let mut pt: POINT = core::mem::zeroed();

                if (*infoPtr).dwStyle & LVS_ALIGNLEFT != 0 {
                    LISTVIEW_NextIconPosLeft(infoPtr, &raw mut pt, nItem);
                } else {
                    LISTVIEW_NextIconPosTop(infoPtr, &raw mut pt, nItem);
                }

                LISTVIEW_MoveIconTo(infoPtr, nItem, &raw mut pt, TRUE);
            }

            /* now is the invalidation fun */
            LISTVIEW_ScrollOnInsert(infoPtr, nItem, 1);
            return nItem;
        }

        // undo:
        LISTVIEW_ShiftIndices(infoPtr, nItem, -(1));
        LISTVIEW_ShiftFocus(
            infoPtr,
            (*infoPtr).nFocusedItem,
            nItem,
            -(1),
        );
        DPA_DeletePtr((*infoPtr).hdpaItems, nItem);
        (*infoPtr).nItemCount -= 1;
    }

    // fail:
    DPA_DeletePtr(hdpaSubItems, 0);
    DPA_Destroy(hdpaSubItems);
    Free(lpItem as LPVOID);
    return -(1);
}

unsafe extern "C" fn LISTVIEW_IsItemVisible(infoPtr: *const LISTVIEW_INFO, nItem: INT) -> BOOL {
    let mut Origin: POINT = POINT { x: 0, y: 0 };
    let mut Position: POINT = POINT { x: 0, y: 0 };
    let mut rcItem: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut hdc: HDC = core::ptr::null_mut();
    let mut ret: BOOL = 0;

    if nItem < 0 || nItem >= DPA_GetPtrCount((*infoPtr).hdpaItems) {
        return FALSE;
    }

    LISTVIEW_GetOrigin(infoPtr, core::ptr::addr_of_mut!(Origin));
    LISTVIEW_GetItemOrigin(infoPtr, nItem, core::ptr::addr_of_mut!(Position));
    rcItem.left = Position.x + Origin.x;
    rcItem.top = Position.y + Origin.y;
    rcItem.right = rcItem.left + (*infoPtr).nItemWidth;
    rcItem.bottom = rcItem.top + (*infoPtr).nItemHeight;

    hdc = GetDC((*infoPtr).hwndSelf);
    if hdc.is_null() {
        return FALSE;
    }
    ret = RectVisible(hdc, core::ptr::addr_of!(rcItem));
    ReleaseDC((*infoPtr).hwndSelf, hdc);

    return ret;
}

unsafe extern "C" fn LISTVIEW_RedrawItems(infoPtr: *const LISTVIEW_INFO, nFirst: INT, nLast: INT) -> BOOL {
    let mut i: INT = 0;
    i = max(nFirst, 0);
    while i <= min(nLast, (*infoPtr).nItemCount - 1) {
        LISTVIEW_InvalidateItem(infoPtr, i);
        i += 1;
    }

    return TRUE;
}

unsafe extern "C" fn LISTVIEW_Scroll(infoPtr: *mut LISTVIEW_INFO, mut dx: INT, mut dy: INT) -> BOOL {
    match (*infoPtr).uView {
        LV_VIEW_DETAILS => {
            dy += (if dy < 0 {
                -1
            } else {
                1
            }) * (*infoPtr).nItemHeight / 2;
            dy /= (*infoPtr).nItemHeight;
        }
        LV_VIEW_LIST => {
            if dy != 0 {
                return FALSE;
            }
        }
        _ => {}
    }

    if dx != 0 {
        LISTVIEW_HScroll(infoPtr, SB_INTERNAL, dx);
    }
    if dy != 0 {
        LISTVIEW_VScroll(infoPtr, SB_INTERNAL, dy);
    }

    return TRUE;
}

unsafe extern "C" fn LISTVIEW_SetBkColor(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut color: COLORREF,
) -> BOOL {
    (*infoPtr).bDefaultBkColor = FALSE;
    if (*infoPtr).clrBk != color {
        if (*infoPtr).clrBk as ::core::ffi::c_ulong != CLR_NONE {
            DeleteObject((*infoPtr).hBkBrush as HGDIOBJ);
        }
        (*infoPtr).clrBk = color;
        if color as ::core::ffi::c_ulong == CLR_NONE {
            (*infoPtr).hBkBrush =
                GetClassLongPtrW((*infoPtr).hwndSelf, GCLP_HBRBACKGROUND) as HBRUSH;
        } else {
            (*infoPtr).hBkBrush = CreateSolidBrush(color);
            (*infoPtr).dwLvExStyle &= !LVS_EX_TRANSPARENTBKGND as DWORD;
        }
    }
    return TRUE;
}

unsafe extern "C" fn column_fill_hditem(
    mut infoPtr: *const LISTVIEW_INFO,
    mut lphdi: *mut HDITEMW,
    mut nColumn: INT,
    mut lpColumn: *const LVCOLUMNW,
    mut isW: BOOL,
) {
    if (*lpColumn).mask & LVCF_FMT as UINT != 0 {
        /* format member is valid */
        (*lphdi).mask |= HDI_FORMAT as UINT;

        /* set text alignment (leftmost column must be left-aligned) */
        if nColumn == 0 as INT
            || (*lpColumn).fmt & (LVCFMT_JUSTIFYMASK as i32) == (LVCFMT_LEFT as i32)
        {
            (*lphdi).fmt |= (HDF_LEFT as i32);
        } else if (*lpColumn).fmt & (LVCFMT_JUSTIFYMASK as i32) == (LVCFMT_RIGHT as i32) {
            (*lphdi).fmt |= (HDF_RIGHT as i32);
        } else if (*lpColumn).fmt & (LVCFMT_JUSTIFYMASK as i32) == (LVCFMT_CENTER as i32) {
            (*lphdi).fmt |= (HDF_CENTER as i32);
        }
        if (*lpColumn).fmt & (LVCFMT_BITMAP_ON_RIGHT as i32) != 0 {
            (*lphdi).fmt |= (HDF_BITMAP_ON_RIGHT as i32);
        }
        if (*lpColumn).fmt & (LVCFMT_COL_HAS_IMAGES as i32) != 0 {
            (*lphdi).fmt |= (HDF_IMAGE as i32);
            (*lphdi).iImage = I_IMAGECALLBACK;
        }
        if (*lpColumn).fmt & (LVCFMT_FIXED_WIDTH as i32) != 0 {
            (*lphdi).fmt |= (HDF_FIXEDWIDTH as i32);
        }
    }

    if (*lpColumn).mask & LVCF_WIDTH as UINT != 0 {
        (*lphdi).mask |= HDI_WIDTH as UINT;
        if (*lpColumn).cx == LVSCW_AUTOSIZE_USEHEADER {
            /* make it fill the remainder of the controls width */
            let mut rcHeader: RECT = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            let mut item_index: INT = 0;

            item_index = 0 as INT;
            while item_index < nColumn - 1 as INT {
                LISTVIEW_GetHeaderRect(infoPtr, item_index, core::ptr::addr_of_mut!(rcHeader));
                (*lphdi).cxy += (rcHeader.right - rcHeader.left) as INT;
                item_index += 1;
            }

            /* retrieve the layout of the header */
            GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcHeader));
            /* TRACE("start cxy=%d rcHeader=%s\n", ...) elided: debug-only output */

            (*lphdi).cxy = (rcHeader.right - rcHeader.left) as INT - (*lphdi).cxy;
        } else {
            (*lphdi).cxy = (*lpColumn).cx;
        }
    }

    if (*lpColumn).mask & LVCF_TEXT as UINT != 0 {
        (*lphdi).mask |= (HDI_TEXT | HDI_FORMAT) as UINT;
        (*lphdi).fmt |= (HDF_STRING as i32);
        (*lphdi).pszText = (*lpColumn).pszText;
        (*lphdi).cchTextMax = textlenT((*lpColumn).pszText as LPCWSTR, isW);
    }

    if (*lpColumn).mask & LVCF_IMAGE as UINT != 0 {
        (*lphdi).mask |= HDI_IMAGE as UINT;
        (*lphdi).iImage = (*lpColumn).iImage;
    }

    if (*lpColumn).mask & LVCF_ORDER as UINT != 0 {
        (*lphdi).mask |= HDI_ORDER as UINT;
        (*lphdi).iOrder = (*lpColumn).iOrder;
    }
}

unsafe extern "C" fn LISTVIEW_InsertColumnT(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut nColumn: INT,
    mut lpColumn: *const LVCOLUMNW,
    mut isW: BOOL,
) -> INT {
    let mut lpColumnInfo: *mut COLUMN_INFO = ::core::ptr::null_mut::<COLUMN_INFO>();
    let mut nNewColumn: INT = 0;
    let mut hdi: HDITEMW = _HD_ITEMW {
        mask: 0,
        cxy: 0,
        pszText: ::core::ptr::null_mut::<WCHAR>(),
        hbm: ::core::ptr::null_mut::<HBITMAP__>(),
        cchTextMax: 0,
        fmt: 0,
        lParam: 0,
        iImage: 0,
        iOrder: 0,
        type_: 0,
        pvFilter: ::core::ptr::null_mut::<::core::ffi::c_void>(),
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
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_InsertColumnT\0" as *const u8 as *const ::core::ffi::c_char,
            8322 as ::core::ffi::c_int,
            b"(nColumn=%d, lpColumn=%s, isW=%d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            nColumn,
            debuglvcolumn_t(lpColumn, isW),
            isW,
        );
    }
    if lpColumn.is_null() || nColumn < 0 as INT {
        return -(1 as INT);
    }
    nColumn = min(nColumn, DPA_GetPtrCount((*infoPtr).hdpaColumns));

    ZeroMemory!(
        &raw mut hdi as *mut ::core::ffi::c_void,
        core::mem::size_of::<HDITEMW>() as usize,
    );
    column_fill_hditem(infoPtr, &raw mut hdi, nColumn, lpColumn, isW);

    if ((*lpColumn).mask & (LVCF_WIDTH as UINT)) == 0 {
        hdi.mask |= HDI_WIDTH as UINT;
        hdi.cxy = 10 as INT;
    }

    if ((*lpColumn).mask & (LVCF_SUBITEM as UINT)) != 0 {
        hdi.mask |= HDI_LPARAM as UINT;
        hdi.lParam = (*lpColumn).iSubItem as LPARAM;
    }

    LISTVIEW_CreateHeader(infoPtr);
    if ((LVS_NOCOLUMNHEADER as DWORD) & (*infoPtr).dwStyle) == 0
        && (*infoPtr).uView == (LV_VIEW_DETAILS as DWORD)
        && ((WS_VISIBLE as DWORD) & (*infoPtr).dwStyle) != 0
    {
        ShowWindow((*infoPtr).hwndHeader, SW_SHOWNORMAL as INT);
    }

    nNewColumn = SendMessageW(
        (*infoPtr).hwndHeader,
        (if isW != 0 {
            HDM_INSERTITEMW
        } else {
            HDM_INSERTITEMA
        }) as UINT,
        nColumn as WPARAM,
        &raw mut hdi as LPARAM,
    ) as INT;
    if nNewColumn == -(1 as INT) {
        return -(1 as INT);
    }
    if nNewColumn != nColumn {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/listview.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"LISTVIEW_InsertColumnT\0" as *const u8 as *const ::core::ffi::c_char,
                8362 as ::core::ffi::c_int,
                b"nColumn=%d, nNewColumn=%d\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                nColumn,
                nNewColumn,
            );
        }
    }

    'fail: {
        lpColumnInfo = Alloc(core::mem::size_of::<COLUMN_INFO>() as DWORD) as *mut COLUMN_INFO;
        if lpColumnInfo.is_null() {
            break 'fail;
        }
        if DPA_InsertPtr(
            (*infoPtr).hdpaColumns,
            nNewColumn,
            lpColumnInfo as *mut ::core::ffi::c_void,
        ) == -(1 as INT)
        {
            break 'fail;
        }

        if (*lpColumn).mask & (LVCF_FMT as UINT) != 0 {
            (*lpColumnInfo).fmt = (*lpColumn).fmt as INT;
        }
        if (*lpColumn).mask & (LVCF_MINWIDTH as UINT) != 0 {
            (*lpColumnInfo).cxMin = (*lpColumn).cxMin as INT;
        }
        if SendMessageW(
            (*infoPtr).hwndHeader,
            HDM_GETITEMRECT as UINT,
            nNewColumn as WPARAM,
            &raw mut (*lpColumnInfo).rcHeader as LPARAM,
        ) == 0
        {
            break 'fail;
        }

        if (*infoPtr).dwStyle & (LVS_OWNERDATA as DWORD) == 0
            && (*infoPtr).nItemCount > 0 as INT
        {
            let mut lpSubItem: *mut SUBITEM_INFO = ::core::ptr::null_mut::<SUBITEM_INFO>();
            let mut hdpaSubItems: HDPA = ::core::ptr::null_mut::<_DPA>();
            let mut nItem: INT = 0;
            let mut i: INT = 0;
            let mut item: LVITEMW = tagLVITEMW {
                mask: 0,
                iItem: 0,
                iSubItem: 0,
                state: 0,
                stateMask: 0,
                pszText: ::core::ptr::null_mut::<WCHAR>(),
                cchTextMax: 0,
                iImage: 0,
                lParam: 0,
                iIndent: 0,
                iGroupId: 0,
                cColumns: 0,
                puColumns: ::core::ptr::null_mut::<::core::ffi::c_uint>(),
            };
            let mut changed: BOOL = 0;

            item.iSubItem = nNewColumn as ::core::ffi::c_int;
            item.mask = (LVIF_TEXT | LVIF_IMAGE) as UINT;
            item.iImage = I_IMAGECALLBACK as INT;
            item.pszText = LPSTR_TEXTCALLBACKW as LPWSTR;

            nItem = 0 as INT;
            while nItem < (*infoPtr).nItemCount {
                hdpaSubItems = DPA_GetPtr((*infoPtr).hdpaItems, nItem) as HDPA;
                i = 1 as INT;
                while i < DPA_GetPtrCount(hdpaSubItems) {
                    lpSubItem = DPA_GetPtr(hdpaSubItems, i) as *mut SUBITEM_INFO;
                    if (*lpSubItem).iSubItem >= nNewColumn {
                        (*lpSubItem).iSubItem += 1;
                    }
                    i += 1;
                }

                item.iItem = nItem as ::core::ffi::c_int;
                set_sub_item(infoPtr, &raw mut item, isW, &raw mut changed);
                nItem += 1;
            }
        }

        LISTVIEW_ScrollColumns(
            infoPtr,
            nNewColumn + 1 as INT,
            (*lpColumnInfo).rcHeader.right - (*lpColumnInfo).rcHeader.left,
        );
        LISTVIEW_UpdateItemSize(infoPtr);

        return nNewColumn;
    }

    if nNewColumn != -(1 as INT) {
        SendMessageW(
            (*infoPtr).hwndHeader,
            HDM_DELETEITEM as UINT,
            nNewColumn as WPARAM,
            0 as LPARAM,
        );
    }
    if !lpColumnInfo.is_null() {
        DPA_DeletePtr((*infoPtr).hdpaColumns, nNewColumn);
        Free(lpColumnInfo as LPVOID);
    }
    return -(1 as INT);
}

unsafe extern "C" fn LISTVIEW_SetColumnT(infoPtr: *const LISTVIEW_INFO, nColumn: INT, lpColumn: *const LVCOLUMNW, isW: BOOL) -> BOOL {
    let mut hdi: HDITEMW = core::mem::zeroed();
    let mut hdiget: HDITEMW = core::mem::zeroed();
    let mut bResult: BOOL;

    if lpColumn.is_null() || nColumn < 0 || nColumn >= DPA_GetPtrCount((*infoPtr).hdpaColumns) {
        return FALSE;
    }

    ZeroMemory!(&raw mut hdi as *mut core::ffi::c_void, core::mem::size_of::<HDITEMW>() as usize);
    if ((*lpColumn).mask & LVCF_FMT) != 0 {
        hdi.mask |= HDI_FORMAT;
        hdiget.mask = HDI_FORMAT;
        if SendMessageW(
            (*infoPtr).hwndHeader,
            HDM_GETITEMW,
            nColumn as WPARAM,
            &raw mut hdiget as LPARAM,
        ) != 0
        {
            hdi.fmt = hdiget.fmt & (HDF_STRING as i32);
        }
    }
    column_fill_hditem(infoPtr, &raw mut hdi, nColumn, lpColumn, isW);

    /* set header item attributes */
    bResult = SendMessageW(
        (*infoPtr).hwndHeader,
        (if isW != 0 { HDM_SETITEMW } else { HDM_SETITEMA }) as UINT,
        nColumn as WPARAM,
        &raw mut hdi as LPARAM,
    ) as BOOL;
    if bResult == 0 {
        return FALSE;
    }

    if ((*lpColumn).mask & LVCF_FMT) != 0 {
        let lpColumnInfo: *mut COLUMN_INFO = LISTVIEW_GetColumnInfo(infoPtr, nColumn);
        let oldFmt: INT = (*lpColumnInfo).fmt;

        (*lpColumnInfo).fmt = (*lpColumn).fmt as INT;
        if ((oldFmt ^ (*lpColumn).fmt as INT) & ((LVCFMT_JUSTIFYMASK | LVCFMT_IMAGE) as INT)) != 0 {
            if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
                LISTVIEW_InvalidateColumn(infoPtr, nColumn);
            }
        }
    }

    if ((*lpColumn).mask & LVCF_MINWIDTH) != 0 {
        (*LISTVIEW_GetColumnInfo(infoPtr, nColumn)).cxMin = (*lpColumn).cxMin as INT;
    }

    return TRUE;
}

unsafe extern "C" fn LISTVIEW_SetColumnOrderArray(infoPtr: *mut LISTVIEW_INFO, iCount: INT, lpiArray: *const INT) -> BOOL {
    if (*infoPtr).hwndHeader.is_null() { return FALSE; }
    (*infoPtr).colRectsDirty = TRUE;
    return SendMessageW((*infoPtr).hwndHeader, HDM_SETORDERARRAY, iCount as WPARAM, lpiArray as LPARAM) as BOOL;
}

unsafe extern "C" fn LISTVIEW_SetColumnWidth(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut nColumn: INT,
    mut cx: INT,
) -> BOOL {
    let mut szDispText: [WCHAR; (DISP_TEXT_SIZE) as usize] = [0; (DISP_TEXT_SIZE) as usize];
    let mut max_cx: INT = 0 as INT;
    let mut hdi: HDITEMW = core::mem::zeroed::<HDITEMW>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_SetColumnWidth\0" as *const u8 as *const ::core::ffi::c_char,
            8514 as ::core::ffi::c_int,
            b"(nColumn=%d, cx=%d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            nColumn,
            cx,
        );
    }
    if (*infoPtr).uView != LV_VIEW_DETAILS as DWORD
        && (*infoPtr).uView != LV_VIEW_LIST as DWORD
    {
        return FALSE;
    }
    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD && cx < LVSCW_AUTOSIZE_USEHEADER as INT {
        cx = LVSCW_AUTOSIZE as INT;
    } else if (*infoPtr).uView == LV_VIEW_LIST as DWORD
        && cx <= 0 as INT
    {
        return FALSE;
    }
    if (*infoPtr).uView == LV_VIEW_LIST as DWORD {
        (*infoPtr).nItemWidth = cx;
        LISTVIEW_InvalidateList(infoPtr);
        return TRUE;
    }
    if nColumn < 0 as INT
        || nColumn >= DPA_GetPtrCount((*infoPtr).hdpaColumns)
    {
        return FALSE;
    }
    if cx == LVSCW_AUTOSIZE as INT
        || (cx == LVSCW_AUTOSIZE_USEHEADER as INT
            && nColumn < DPA_GetPtrCount((*infoPtr).hdpaColumns) - 1 as INT)
    {
        let mut nLabelWidth: INT = 0;
        let mut lvItem: LVITEMW = core::mem::zeroed::<LVITEMW>();
        lvItem.mask = LVIF_TEXT as UINT;
        lvItem.iItem = 0 as INT;
        lvItem.iSubItem = nColumn;
        lvItem.cchTextMax = DISP_TEXT_SIZE as INT;
        while lvItem.iItem < (*infoPtr).nItemCount {
            lvItem.pszText = &raw mut szDispText as *mut WCHAR as LPWSTR;
            if !(LISTVIEW_GetItemW(infoPtr, &raw mut lvItem) == 0) {
                nLabelWidth = LISTVIEW_GetStringWidthT(
                    infoPtr,
                    lvItem.pszText as LPCWSTR,
                    TRUE as BOOL,
                );
                if max_cx < nLabelWidth {
                    max_cx = nLabelWidth;
                }
            }
            lvItem.iItem += 1;
        }
        if !(*infoPtr).himlSmall.is_null()
            && (nColumn == 0 as INT
                || ((*LISTVIEW_GetColumnInfo(infoPtr, nColumn)).fmt as INT
                    & LVCFMT_IMAGE as INT)
                    != 0)
        {
            max_cx += (*infoPtr).iconSize.cx as INT;
        }
        max_cx += TRAILING_LABEL_PADDING as INT;
        if nColumn == 0 as INT && !(*infoPtr).himlState.is_null() {
            max_cx += (*infoPtr).iconStateSize.cx as INT;
        }
    }
    if cx == LVSCW_AUTOSIZE as INT {
        cx = max_cx;
    } else if cx == LVSCW_AUTOSIZE_USEHEADER as INT {
        if nColumn == DPA_GetPtrCount((*infoPtr).hdpaColumns) - 1 as INT {
            let mut rcHeader: RECT = core::mem::zeroed::<RECT>();
            let mut Origin: POINT = core::mem::zeroed::<POINT>();
            LISTVIEW_GetOrigin(infoPtr, &raw mut Origin);
            LISTVIEW_GetHeaderRect(infoPtr, nColumn, &raw mut rcHeader);
            cx = ((*infoPtr).rcList.right - Origin.x - rcHeader.left) as INT;
        } else {
            cx = 0 as INT;
            hdi.mask = (HDI_TEXT as UINT)
                | (HDI_FORMAT as UINT)
                | (HDI_IMAGE as UINT)
                | (HDI_BITMAP as UINT);
            hdi.cchTextMax = DISP_TEXT_SIZE as INT;
            hdi.pszText = &raw mut szDispText as *mut WCHAR as LPWSTR;
            if SendMessageW(
                (*infoPtr).hwndHeader,
                HDM_GETITEMW as UINT,
                nColumn as WPARAM,
                &raw mut hdi as LPARAM,
            ) != 0
            {
                let mut hdc: HDC = GetDC((*infoPtr).hwndSelf);
                let mut old_font: HFONT = SelectObject(
                    hdc,
                    SendMessageW(
                        (*infoPtr).hwndHeader,
                        WM_GETFONT as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) as HFONT as HGDIOBJ,
                ) as HFONT;
                let mut himl: HIMAGELIST = SendMessageW(
                    (*infoPtr).hwndHeader,
                    HDM_GETIMAGELIST as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                ) as HIMAGELIST;
                let mut bitmap_margin: INT = 0 as INT;
                let mut size: SIZE = core::mem::zeroed::<SIZE>();
                if GetTextExtentPoint32W(
                    hdc,
                    hdi.pszText as LPCWSTR,
                    lstrlenW(hdi.pszText as LPCWSTR),
                    &raw mut size,
                ) != 0
                {
                    cx = (size.cx as INT + TRAILING_HEADER_PADDING as INT) as INT;
                }
                if (hdi.fmt & (HDF_IMAGE as INT | HDF_BITMAP as INT))
                    != 0
                {
                    bitmap_margin = SendMessageW(
                        (*infoPtr).hwndHeader,
                        HDM_GETBITMAPMARGIN as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) as INT;
                }
                if (hdi.fmt & HDF_IMAGE as INT) != 0 && !himl.is_null() {
                    let mut icon_cx: INT = 0;
                    let mut icon_cy: INT = 0;
                    if ImageList_GetIconSize(himl, &raw mut icon_cx, &raw mut icon_cy) == 0 {
                        cx += (icon_cx + 2 as INT * bitmap_margin) as INT;
                    }
                } else if (hdi.fmt & HDF_BITMAP as INT) != 0 {
                    let mut bmp: BITMAP = core::mem::zeroed::<BITMAP>();
                    GetObjectW(
                        hdi.hbm as HANDLE,
                        core::mem::size_of::<BITMAP>() as ::core::ffi::c_int,
                        &raw mut bmp as LPVOID,
                    );
                    cx += (bmp.bmWidth as INT + 2 as INT * bitmap_margin)
                        as INT;
                }
                SelectObject(hdc, old_font as HGDIOBJ);
                ReleaseDC((*infoPtr).hwndSelf, hdc);
            }
            cx = if cx > max_cx { cx } else { max_cx };
        }
    }
    if cx < 0 as INT {
        return FALSE;
    }
    hdi.mask = HDI_WIDTH as UINT;
    let __colinfo: *mut COLUMN_INFO = LISTVIEW_GetColumnInfo(infoPtr, nColumn);
    hdi.cxy = (if cx > (*__colinfo).cxMin { cx } else { (*__colinfo).cxMin }) as INT;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_SetColumnWidth\0" as *const u8 as *const ::core::ffi::c_char,
            8632 as ::core::ffi::c_int,
            b"hdi.cxy=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            hdi.cxy,
        );
    }
    return SendMessageW(
        (*infoPtr).hwndHeader,
        HDM_SETITEMW as UINT,
        nColumn as WPARAM,
        &raw mut hdi as LPARAM,
    ) as BOOL;
}

unsafe extern "C" fn LISTVIEW_CreateCheckBoxIL(infoPtr: *const LISTVIEW_INFO) -> HIMAGELIST {
    let mut hdc_wnd: HDC = core::ptr::null_mut();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hbm_im: HBITMAP = core::ptr::null_mut();
    let mut hbm_mask: HBITMAP = core::ptr::null_mut();
    let mut hbm_orig: HBITMAP = core::ptr::null_mut();
    let mut rc: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let hbr_white: HBRUSH = GetStockObject((WHITE_BRUSH as i32)) as HBRUSH;
    let hbr_black: HBRUSH = GetStockObject((BLACK_BRUSH as i32)) as HBRUSH;
    let mut himl: HIMAGELIST;

    himl = ImageList_Create(
        GetSystemMetrics((SM_CXSMICON) as i32),
        GetSystemMetrics((SM_CYSMICON) as i32),
        (ILC_COLOR | ILC_MASK) as UINT,
        2 as core::ffi::c_int,
        2 as core::ffi::c_int,
    );
    hdc_wnd = GetDC((*infoPtr).hwndSelf);
    hdc = CreateCompatibleDC(hdc_wnd);
    hbm_im = CreateCompatibleBitmap(
        hdc_wnd,
        GetSystemMetrics((SM_CXSMICON) as i32) as INT,
        GetSystemMetrics((SM_CYSMICON) as i32) as INT,
    );
    hbm_mask = CreateBitmap(
        GetSystemMetrics((SM_CXSMICON) as i32) as INT,
        GetSystemMetrics((SM_CYSMICON) as i32) as INT,
        1 as UINT,
        1 as UINT,
        core::ptr::null(),
    );
    ReleaseDC((*infoPtr).hwndSelf, hdc_wnd);

    SetRect(
        &raw mut rc,
        0 as core::ffi::c_int,
        0 as core::ffi::c_int,
        GetSystemMetrics((SM_CXSMICON) as i32),
        GetSystemMetrics((SM_CYSMICON) as i32),
    );
    hbm_orig = SelectObject(hdc, hbm_mask as HGDIOBJ) as HBITMAP;
    FillRect(hdc, &raw mut rc as LPCRECT, hbr_white);
    InflateRect(
        &raw mut rc,
        -(2 as core::ffi::c_int),
        -(2 as core::ffi::c_int),
    );
    FillRect(hdc, &raw mut rc as LPCRECT, hbr_black);

    SelectObject(hdc, hbm_im as HGDIOBJ);
    DrawFrameControl(
        hdc,
        &raw mut rc,
        DFC_BUTTON as UINT,
        (DFCS_BUTTONCHECK | DFCS_MONO) as UINT,
    );
    SelectObject(hdc, hbm_orig as HGDIOBJ);
    ImageList_Add(himl, hbm_im, hbm_mask);

    SelectObject(hdc, hbm_im as HGDIOBJ);
    DrawFrameControl(
        hdc,
        &raw mut rc,
        DFC_BUTTON as UINT,
        (DFCS_BUTTONCHECK | DFCS_MONO | DFCS_CHECKED) as UINT,
    );
    SelectObject(hdc, hbm_orig as HGDIOBJ);
    ImageList_Add(himl, hbm_im, hbm_mask);

    DeleteObject(hbm_mask as HGDIOBJ);
    DeleteObject(hbm_im as HGDIOBJ);
    DeleteDC(hdc);

    himl
}

unsafe extern "C" fn LISTVIEW_SetExtendedListViewStyle(
    infoPtr: *mut LISTVIEW_INFO,
    mask: DWORD,
    ex_style: DWORD,
) -> DWORD {
    let old_ex_style: DWORD = (*infoPtr).dwLvExStyle;

    /* set new style */
    if mask != 0 {
        (*infoPtr).dwLvExStyle = (old_ex_style & !mask) | (ex_style & mask);
    } else {
        (*infoPtr).dwLvExStyle = ex_style;
    }

    if (((*infoPtr).dwLvExStyle ^ old_ex_style) & (LVS_EX_CHECKBOXES as DWORD)) != 0 {
        let mut himl: HIMAGELIST = core::ptr::null_mut();
        if ((*infoPtr).dwLvExStyle & (LVS_EX_CHECKBOXES as DWORD)) != 0 {
            let mut item: LVITEMW = core::mem::zeroed();
            item.mask = LVIF_STATE as UINT;
            item.stateMask = LVIS_STATEIMAGEMASK as UINT;
            item.state = INDEXTOSTATEIMAGEMASK!(1 as INT) as UINT;
            LISTVIEW_SetItemState(infoPtr, -(1 as INT), core::ptr::addr_of_mut!(item));

            himl = LISTVIEW_CreateCheckBoxIL(infoPtr);
            if ((*infoPtr).dwStyle & (LVS_SHAREIMAGELISTS as DWORD)) == 0 {
                ImageList_Destroy((*infoPtr).himlState);
            }
        }
        himl = LISTVIEW_SetImageList(infoPtr, LVSIL_STATE as INT, himl);
        /*   checkbox list replaces previous custom list or... */
        if (((*infoPtr).dwLvExStyle & (LVS_EX_CHECKBOXES as DWORD)) != 0
            && ((*infoPtr).dwStyle & (LVS_SHAREIMAGELISTS as DWORD)) == 0)
            /* ...previous was checkbox list */
            || (old_ex_style & (LVS_EX_CHECKBOXES as DWORD)) != 0
        {
            ImageList_Destroy(himl);
        }
    }

    if (((*infoPtr).dwLvExStyle ^ old_ex_style) & (LVS_EX_HEADERDRAGDROP as DWORD)) != 0 {
        let mut style: DWORD;

        /* if not already created */
        LISTVIEW_CreateHeader(infoPtr);

        style = GetWindowLongW((*infoPtr).hwndHeader, GWL_STYLE as INT) as DWORD;
        if ((*infoPtr).dwLvExStyle & (LVS_EX_HEADERDRAGDROP as DWORD)) != 0 {
            style |= HDS_DRAGDROP as DWORD;
        } else {
            style &= !(HDS_DRAGDROP as DWORD);
        }
        SetWindowLongW((*infoPtr).hwndHeader, GWL_STYLE as INT, style as LONG);
    }

    /* GRIDLINES adds decoration at top so changes sizes */
    if (((*infoPtr).dwLvExStyle ^ old_ex_style) & (LVS_EX_GRIDLINES as DWORD)) != 0 {
        LISTVIEW_CreateHeader(infoPtr);
        LISTVIEW_UpdateSize(infoPtr);
    }

    if (((*infoPtr).dwLvExStyle ^ old_ex_style) & (LVS_EX_FULLROWSELECT as DWORD)) != 0 {
        LISTVIEW_CreateHeader(infoPtr);
    }

    if (((*infoPtr).dwLvExStyle ^ old_ex_style) & (LVS_EX_TRANSPARENTBKGND as DWORD)) != 0 {
        if ((*infoPtr).dwLvExStyle & (LVS_EX_TRANSPARENTBKGND as DWORD)) != 0 {
            LISTVIEW_SetBkColor(infoPtr, CLR_NONE as COLORREF);
        }
    }

    if (((*infoPtr).dwLvExStyle ^ old_ex_style) & (LVS_EX_HEADERINALLVIEWS as DWORD)) != 0 {
        if ((*infoPtr).dwLvExStyle & (LVS_EX_HEADERINALLVIEWS as DWORD)) != 0 {
            LISTVIEW_CreateHeader(infoPtr);
        } else {
            ShowWindow((*infoPtr).hwndHeader, SW_HIDE as INT);
        }
        LISTVIEW_UpdateSize(infoPtr);
        LISTVIEW_UpdateScroll(infoPtr);
    }

    if ((*infoPtr).dwLvExStyle & (LVS_EX_SNAPTOGRID as DWORD))
        > (old_ex_style & (LVS_EX_SNAPTOGRID as DWORD))
    {
        LISTVIEW_Arrange(infoPtr, LVA_SNAPTOGRID as INT);
    }

    LISTVIEW_InvalidateList(infoPtr);
    return old_ex_style;
}

unsafe extern "C" fn LISTVIEW_SetHotCursor(infoPtr: *mut LISTVIEW_INFO, hCursor: HCURSOR) -> HCURSOR {
    let mut oldCursor: HCURSOR = (*infoPtr).hHotCursor;
    (*infoPtr).hHotCursor = hCursor;
    return oldCursor;
}

unsafe extern "C" fn LISTVIEW_SetHotItem(infoPtr: *mut LISTVIEW_INFO, iIndex: INT) -> INT {
    let mut iOldIndex: INT = (*infoPtr).nHotItem;
    (*infoPtr).nHotItem = iIndex;
    return iOldIndex;
}

unsafe extern "C" fn LISTVIEW_SetHoverTime(infoPtr: *mut LISTVIEW_INFO, dwHoverTime: DWORD) -> DWORD {
    let oldHoverTime: DWORD = (*infoPtr).dwHoverTime;

    (*infoPtr).dwHoverTime = dwHoverTime;

    return oldHoverTime;
}

unsafe extern "C" fn LISTVIEW_SetIconSpacing(infoPtr: *mut LISTVIEW_INFO, mut cx: INT, mut cy: INT) -> DWORD {
    let mut iconWidth: INT = 0;
    let mut iconHeight: INT = 0;
    let oldspacing: DWORD = MAKELONG!((*infoPtr).iconSpacing.cx as WORD, (*infoPtr).iconSpacing.cy as WORD) as DWORD;

    /* set to defaults, if instructed to */
    if cx == -1 && cy == -1 {
        (*infoPtr).autoSpacing = TRUE;
        if !(*infoPtr).himlNormal.is_null() {
            ImageList_GetIconSize(
                (*infoPtr).himlNormal,
                core::ptr::addr_of_mut!(iconWidth),
                core::ptr::addr_of_mut!(iconHeight),
            );
        }
        cx = GetSystemMetrics((SM_CXICONSPACING) as i32) - GetSystemMetrics((SM_CXICON) as i32) + iconWidth;
        cy = GetSystemMetrics((SM_CYICONSPACING) as i32) - GetSystemMetrics((SM_CYICON) as i32) + iconHeight;
    } else {
        (*infoPtr).autoSpacing = FALSE;
    }

    /* if 0 then keep width */
    if cx != 0 {
        (*infoPtr).iconSpacing.cx = cx;
    }

    /* if 0 then keep height */
    if cy != 0 {
        (*infoPtr).iconSpacing.cy = cy;
    }

    /* these depend on the iconSpacing */
    LISTVIEW_UpdateItemSize(infoPtr);

    return oldspacing;
}

unsafe extern "C" fn set_icon_size(size: *mut SIZE, himl: HIMAGELIST, is_small: BOOL) {
    let mut cx: INT = 0;
    let mut cy: INT = 0;
    if !himl.is_null() && ImageList_GetIconSize(himl, core::ptr::addr_of_mut!(cx), core::ptr::addr_of_mut!(cy)) != 0 {
        (*size).cx = cx as LONG;
        (*size).cy = cy as LONG;
    } else {
        (*size).cx = GetSystemMetrics(if is_small != 0 { (SM_CXSMICON) as i32 } else { (SM_CXICON) as i32 }) as LONG;
        (*size).cy = GetSystemMetrics(if is_small != 0 { (SM_CYSMICON) as i32 } else { (SM_CYICON) as i32 }) as LONG;
    }
}

unsafe extern "C" fn LISTVIEW_SetImageList(infoPtr: *mut LISTVIEW_INFO, nType: INT, himl: HIMAGELIST) -> HIMAGELIST {
    let oldHeight: INT = (*infoPtr).nItemHeight;
    let mut himlOld: HIMAGELIST = core::ptr::null_mut();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_SetImageList\0" as *const u8 as *const ::core::ffi::c_char,
            8930 as ::core::ffi::c_int,
            b"(nType=%d, himl=%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            nType,
            himl,
        );
    }
    match (nType) as u32 {
        LVSIL_NORMAL => {
            himlOld = (*infoPtr).himlNormal;
            (*infoPtr).himlNormal = himl;
            if (*infoPtr).uView == LV_VIEW_ICON as DWORD {
                set_icon_size(core::ptr::addr_of_mut!((*infoPtr).iconSize), himl, FALSE);
            }
            if (*infoPtr).autoSpacing != 0 {
                LISTVIEW_SetIconSpacing(infoPtr, -1, -1);
            }
        }
        LVSIL_SMALL => {
            himlOld = (*infoPtr).himlSmall;
            (*infoPtr).himlSmall = himl;
            if (*infoPtr).uView != LV_VIEW_ICON as DWORD {
                set_icon_size(core::ptr::addr_of_mut!((*infoPtr).iconSize), himl, TRUE);
            }
            if !(*infoPtr).hwndHeader.is_null() {
                SendMessageW(
                    (*infoPtr).hwndHeader,
                    HDM_SETIMAGELIST as UINT,
                    0 as WPARAM,
                    himl as LPARAM,
                );
            }
        }
        LVSIL_STATE => {
            himlOld = (*infoPtr).himlState;
            (*infoPtr).himlState = himl;
            set_icon_size(core::ptr::addr_of_mut!((*infoPtr).iconStateSize), himl, TRUE);
            ImageList_SetBkColor((*infoPtr).himlState, CLR_NONE as COLORREF);
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/listview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"LISTVIEW_SetImageList\0" as *const u8 as *const ::core::ffi::c_char,
                    8958 as ::core::ffi::c_int,
                    b"Unknown icon type=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
                    nType,
                );
            }
            return core::ptr::null_mut();
        }
    }
    (*infoPtr).nItemHeight = LISTVIEW_CalculateItemHeight(infoPtr);
    if (*infoPtr).nItemHeight != oldHeight {
        LISTVIEW_UpdateScroll(infoPtr);
    }
    return himlOld;
}

unsafe extern "C" fn LISTVIEW_SetItemCount(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut nItems: INT,
    mut dwFlags: DWORD,
) -> BOOL {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_SetItemCount\0" as *const u8 as *const ::core::ffi::c_char,
            8984 as ::core::ffi::c_int,
            b"(nItems=%d, dwFlags=%x)\n\0" as *const u8 as *const ::core::ffi::c_char,
            nItems,
            dwFlags,
        );
    }
    if (*infoPtr).dwStyle & LVS_OWNERDATA as DWORD != 0 {
        let mut nOldCount: INT = (*infoPtr).nItemCount;
        (*infoPtr).nItemCount = nItems;
        if nItems < nOldCount {
            let mut range: RANGE = RANGE {
                lower: nItems,
                upper: nOldCount,
            };
            ranges_del((*infoPtr).selectionRanges, range);
            if (*infoPtr).nFocusedItem >= nItems {
                LISTVIEW_SetItemFocus(infoPtr, -(1 as INT));
                (*infoPtr).nFocusedItem = -(1 as INT);
                SetRectEmpty(&raw mut (*infoPtr).rcFocus);
            }
        }
        LISTVIEW_UpdateScroll(infoPtr);
        if (*infoPtr).uView == LV_VIEW_ICON as DWORD
            || (*infoPtr).uView == LV_VIEW_SMALLICON as DWORD
        {
            dwFlags = 0 as DWORD;
        }
        if dwFlags & LVSICF_NOSCROLL as DWORD == 0
            && (*infoPtr).nFocusedItem != -(1 as INT)
        {
            LISTVIEW_EnsureVisible(infoPtr, (*infoPtr).nFocusedItem, FALSE);
        }
        if dwFlags & LVSICF_NOINVALIDATEALL as DWORD == 0 {
            LISTVIEW_InvalidateList(infoPtr);
        } else {
            let mut nFrom: INT = 0;
            let mut nTo: INT = 0;
            let mut Origin: POINT = tagPOINT { x: 0, y: 0 };
            let mut rcErase: RECT = tagRECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            LISTVIEW_GetOrigin(infoPtr, &raw mut Origin);
            nFrom = min(nOldCount, nItems);
            nTo = max(nOldCount, nItems);
            if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
                SetRect(
                    &raw mut rcErase,
                    0 as ::core::ffi::c_int,
                    nFrom as ::core::ffi::c_int * (*infoPtr).nItemHeight as ::core::ffi::c_int,
                    (*infoPtr).nItemWidth as ::core::ffi::c_int,
                    nTo as ::core::ffi::c_int * (*infoPtr).nItemHeight as ::core::ffi::c_int,
                );
                OffsetRect(
                    &raw mut rcErase,
                    Origin.x as ::core::ffi::c_int,
                    Origin.y as ::core::ffi::c_int,
                );
                if IntersectRect(
                    &raw mut rcErase,
                    &raw mut rcErase as LPCRECT,
                    &raw mut (*infoPtr).rcList as LPCRECT,
                ) != 0
                {
                    LISTVIEW_InvalidateRect(infoPtr, &raw mut rcErase);
                }
            } else {
                let mut nPerCol: INT = LISTVIEW_GetCountPerColumn(infoPtr);
                rcErase.left = (nFrom / nPerCol * (*infoPtr).nItemWidth) as LONG;
                rcErase.top = (nFrom % nPerCol * (*infoPtr).nItemHeight) as LONG;
                rcErase.right = (rcErase.left as INT + (*infoPtr).nItemWidth) as LONG;
                rcErase.bottom = (nPerCol * (*infoPtr).nItemHeight) as LONG;
                OffsetRect(
                    &raw mut rcErase,
                    Origin.x as ::core::ffi::c_int,
                    Origin.y as ::core::ffi::c_int,
                );
                if IntersectRect(
                    &raw mut rcErase,
                    &raw mut rcErase as LPCRECT,
                    &raw mut (*infoPtr).rcList as LPCRECT,
                ) != 0
                {
                    LISTVIEW_InvalidateRect(infoPtr, &raw mut rcErase);
                }
                rcErase.left = ((nFrom / nPerCol + 1 as INT)
                    * (*infoPtr).nItemWidth) as LONG;
                rcErase.top = 0 as LONG;
                rcErase.right = ((nTo / nPerCol + 1 as INT)
                    * (*infoPtr).nItemWidth) as LONG;
                rcErase.bottom = (nPerCol * (*infoPtr).nItemHeight) as LONG;
                OffsetRect(
                    &raw mut rcErase,
                    Origin.x as ::core::ffi::c_int,
                    Origin.y as ::core::ffi::c_int,
                );
                if IntersectRect(
                    &raw mut rcErase,
                    &raw mut rcErase as LPCRECT,
                    &raw mut (*infoPtr).rcList as LPCRECT,
                ) != 0
                {
                    LISTVIEW_InvalidateRect(infoPtr, &raw mut rcErase);
                }
            }
        }
    } else if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_SetItemCount\0" as *const u8 as *const ::core::ffi::c_char,
            9061 as ::core::ffi::c_int,
            b"for non-ownerdata performance option not implemented.\n\0" as *const u8
                as *const ::core::ffi::c_char,
        );
    }
    return TRUE;
}

unsafe extern "C" fn LISTVIEW_SetItemPosition(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut nItem: INT,
    mut pt: *const POINT,
) -> BOOL {
    let mut Origin: POINT = POINT { x: 0, y: 0 };
    let mut Pt: POINT = POINT { x: 0, y: 0 };
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_SetItemPosition\0" as *const u8 as *const ::core::ffi::c_char,
            9084 as ::core::ffi::c_int,
            b"(nItem=%d, pt=%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            nItem,
            wine_dbgstr_point(pt),
        );
    }
    if pt.is_null()
        || nItem < 0
        || nItem >= (*infoPtr).nItemCount
        || !((*infoPtr).uView == LV_VIEW_ICON
            || (*infoPtr).uView == LV_VIEW_SMALLICON)
    {
        return FALSE;
    }
    if is_autoarrange(infoPtr) != 0 {
        return FALSE;
    }
    Pt = *pt;
    LISTVIEW_GetOrigin(infoPtr, &raw mut Origin);
    if (Pt.x == -1) && (Pt.y == -1) {
        Pt = Origin;
    }
    if (*infoPtr).uView == LV_VIEW_ICON {
        Pt.x -= ((*infoPtr).nItemWidth - (*infoPtr).iconSize.cx) / 2;
        Pt.y -= ICON_TOP_PADDING;
    }
    Pt.x -= Origin.x;
    Pt.y -= Origin.y;
    if (*infoPtr).dwLvExStyle & LVS_EX_SNAPTOGRID != 0 {
        Pt.x = max(
            0,
            Pt.x + ((*infoPtr).nItemWidth >> 1)
                - (Pt.x + ((*infoPtr).nItemWidth >> 1))
                    % (*infoPtr).nItemWidth,
        );
        Pt.y = max(
            0,
            Pt.y + ((*infoPtr).nItemHeight >> 1)
                - (Pt.y + ((*infoPtr).nItemHeight >> 1))
                    % (*infoPtr).nItemHeight,
        );
    }
    return LISTVIEW_MoveIconTo(infoPtr, nItem, &raw mut Pt, FALSE);
}

unsafe extern "C" fn LISTVIEW_SetItemState(infoPtr: *mut LISTVIEW_INFO, nItem: INT, item: *const LVITEMW) -> BOOL {
    let mut ret: BOOL = TRUE as BOOL;
    let mut lvItem: LVITEMW = core::mem::zeroed();
    if item.is_null() {
        return FALSE as BOOL;
    }
    lvItem.iItem = nItem as INT;
    lvItem.iSubItem = 0 as INT;
    lvItem.mask = LVIF_STATE as UINT;
    lvItem.state = (*item).state;
    lvItem.stateMask = (*item).stateMask;
    if (*__wine_dbch___default).flags as INT & ((1 as INT) << __WINE_DBCL_TRACE as INT) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_SetItemState\0" as *const u8 as *const ::core::ffi::c_char,
            9148 as ::core::ffi::c_int,
            b"item=%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            debuglvitem_t(core::ptr::addr_of_mut!(lvItem), 1 as BOOL),
        );
    }
    if nItem == -(1 as INT) {
        let mut oldstate: UINT = 0 as UINT;
        let mut old_mask: DWORD;

        /* special case optimization for recurring attempt to deselect all */
        if lvItem.state == (0 as UINT)
            && lvItem.stateMask == (LVIS_SELECTED as UINT)
            && LISTVIEW_GetSelectedCount(infoPtr) == 0
        {
            return TRUE as BOOL;
        }

        /* select all isn't allowed in LVS_SINGLESEL */
        if ((lvItem.state & lvItem.stateMask & (LVIS_SELECTED as UINT)) != 0)
            && ((*infoPtr).dwStyle & (LVS_SINGLESEL as DWORD)) != 0
        {
            return FALSE as BOOL;
        }

        /* focus all isn't allowed */
        if (lvItem.state & lvItem.stateMask & (LVIS_FOCUSED as UINT)) != 0 {
            return FALSE as BOOL;
        }

        old_mask = (*infoPtr).notify_mask & (NOTIFY_MASK_ITEM_CHANGE as DWORD);
        if (*infoPtr).dwStyle & (LVS_OWNERDATA as DWORD) != 0 {
            (*infoPtr).notify_mask &= !(NOTIFY_MASK_ITEM_CHANGE as DWORD);
            if (lvItem.state & (LVIS_SELECTED as UINT)) == 0
                && LISTVIEW_GetSelectedCount(infoPtr) != 0
            {
                oldstate |= LVIS_SELECTED as UINT;
            }
            if (*infoPtr).nFocusedItem != -(1 as INT) {
                oldstate |= LVIS_FOCUSED as UINT;
            }
        }

        /* apply to all items */
        lvItem.iItem = 0 as INT;
        while lvItem.iItem < (*infoPtr).nItemCount {
            if LISTVIEW_SetItemT(infoPtr, core::ptr::addr_of_mut!(lvItem), TRUE as BOOL) == 0 {
                ret = FALSE as BOOL;
            }
            lvItem.iItem += 1;
        }

        if (*infoPtr).dwStyle & (LVS_OWNERDATA as DWORD) != 0 {
            let mut nmlv: NMLISTVIEW = core::mem::zeroed();

            (*infoPtr).notify_mask |= old_mask;

            nmlv.iItem = -(1 as INT);
            nmlv.iSubItem = 0 as INT;
            nmlv.uNewState = lvItem.state & lvItem.stateMask;
            nmlv.uOldState = oldstate & lvItem.stateMask;
            nmlv.uChanged = LVIF_STATE as UINT;
            nmlv.ptAction.y = 0 as LONG;
            nmlv.ptAction.x = nmlv.ptAction.y;
            nmlv.lParam = 0 as LPARAM;

            notify_listview(infoPtr, LVN_ITEMCHANGED as INT, core::ptr::addr_of_mut!(nmlv));
        }
    } else {
        ret = LISTVIEW_SetItemT(infoPtr, core::ptr::addr_of_mut!(lvItem), TRUE as BOOL);
    }

    return ret;
}

unsafe extern "C" fn LISTVIEW_SetItemTextT(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut nItem: INT,
    mut lpLVItem: *const LVITEMW,
    mut isW: BOOL,
) -> BOOL {
    let mut lvItem: LVITEMW = tagLVITEMW {
        mask: 0,
        iItem: 0,
        iSubItem: 0,
        state: 0,
        stateMask: 0,
        pszText: ::core::ptr::null_mut::<WCHAR>(),
        cchTextMax: 0,
        iImage: 0,
        lParam: 0,
        iIndent: 0,
        iGroupId: 0,
        cColumns: 0,
        puColumns: ::core::ptr::null_mut::<::core::ffi::c_uint>(),
    };
    if lpLVItem.is_null() || nItem < 0 as ::core::ffi::c_int || nItem >= (*infoPtr).nItemCount {
        return FALSE as BOOL;
    }
    if (*infoPtr).dwStyle & LVS_OWNERDATA as DWORD != 0 {
        return FALSE as BOOL;
    }
    lvItem.iItem = nItem as ::core::ffi::c_int;
    lvItem.iSubItem = (*lpLVItem).iSubItem;
    lvItem.mask = LVIF_TEXT as UINT;
    lvItem.pszText = (*lpLVItem).pszText;
    lvItem.cchTextMax = (*lpLVItem).cchTextMax;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_SetItemTextT\0" as *const u8 as *const ::core::ffi::c_char,
            9229 as ::core::ffi::c_int,
            b"(nItem=%d, lpLVItem=%s, isW=%d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            nItem,
            debuglvitem_t(&raw mut lvItem, isW),
            isW,
        );
    }
    return LISTVIEW_SetItemT(infoPtr, &raw mut lvItem, isW);
}

unsafe extern "C" fn LISTVIEW_SetSelectionMark(infoPtr: *mut LISTVIEW_INFO, nIndex: INT) -> INT {
    let mut nOldIndex: INT = (*infoPtr).nSelectionMark;

    if nIndex >= -1 && nIndex < (*infoPtr).nItemCount {
        (*infoPtr).nSelectionMark = nIndex;
    }

    return nOldIndex;
}

unsafe extern "C" fn LISTVIEW_SetTextBkColor(infoPtr: *mut LISTVIEW_INFO, color: COLORREF) -> BOOL {
    (*infoPtr).clrTextBk = color;
    return TRUE;
}

unsafe extern "C" fn LISTVIEW_SetTextColor(mut infoPtr: *mut LISTVIEW_INFO, mut color: COLORREF) -> BOOL {
    (*infoPtr).clrText = color;
    return TRUE;
}

unsafe extern "C" fn LISTVIEW_SetToolTips(infoPtr: *mut LISTVIEW_INFO, hwndNewToolTip: HWND) -> HWND {
    let hwndOldToolTip: HWND = (*infoPtr).hwndToolTip;
    (*infoPtr).hwndToolTip = hwndNewToolTip;
    return hwndOldToolTip;
}

unsafe extern "C" fn LISTVIEW_SetUnicodeFormat(
    infoPtr: *mut LISTVIEW_INFO,
    unicode: BOOL,
) -> BOOL {
    let rc: SHORT = (*infoPtr).notifyFormat;
    (*infoPtr).notifyFormat = (if unicode != 0 { NFR_UNICODE } else { NFR_ANSI }) as SHORT;
    return ((rc as ::core::ffi::c_int) as u32 == NFR_UNICODE) as BOOL;
}

unsafe extern "C" fn LISTVIEW_SetView(mut infoPtr: *mut LISTVIEW_INFO, mut nView: DWORD) -> INT {
    let mut himl: HIMAGELIST = ::core::ptr::null_mut::<_IMAGELIST>();
    if (*infoPtr).uView == nView {
        return 1;
    }
    if (nView as INT) < 0 || nView > LV_VIEW_MAX as DWORD {
        return -(1);
    }
    if nView == LV_VIEW_TILE as DWORD {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/listview.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"LISTVIEW_SetView\0" as *const u8 as *const ::core::ffi::c_char,
                9352 as ::core::ffi::c_int,
                b"View LV_VIEW_TILE unimplemented\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return -(1);
    }

    (*infoPtr).uView = nView;

    SendMessageW(
        (*infoPtr).hwndEdit,
        WM_KILLFOCUS as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
    ShowWindow((*infoPtr).hwndHeader, (SW_HIDE) as i32);

    ShowScrollBar(
        (*infoPtr).hwndSelf,
        (SB_BOTH) as i32,
        FALSE,
    );
    SetRectEmpty(::core::ptr::addr_of_mut!((*infoPtr).rcFocus));

    himl = if nView == LV_VIEW_ICON as DWORD {
        (*infoPtr).himlNormal
    } else {
        (*infoPtr).himlSmall
    };
    set_icon_size(
        ::core::ptr::addr_of_mut!((*infoPtr).iconSize),
        himl,
        (nView != LV_VIEW_ICON as DWORD) as ::core::ffi::c_int,
    );

    match nView {
        LV_VIEW_ICON | LV_VIEW_SMALLICON => {
            LISTVIEW_Arrange(infoPtr, (LVA_DEFAULT) as INT);
        }
        LV_VIEW_DETAILS => {
            let mut hl: HDLAYOUT = _HD_LAYOUT {
                prc: ::core::ptr::null_mut::<RECT>(),
                pwpos: ::core::ptr::null_mut::<WINDOWPOS>(),
            };
            let mut wp: WINDOWPOS = _WINDOWPOS {
                hwnd: ::core::ptr::null_mut::<HWND__>(),
                hwndInsertAfter: ::core::ptr::null_mut::<HWND__>(),
                x: 0,
                y: 0,
                cx: 0,
                cy: 0,
                flags: 0,
            };

            LISTVIEW_CreateHeader(infoPtr);

            hl.prc = ::core::ptr::addr_of_mut!((*infoPtr).rcList);
            hl.pwpos = ::core::ptr::addr_of_mut!(wp);
            SendMessageW(
                (*infoPtr).hwndHeader,
                HDM_LAYOUT as UINT,
                0 as WPARAM,
                ::core::ptr::addr_of_mut!(hl) as LPARAM,
            );
            SetWindowPos(
                (*infoPtr).hwndHeader,
                (*infoPtr).hwndSelf,
                wp.x,
                wp.y,
                wp.cx,
                wp.cy,
                wp.flags
                    | (if ((*infoPtr).dwStyle & (LVS_NOCOLUMNHEADER as DWORD)) != 0 {
                        SWP_HIDEWINDOW
                    } else {
                        SWP_SHOWWINDOW
                    }) as UINT,
            );
        }
        LV_VIEW_LIST => {}
        _ => {}
    }

    LISTVIEW_UpdateItemSize(infoPtr);
    LISTVIEW_UpdateSize(infoPtr);
    LISTVIEW_UpdateScroll(infoPtr);
    LISTVIEW_InvalidateList(infoPtr);

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_SetView\0" as *const u8 as *const ::core::ffi::c_char,
            9396 as ::core::ffi::c_int,
            b"nView=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            nView,
        );
    }

    return 1;
}
unsafe extern "system" fn LISTVIEW_CallBackCompare(
    mut first: LPVOID,
    mut second: LPVOID,
    mut lParam: LPARAM,
) -> INT {
    let mut infoPtr: *mut LISTVIEW_INFO = lParam as *mut LISTVIEW_INFO;
    let mut lv_first: *mut ITEM_INFO =
        DPA_GetPtr(first as HDPA, 0 as INT) as *mut ITEM_INFO;
    let mut lv_second: *mut ITEM_INFO =
        DPA_GetPtr(second as HDPA, 0 as INT) as *mut ITEM_INFO;

    /* Forward the call to the client defined callback */
    return (*infoPtr).pfnCompare.unwrap_unchecked()(
        (*lv_first).lParam,
        (*lv_second).lParam,
        (*infoPtr).lParamSort,
    ) as INT;
}
unsafe extern "system" fn LISTVIEW_CallBackCompareEx(first: LPVOID, second: LPVOID, lParam: LPARAM) -> INT {
    let mut infoPtr: *mut LISTVIEW_INFO = lParam as *mut LISTVIEW_INFO;
    let mut first_idx: INT = DPA_GetPtrIndex((*infoPtr).hdpaItems, first as *const ::core::ffi::c_void) as INT;
    let mut second_idx: INT = DPA_GetPtrIndex((*infoPtr).hdpaItems, second as *const ::core::ffi::c_void) as INT;

    /* Forward the call to the client defined callback */
    return (*infoPtr).pfnCompare.unwrap_unchecked()(
        first_idx as LPARAM,
        second_idx as LPARAM,
        (*infoPtr).lParamSort,
    ) as INT;
}

unsafe extern "C" fn LISTVIEW_SortItems(
    infoPtr: *mut LISTVIEW_INFO,
    pfnCompare: PFNLVCOMPARE,
    lParamSort: LPARAM,
    IsEx: BOOL,
) -> BOOL {
    let mut hdpaSubItems: HDPA = core::ptr::null_mut();
    let mut lpItem: *mut ITEM_INFO = core::ptr::null_mut();
    let mut selectionMarkItem: LPVOID = core::ptr::null_mut();
    let mut focusedItem: LPVOID = core::ptr::null_mut();
    let mut i: INT;

    if (*infoPtr).dwStyle & LVS_OWNERDATA as DWORD != 0 {
        return FALSE;
    }

    if pfnCompare.is_none() {
        return FALSE;
    }
    if (*infoPtr).hdpaItems.is_null() {
        return FALSE;
    }

    /* if there are 0 or 1 items, there is no need to sort */
    if (*infoPtr).nItemCount < 2 {
        return TRUE;
    }

    /* clear selection */
    ranges_clear((*infoPtr).selectionRanges);

    /* save selection mark and focused item */
    if (*infoPtr).nSelectionMark >= 0 {
        selectionMarkItem = DPA_GetPtr((*infoPtr).hdpaItems, (*infoPtr).nSelectionMark);
    }
    if (*infoPtr).nFocusedItem >= 0 {
        focusedItem = DPA_GetPtr((*infoPtr).hdpaItems, (*infoPtr).nFocusedItem);
    }

    (*infoPtr).pfnCompare = pfnCompare;
    (*infoPtr).lParamSort = lParamSort;
    if IsEx != 0 {
        DPA_Sort(
            (*infoPtr).hdpaItems,
            Some(LISTVIEW_CallBackCompareEx),
            infoPtr as LPARAM,
        );
    } else {
        DPA_Sort(
            (*infoPtr).hdpaItems,
            Some(LISTVIEW_CallBackCompare),
            infoPtr as LPARAM,
        );
    }

    /* restore selection ranges */
    i = 0;
    while i < (*infoPtr).nItemCount {
        hdpaSubItems = DPA_GetPtr((*infoPtr).hdpaItems, i) as HDPA;
        lpItem = DPA_GetPtr(hdpaSubItems, 0) as *mut ITEM_INFO;

        if (*lpItem).state & LVIS_SELECTED as UINT != 0 {
            ranges_additem((*infoPtr).selectionRanges, i);
        }
        i += 1;
    }
    /* restore selection mark and focused item */
    (*infoPtr).nSelectionMark = DPA_GetPtrIndex(
        (*infoPtr).hdpaItems,
        selectionMarkItem as *const core::ffi::c_void,
    ) as INT;
    (*infoPtr).nFocusedItem = DPA_GetPtrIndex(
        (*infoPtr).hdpaItems,
        focusedItem as *const core::ffi::c_void,
    ) as INT;

    /* I believe nHotItem should be left alone, see LISTVIEW_ShiftIndices */

    /* refresh the display */
    LISTVIEW_InvalidateList(infoPtr);
    return TRUE;
}

unsafe extern "C" fn LISTVIEW_ThemeChanged(infoPtr: *const LISTVIEW_INFO) -> LRESULT {
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    CloseThemeData(theme);
    OpenThemeData((*infoPtr).hwndSelf, core::ptr::addr_of!(themeClass) as LPCWSTR);
    return 0;
}

unsafe extern "C" fn LISTVIEW_Update(infoPtr: *mut LISTVIEW_INFO, nItem: INT) -> BOOL {
    if nItem < 0 || nItem >= (*infoPtr).nItemCount {
        return FALSE;
    }

    /* rearrange with default alignment style */
    if is_autoarrange(infoPtr) != 0 {
        LISTVIEW_Arrange(infoPtr, (LVA_DEFAULT) as INT);
    } else {
        LISTVIEW_InvalidateItem(infoPtr, nItem);
    }

    return TRUE;
}

unsafe extern "C" fn LISTVIEW_DrawTrackLine(infoPtr: *const LISTVIEW_INFO) -> BOOL {
    let mut hdc: HDC = core::ptr::null_mut::<HDC__>();
    if (*infoPtr).xTrackLine == -1 {
        return FALSE;
    }
    hdc = GetDC((*infoPtr).hwndSelf);
    if hdc.is_null() {
        return FALSE;
    }
    PatBlt(
        hdc,
        (*infoPtr).xTrackLine,
        (*infoPtr).rcList.top,
        1,
        (*infoPtr).rcList.bottom - (*infoPtr).rcList.top,
        DSTINVERT,
    );
    ReleaseDC((*infoPtr).hwndSelf, hdc);
    return TRUE;
}

unsafe extern "system" fn LISTVIEW_DelayedEditItem(hwnd: HWND, uMsg: UINT, idEvent: UINT_PTR, dwTime: DWORD) {
    let editItem: *mut DELAYED_ITEM_EDIT = idEvent as *mut DELAYED_ITEM_EDIT;
    let infoPtr: *mut LISTVIEW_INFO = GetWindowLongPtrW(hwnd, 0) as *mut LISTVIEW_INFO;

    KillTimer(hwnd, idEvent);
    (*editItem).fEnabled = FALSE;
    /* check if the item is still selected */
    if (*infoPtr).bFocus != 0
        && LISTVIEW_GetItemState(infoPtr, (*editItem).iItem, LVIS_SELECTED) != 0
    {
        LISTVIEW_EditLabelT(infoPtr, (*editItem).iItem, TRUE);
    }
}

unsafe extern "C" fn LISTVIEW_NCCreate(hwnd: HWND, wParam: WPARAM, lpcs: *const CREATESTRUCTW) -> LRESULT {
    let mut infoPtr: *mut LISTVIEW_INFO = core::ptr::null_mut();
    let mut logFont: LOGFONTW = core::mem::zeroed();

    /* initialize info pointer */
    infoPtr = Alloc(core::mem::size_of::<LISTVIEW_INFO>() as DWORD) as *mut LISTVIEW_INFO;
    if infoPtr.is_null() {
        return FALSE as LRESULT;
    }

    SetWindowLongPtrW(hwnd, 0 as ::core::ffi::c_int, infoPtr as DWORD_PTR as LONG_PTR);

    (*infoPtr).hwndSelf = hwnd;
    (*infoPtr).dwStyle = (*lpcs).style as DWORD;    /* Note: may be changed in WM_CREATE */
    map_style_view(infoPtr);
    /* determine the type of structures to use */
    (*infoPtr).hwndNotify = (*lpcs).hwndParent;
    /* infoPtr->notifyFormat will be filled in WM_CREATE */

    /* initialize color information  */
    (*infoPtr).clrBk = CLR_NONE as COLORREF;
    (*infoPtr).clrText = CLR_DEFAULT as COLORREF;
    (*infoPtr).clrTextBk = CLR_DEFAULT as COLORREF;
    LISTVIEW_SetBkColor(infoPtr, comctl32_color.clrWindow);
    (*infoPtr).bDefaultBkColor = TRUE as BOOL;

    /* set default values */
    (*infoPtr).nFocusedItem = -(1 as INT);
    (*infoPtr).nSelectionMark = -(1 as INT);
    (*infoPtr).nHotItem = -(1 as INT);
    (*infoPtr).redraw = TRUE as BOOL;
    (*infoPtr).bNoItemMetrics = TRUE as BOOL;
    (*infoPtr).notify_mask = NOTIFY_MASK_UNMASK_ALL as DWORD;
    (*infoPtr).autoSpacing = TRUE as BOOL;
    (*infoPtr).iconSpacing.cx = (GetSystemMetrics((SM_CXICONSPACING) as i32) - GetSystemMetrics((SM_CXICON) as i32)) as LONG;
    (*infoPtr).iconSpacing.cy = (GetSystemMetrics((SM_CYICONSPACING) as i32) - GetSystemMetrics((SM_CYICON) as i32)) as LONG;
    (*infoPtr).nEditLabelItem = -(1 as INT);
    (*infoPtr).nLButtonDownItem = -(1 as INT);
    (*infoPtr).dwHoverTime = HOVER_DEFAULT as DWORD; /* default system hover time */
    (*infoPtr).cWheelRemainder = 0 as INT;
    (*infoPtr).nMeasureItemHeight = 0 as INT;
    (*infoPtr).xTrackLine = -(1 as INT);  /* no track line */
    (*infoPtr).itemEdit.fEnabled = FALSE as BOOL;
    (*infoPtr).iVersion = COMCTL32_VERSION as DWORD;
    (*infoPtr).colRectsDirty = FALSE as BOOL;

    /* get default font (icon title) */
    SystemParametersInfoW(
        SPI_GETICONTITLELOGFONT as UINT,
        0 as UINT,
        core::ptr::addr_of_mut!(logFont) as PVOID,
        0 as UINT,
    );
    (*infoPtr).hDefaultFont = CreateFontIndirectW(core::ptr::addr_of_mut!(logFont));
    (*infoPtr).hFont = (*infoPtr).hDefaultFont;
    LISTVIEW_SaveTextMetrics(infoPtr);

    /* allocate memory for the data structure */
    'fail: {
        (*infoPtr).selectionRanges = ranges_create(10 as INT);
        if (*infoPtr).selectionRanges.is_null() {
            break 'fail;
        }
        (*infoPtr).hdpaItems = DPA_Create(10 as INT);
        if (*infoPtr).hdpaItems.is_null() {
            break 'fail;
        }
        (*infoPtr).hdpaItemIds = DPA_Create(10 as INT);
        if (*infoPtr).hdpaItemIds.is_null() {
            break 'fail;
        }
        (*infoPtr).hdpaPosX = DPA_Create(10 as INT);
        if (*infoPtr).hdpaPosX.is_null() {
            break 'fail;
        }
        (*infoPtr).hdpaPosY = DPA_Create(10 as INT);
        if (*infoPtr).hdpaPosY.is_null() {
            break 'fail;
        }
        (*infoPtr).hdpaColumns = DPA_Create(10 as INT);
        if (*infoPtr).hdpaColumns.is_null() {
            break 'fail;
        }

        return DefWindowProcW(hwnd, WM_NCCREATE as UINT, wParam, lpcs as LPARAM);
    }

    // fail:
    DestroyWindow((*infoPtr).hwndHeader);
    ranges_destroy((*infoPtr).selectionRanges);
    DPA_Destroy((*infoPtr).hdpaItems);
    DPA_Destroy((*infoPtr).hdpaItemIds);
    DPA_Destroy((*infoPtr).hdpaPosX);
    DPA_Destroy((*infoPtr).hdpaPosY);
    DPA_Destroy((*infoPtr).hdpaColumns);
    Free((infoPtr).cast());
    return FALSE as LRESULT;
}

unsafe extern "C" fn LISTVIEW_Create(hwnd: HWND, lpcs: *const CREATESTRUCTW) -> LRESULT {
    let mut infoPtr: *mut LISTVIEW_INFO = GetWindowLongPtrW(hwnd, 0 as ::core::ffi::c_int) as *mut LISTVIEW_INFO;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_Create\0" as *const u8 as *const ::core::ffi::c_char,
            9726 as ::core::ffi::c_int,
            b"(lpcs=%p, style=0x%08x)\n\0" as *const u8 as *const ::core::ffi::c_char,
            lpcs,
            (*lpcs).style,
        );
    }
    (*infoPtr).dwStyle = (*lpcs).style as DWORD;
    map_style_view(infoPtr);
    (*infoPtr).notifyFormat = SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFYFORMAT as UINT,
        (*infoPtr).hwndSelf as WPARAM,
        NF_QUERY as LPARAM,
    ) as SHORT;
    if (*infoPtr).notifyFormat as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        (*infoPtr).notifyFormat = NFR_ANSI as SHORT;
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
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_Create\0" as *const u8 as *const ::core::ffi::c_char,
            9735 as ::core::ffi::c_int,
            b"notify format=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).notifyFormat as ::core::ffi::c_int,
        );
    }
    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD
        && (((*lpcs).style as DWORD & (WS_VISIBLE as DWORD)) != 0)
    {
        if LISTVIEW_CreateHeader(infoPtr) < 0 as ::core::ffi::c_int {
            return -(1 as ::core::ffi::c_int) as LRESULT;
        }
    } else {
        (*infoPtr).hwndHeader = core::ptr::null_mut();
    }
    LISTVIEW_UpdateItemSize(infoPtr);
    LISTVIEW_UpdateSize(infoPtr);
    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
        if (LVS_NOCOLUMNHEADER as DWORD & (*lpcs).style as DWORD) == 0
            && ((WS_VISIBLE as DWORD & (*lpcs).style as DWORD) != 0)
        {
            ShowWindow((*infoPtr).hwndHeader, SW_SHOWNORMAL as ::core::ffi::c_int);
        }
        LISTVIEW_UpdateScroll(infoPtr);
        if ((*infoPtr).dwStyle & (LVS_OWNERDRAWFIXED as DWORD)) != 0 {
            notify_measureitem(infoPtr);
        }
    }
    OpenThemeData(hwnd, &raw const themeClass as LPCWSTR);
    set_icon_size(
        &raw mut (*infoPtr).iconSize,
        (*infoPtr).himlNormal,
        ((*infoPtr).uView != LV_VIEW_ICON as DWORD) as ::core::ffi::c_int,
    );
    set_icon_size(
        &raw mut (*infoPtr).iconStateSize,
        (*infoPtr).himlState,
        TRUE as ::core::ffi::c_int,
    );
    return 0 as LRESULT;
}

unsafe extern "C" fn LISTVIEW_Destroy(infoPtr: *mut LISTVIEW_INFO) -> LRESULT {
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    CloseThemeData(theme);

    /* delete all items */
    LISTVIEW_DeleteAllItems(infoPtr, TRUE);

    return 0;
}

unsafe extern "C" fn LISTVIEW_Enable(infoPtr: *const LISTVIEW_INFO) -> BOOL {
    if ((*infoPtr).dwStyle & (LVS_OWNERDRAWFIXED as DWORD)) != 0 {
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE);
    }
    return TRUE;
}

unsafe extern "C" fn LISTVIEW_EraseBkgnd(infoPtr: *const LISTVIEW_INFO, hdc: HDC) -> BOOL {
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    if GetClipBox(hdc, core::ptr::addr_of_mut!(rc)) == 0 {
        return FALSE;
    }

    if (*infoPtr).clrBk as ::core::ffi::c_ulong == CLR_NONE {
        if (*infoPtr).dwLvExStyle & LVS_EX_TRANSPARENTBKGND as DWORD != 0 {
            return SendMessageW(
                (*infoPtr).hwndNotify,
                WM_PRINTCLIENT as UINT,
                hdc as WPARAM,
                PRF_ERASEBKGND as LPARAM,
            ) as BOOL;
        } else {
            return SendMessageW(
                (*infoPtr).hwndNotify,
                WM_ERASEBKGND as UINT,
                hdc as WPARAM,
                0 as LPARAM,
            ) as BOOL;
        }
    }

    /* for double buffered controls we need to do this during refresh */
    if (*infoPtr).dwLvExStyle & LVS_EX_DOUBLEBUFFER as DWORD != 0 {
        return FALSE;
    }

    return LISTVIEW_FillBkgnd(infoPtr, hdc, core::ptr::addr_of_mut!(rc));
}

unsafe extern "C" fn scroll_list(infoPtr: *mut LISTVIEW_INFO, dx: INT, dy: INT) {
    /* now we can scroll the list */
    ScrollWindowEx(
        (*infoPtr).hwndSelf,
        dx as ::core::ffi::c_int,
        dy as ::core::ffi::c_int,
        core::ptr::addr_of_mut!((*infoPtr).rcList) as LPCRECT,
        core::ptr::addr_of_mut!((*infoPtr).rcList) as LPCRECT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (SW_ERASE | SW_INVALIDATE) as UINT,
    );
    /* if we have focus, adjust rect */
    OffsetRect(
        core::ptr::addr_of_mut!((*infoPtr).rcFocus),
        dx as ::core::ffi::c_int,
        dy as ::core::ffi::c_int,
    );
    UpdateWindow((*infoPtr).hwndSelf);
}

unsafe extern "C" fn LISTVIEW_VScroll(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut nScrollCode: INT,
    mut nScrollDiff: INT,
) -> LRESULT {
    let mut nOldScrollPos: INT = 0;
    let mut nNewScrollPos: INT = 0;
    let mut scrollInfo: SCROLLINFO = tagSCROLLINFO {
        cbSize: 0,
        fMask: 0,
        nMin: 0,
        nMax: 0,
        nPage: 0,
        nPos: 0,
        nTrackPos: 0,
    };
    let mut is_an_icon: BOOL = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_VScroll\0" as *const u8 as *const ::core::ffi::c_char,
            9892 as ::core::ffi::c_int,
            b"(nScrollCode=%d(%s), nScrollDiff=%d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            nScrollCode,
            debugscrollcode(nScrollCode as ::core::ffi::c_int),
            nScrollDiff,
        );
    }
    if !(*infoPtr).hwndEdit.is_null() {
        SendMessageW(
            (*infoPtr).hwndEdit,
            WM_KILLFOCUS as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
    }
    scrollInfo.cbSize = core::mem::size_of::<SCROLLINFO>() as UINT;
    scrollInfo.fMask = (SIF_PAGE | SIF_POS | SIF_RANGE | SIF_TRACKPOS) as UINT;
    is_an_icon = (((*infoPtr).uView == LV_VIEW_ICON as DWORD)
        || ((*infoPtr).uView == LV_VIEW_SMALLICON as DWORD)) as ::core::ffi::c_int
        as BOOL;
    if GetScrollInfo(
        (*infoPtr).hwndSelf,
        (SB_VERT) as i32,
        core::ptr::addr_of_mut!(scrollInfo),
    ) == 0
    {
        return 1 as LRESULT;
    }
    nOldScrollPos = scrollInfo.nPos as INT;
    match (((((((((nScrollCode) as u32) as i32) as u32) as i32) as u32) as i32) as u32) as i32) as u32 {
        _ if nScrollCode == SB_INTERNAL => {}
        SB_LINEUP => {
            nScrollDiff = (if is_an_icon != 0 {
                -(LISTVIEW_SCROLL_ICON_LINE_SIZE as ::core::ffi::c_int)
            } else {
                -(1 as ::core::ffi::c_int)
            }) as INT;
        }
        SB_LINEDOWN => {
            nScrollDiff = (if is_an_icon != 0 {
                LISTVIEW_SCROLL_ICON_LINE_SIZE as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            }) as INT;
        }
        SB_PAGEUP => {
            nScrollDiff = (scrollInfo.nPage as INT).wrapping_neg();
        }
        SB_PAGEDOWN => {
            nScrollDiff = scrollInfo.nPage as INT;
        }
        SB_THUMBPOSITION | SB_THUMBTRACK => {
            nScrollDiff = (scrollInfo.nTrackPos - scrollInfo.nPos) as INT;
        }
        _ => {
            nScrollDiff = 0 as INT;
        }
    }
    if nScrollDiff == 0 as INT {
        return 0 as LRESULT;
    }
    nNewScrollPos = scrollInfo.nPos as INT + nScrollDiff;
    if nScrollDiff > 0 as INT {
        if nNewScrollPos < nOldScrollPos || nNewScrollPos > scrollInfo.nMax as INT {
            nNewScrollPos = scrollInfo.nMax as INT;
        }
    } else if nNewScrollPos > nOldScrollPos || nNewScrollPos < scrollInfo.nMin as INT {
        nNewScrollPos = scrollInfo.nMin as INT;
    }
    scrollInfo.fMask = SIF_POS as UINT;
    scrollInfo.nPos = nNewScrollPos as ::core::ffi::c_int;
    nNewScrollPos = SetScrollInfo(
        (*infoPtr).hwndSelf,
        (SB_VERT) as i32,
        core::ptr::addr_of_mut!(scrollInfo) as LPCSCROLLINFO,
        TRUE,
    ) as INT;
    if nNewScrollPos == nOldScrollPos {
        return 0 as LRESULT;
    }
    nScrollDiff = nOldScrollPos - nNewScrollPos;
    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
        nScrollDiff *= (*infoPtr).nItemHeight;
    }
    scroll_list(infoPtr, 0 as INT, nScrollDiff);
    return 0 as LRESULT;
}

unsafe extern "C" fn LISTVIEW_HScroll(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut nScrollCode: INT,
    mut nScrollDiff: INT,
) -> LRESULT {
    let mut nOldScrollPos: INT = 0;
    let mut nNewScrollPos: INT = 0;
    let mut scrollInfo: SCROLLINFO = tagSCROLLINFO {
        cbSize: 0,
        fMask: 0,
        nMin: 0,
        nMax: 0,
        nPage: 0,
        nPos: 0,
        nTrackPos: 0,
    };
    let mut is_an_icon: BOOL = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_HScroll\0" as *const u8 as *const ::core::ffi::c_char,
            9996 as ::core::ffi::c_int,
            b"(nScrollCode=%d(%s), nScrollDiff=%d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            nScrollCode,
            debugscrollcode(nScrollCode as ::core::ffi::c_int),
            nScrollDiff,
        );
    }
    if !(*infoPtr).hwndEdit.is_null() {
        SendMessageW(
            (*infoPtr).hwndEdit,
            WM_KILLFOCUS as UINT,
            0 as WPARAM,
            0 as LPARAM,
        );
    }
    scrollInfo.cbSize = ::core::mem::size_of::<SCROLLINFO>() as UINT;
    scrollInfo.fMask = (SIF_PAGE | SIF_POS | SIF_RANGE | SIF_TRACKPOS) as UINT;
    is_an_icon = ((*infoPtr).uView == LV_VIEW_ICON as DWORD
        || (*infoPtr).uView == LV_VIEW_SMALLICON as DWORD) as ::core::ffi::c_int
        as BOOL;
    if GetScrollInfo(
        (*infoPtr).hwndSelf,
        SB_HORZ as INT,
        &raw mut scrollInfo,
    ) == 0
    {
        return 1 as LRESULT;
    }
    nOldScrollPos = scrollInfo.nPos as INT;
    match nScrollCode as u32 {
        _ if nScrollCode as u32 == SB_INTERNAL as u32 => {}
        SB_LINELEFT => {
            nScrollDiff = (if is_an_icon != 0 {
                -LISTVIEW_SCROLL_ICON_LINE_SIZE
            } else {
                -(1 as ::core::ffi::c_int)
            }) as INT;
        }
        SB_LINERIGHT => {
            nScrollDiff = (if is_an_icon != 0 {
                LISTVIEW_SCROLL_ICON_LINE_SIZE
            } else {
                1 as ::core::ffi::c_int
            }) as INT;
        }
        SB_PAGELEFT => {
            nScrollDiff = scrollInfo.nPage.wrapping_neg() as INT;
        }
        SB_PAGERIGHT => {
            nScrollDiff = scrollInfo.nPage as INT;
        }
        SB_THUMBPOSITION | SB_THUMBTRACK => {
            nScrollDiff = (scrollInfo.nTrackPos - scrollInfo.nPos) as INT;
        }
        _ => {
            nScrollDiff = 0 as ::core::ffi::c_int as INT;
        }
    }
    if nScrollDiff == 0 as ::core::ffi::c_int {
        return 0 as LRESULT;
    }
    nNewScrollPos = scrollInfo.nPos as INT + nScrollDiff;
    if nScrollDiff > 0 as ::core::ffi::c_int {
        if nNewScrollPos < nOldScrollPos || nNewScrollPos > scrollInfo.nMax {
            nNewScrollPos = scrollInfo.nMax as INT;
        }
    } else if nNewScrollPos > nOldScrollPos || nNewScrollPos < scrollInfo.nMin {
        nNewScrollPos = scrollInfo.nMin as INT;
    }
    scrollInfo.fMask = SIF_POS as UINT;
    scrollInfo.nPos = nNewScrollPos as ::core::ffi::c_int;
    nNewScrollPos = SetScrollInfo(
        (*infoPtr).hwndSelf,
        SB_HORZ as INT,
        &raw mut scrollInfo as LPCSCROLLINFO,
        TRUE as BOOL,
    ) as INT;
    if nNewScrollPos == nOldScrollPos {
        return 0 as LRESULT;
    }
    LISTVIEW_UpdateHeaderSize(infoPtr, nNewScrollPos);
    nScrollDiff = nOldScrollPos - nNewScrollPos;
    if (*infoPtr).uView == LV_VIEW_LIST as DWORD {
        nScrollDiff *= (*infoPtr).nItemWidth;
    }
    scroll_list(infoPtr, nScrollDiff, 0 as INT);
    return 0 as LRESULT;
}

unsafe extern "C" fn LISTVIEW_MouseWheel(
    infoPtr: *mut LISTVIEW_INFO,
    wheelDelta: INT,
) -> LRESULT {
    let mut pulScrollLines: INT = 3;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_MouseWheel\0" as *const u8 as *const ::core::ffi::c_char,
            10078 as ::core::ffi::c_int,
            b"(wheelDelta=%d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            wheelDelta,
        );
    }
    match (*infoPtr).uView {
        LV_VIEW_ICON | LV_VIEW_SMALLICON => {
            LISTVIEW_VScroll(
                infoPtr,
                SB_INTERNAL,
                if wheelDelta > 0 {
                    -LISTVIEW_SCROLL_ICON_LINE_SIZE
                } else {
                    LISTVIEW_SCROLL_ICON_LINE_SIZE
                },
            );
        }
        LV_VIEW_DETAILS => {
            SystemParametersInfoW(
                SPI_GETWHEELSCROLLLINES,
                0,
                core::ptr::addr_of_mut!(pulScrollLines) as PVOID,
                0,
            );
            if (wheelDelta < 0 && (*infoPtr).cWheelRemainder < 0)
                || (wheelDelta > 0 && (*infoPtr).cWheelRemainder > 0)
            {
                (*infoPtr).cWheelRemainder += wheelDelta;
            } else {
                (*infoPtr).cWheelRemainder = wheelDelta;
            }
            if (*infoPtr).cWheelRemainder != 0 && pulScrollLines != 0 {
                let mut cLineScroll: ::core::ffi::c_int = 0;
                pulScrollLines = min(
                    LISTVIEW_GetCountPerColumn(infoPtr) as UINT,
                    pulScrollLines as UINT,
                ) as INT;
                cLineScroll = pulScrollLines * (*infoPtr).cWheelRemainder / (WHEEL_DELTA) as i32;
                (*infoPtr).cWheelRemainder -= (WHEEL_DELTA * (cLineScroll) as u32 / (pulScrollLines) as u32) as i32;
                LISTVIEW_VScroll(infoPtr, SB_INTERNAL, -cLineScroll);
            }
        }
        LV_VIEW_LIST => {
            LISTVIEW_HScroll(
                infoPtr,
                if wheelDelta > 0 {
                    (SB_LINELEFT) as i32
                } else {
                    (SB_LINERIGHT) as i32
                },
                0,
            );
        }
        _ => {}
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn LISTVIEW_KeyDown(infoPtr: *mut LISTVIEW_INFO, nVirtualKey: INT, lKeyData: LONG) -> LRESULT {
    let hwndSelf: HWND = (*infoPtr).hwndSelf;
    let mut nItem: INT = -1;
    let mut nmKeyDown: NMLVKEYDOWN = core::mem::zeroed();

    /* send LVN_KEYDOWN notification */
    nmKeyDown.wVKey = nVirtualKey as WORD;
    nmKeyDown.flags = 0;
    notify_hdr(infoPtr, LVN_KEYDOWN, core::ptr::addr_of_mut!(nmKeyDown.hdr));
    if IsWindow(hwndSelf) == 0 {
        return 0;
    }

    match (nVirtualKey) as u32 {
        VK_SPACE => {
            nItem = (*infoPtr).nFocusedItem;
            if ((*infoPtr).dwLvExStyle & LVS_EX_CHECKBOXES) != 0 {
                toggle_checkbox_state(infoPtr, (*infoPtr).nFocusedItem);
            }
        }

        VK_RETURN => {
            if ((*infoPtr).nItemCount > 0) && ((*infoPtr).nFocusedItem != -1) {
                if notify(infoPtr, NM_RETURN as INT) == 0 {
                    return 0;
                }
                if notify(infoPtr, LVN_ITEMACTIVATE) == 0 {
                    return 0;
                }
            }
        }

        VK_HOME => {
            if (*infoPtr).nItemCount > 0 {
                nItem = 0;
            }
        }

        VK_END => {
            if (*infoPtr).nItemCount > 0 {
                nItem = (*infoPtr).nItemCount - 1;
            }
        }

        VK_LEFT => {
            nItem = LISTVIEW_GetNextItem(infoPtr, (*infoPtr).nFocusedItem, LVNI_TOLEFT);
        }

        VK_UP => {
            nItem = LISTVIEW_GetNextItem(infoPtr, (*infoPtr).nFocusedItem, LVNI_ABOVE);
        }

        VK_RIGHT => {
            nItem = LISTVIEW_GetNextItem(infoPtr, (*infoPtr).nFocusedItem, LVNI_TORIGHT);
        }

        VK_DOWN => {
            nItem = LISTVIEW_GetNextItem(infoPtr, (*infoPtr).nFocusedItem, LVNI_BELOW);
        }

        VK_PRIOR => {
            if (*infoPtr).uView == LV_VIEW_DETAILS {
                let topidx: INT = LISTVIEW_GetTopIndex(infoPtr);
                if (*infoPtr).nFocusedItem == topidx {
                    nItem = topidx - LISTVIEW_GetCountPerColumn(infoPtr) + 1;
                } else {
                    nItem = topidx;
                }
            } else {
                nItem = (*infoPtr).nFocusedItem
                    - LISTVIEW_GetCountPerColumn(infoPtr) * LISTVIEW_GetCountPerRow(infoPtr);
            }
            if nItem < 0 {
                nItem = 0;
            }
        }

        VK_NEXT => {
            if (*infoPtr).uView == LV_VIEW_DETAILS {
                let topidx: INT = LISTVIEW_GetTopIndex(infoPtr);
                let cnt: INT = LISTVIEW_GetCountPerColumn(infoPtr);
                if (*infoPtr).nFocusedItem == topidx + cnt - 1 {
                    nItem = (*infoPtr).nFocusedItem + cnt - 1;
                } else {
                    nItem = topidx + cnt - 1;
                }
            } else {
                nItem = (*infoPtr).nFocusedItem
                    + LISTVIEW_GetCountPerColumn(infoPtr) * LISTVIEW_GetCountPerRow(infoPtr);
            }
            if nItem >= (*infoPtr).nItemCount {
                nItem = (*infoPtr).nItemCount - 1;
            }
        }

        _ => {}
    }

    if (nItem != -1)
        && (nItem != (*infoPtr).nFocusedItem || (nVirtualKey) as u32 == VK_SPACE)
    {
        let mut keys: DWORD = 0;

        if (GetKeyState(VK_SHIFT as INT) as INT & 0x8000) != 0 {
            keys |= SHIFT_KEY;
        }
        if (GetKeyState(VK_CONTROL as INT) as INT & 0x8000) != 0 {
            keys |= CTRL_KEY;
        }
        if (nVirtualKey) as u32 == VK_SPACE {
            keys |= SPACE_KEY;
        }

        LISTVIEW_KeySelection(infoPtr, nItem, keys);
    }

    return 0;
}

unsafe extern "C" fn LISTVIEW_KillFocus(infoPtr: *mut LISTVIEW_INFO) -> LRESULT {
    /* drop any left over scroll amount */
    (*infoPtr).cWheelRemainder = 0 as INT;

    /* if we did not have the focus, there's nothing more to do */
    if (*infoPtr).bFocus == 0 {
        return 0 as LRESULT;
    }

    /* send NM_KILLFOCUS notification */
    if notify(infoPtr, NM_KILLFOCUS as INT) == 0 {
        return 0 as LRESULT;
    }

    /* if we have a focus rectangle, get rid of it */
    LISTVIEW_ShowFocusRect(infoPtr, FALSE as BOOL);

    /* if have a marquee selection, stop it */
    if (*infoPtr).bMarqueeSelect != 0 {
        /* Remove the marquee rectangle and release our mouse capture */
        LISTVIEW_InvalidateRect(infoPtr, core::ptr::addr_of!((*infoPtr).marqueeRect));
        ReleaseCapture();

        SetRectEmpty(core::ptr::addr_of_mut!((*infoPtr).marqueeRect));

        (*infoPtr).bMarqueeSelect = FALSE as BOOL;
        (*infoPtr).bScrolling = FALSE as BOOL;
        KillTimer((*infoPtr).hwndSelf, infoPtr as UINT_PTR);
    }

    /* set window focus flag */
    (*infoPtr).bFocus = FALSE as BOOL;

    /* invalidate the selected items before resetting focus flag */
    LISTVIEW_InvalidateSelectedItems(infoPtr);

    return 0 as LRESULT;
}

unsafe extern "C" fn LISTVIEW_LButtonDblClk(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut wKey: WORD,
    mut x: INT,
    mut y: INT,
) -> LRESULT {
    let mut htInfo: LVHITTESTINFO = core::mem::zeroed();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_LButtonDblClk\0" as *const u8 as *const ::core::ffi::c_char,
            10301 as ::core::ffi::c_int,
            b"(key=%hu, X=%u, Y=%u)\n\0" as *const u8 as *const ::core::ffi::c_char,
            wKey as ::core::ffi::c_int,
            x,
            y,
        );
    }
    if (*infoPtr).itemEdit.fEnabled != 0 {
        KillTimer(
            (*infoPtr).hwndSelf,
            &raw mut (*infoPtr).itemEdit as UINT_PTR,
        );
        (*infoPtr).itemEdit.fEnabled = FALSE as BOOL;
    }
    if notify(infoPtr, NM_RELEASEDCAPTURE as INT) == 0 {
        return 0 as LRESULT;
    }
    htInfo.pt.x = x as LONG;
    htInfo.pt.y = y as LONG;
    LISTVIEW_HitTest(
        infoPtr,
        &raw mut htInfo,
        TRUE as BOOL,
        FALSE as BOOL,
    );
    if notify_click(
        infoPtr,
        NM_DBLCLK as INT,
        &raw mut htInfo,
    ) == 0
    {
        return 0 as LRESULT;
    }
    if htInfo.iItem != -(1 as INT) {
        notify_itemactivate(infoPtr, &raw mut htInfo);
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn LISTVIEW_TrackMouse(infoPtr: *const LISTVIEW_INFO, mut pt: POINT) -> LRESULT {
    let mut msg: MSG = core::mem::zeroed();
    let mut r: RECT = core::mem::zeroed();

    r.bottom = pt.y;
    r.top = r.bottom;
    r.right = pt.x;
    r.left = r.right;

    InflateRect(
        core::ptr::addr_of_mut!(r),
        GetSystemMetrics((SM_CXDRAG) as i32),
        GetSystemMetrics((SM_CYDRAG) as i32),
    );

    SetCapture((*infoPtr).hwndSelf);

    while 1 != 0 {
        if PeekMessageW(
            core::ptr::addr_of_mut!(msg),
            core::ptr::null_mut(),
            0,
            0,
            PM_REMOVE | PM_NOYIELD,
        ) != 0
        {
            if msg.message == WM_MOUSEMOVE {
                pt.x = LOWORD!(msg.lParam as DWORD) as ::core::ffi::c_short as LONG;
                pt.y = HIWORD!(msg.lParam as DWORD) as ::core::ffi::c_short as LONG;
                if PtInRect(core::ptr::addr_of_mut!(r) as LPCRECT, pt) != 0 {
                    continue;
                } else {
                    ReleaseCapture();
                    return 1;
                }
            } else if msg.message >= WM_LBUTTONDOWN
                && msg.message <= WM_RBUTTONDBLCLK
            {
                break;
            }

            DispatchMessageW(core::ptr::addr_of_mut!(msg));
        }

        if GetCapture() != (*infoPtr).hwndSelf {
            return 0;
        }
    }

    ReleaseCapture();
    return 0;
}

unsafe extern "C" fn LISTVIEW_LButtonDown(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut wKey: WORD,
    mut x: INT,
    mut y: INT,
) -> LRESULT {
    let mut lvHitTestInfo: LVHITTESTINFO = tagLVHITTESTINFO {
        pt: tagPOINT { x: 0, y: 0 },
        flags: 0,
        iItem: 0,
        iSubItem: 0,
    };
    static mut bGroupSelect: BOOL = TRUE;
    let mut pt: POINT = tagPOINT {
        x: x as LONG,
        y: y as LONG,
    };
    let mut nItem: INT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_LButtonDown\0" as *const u8 as *const ::core::ffi::c_char,
            10391 as ::core::ffi::c_int,
            b"(key=%hu, X=%u, Y=%u)\n\0" as *const u8 as *const ::core::ffi::c_char,
            wKey as ::core::ffi::c_int,
            x,
            y,
        );
    }
    /* send NM_RELEASEDCAPTURE notification */
    if notify(infoPtr, NM_RELEASEDCAPTURE as INT) == 0 {
        return 0;
    }
    /* set left button down flag and record the click position */
    (*infoPtr).bLButtonDown = TRUE as BOOL;
    (*infoPtr).ptClickPos = pt;
    (*infoPtr).bDragging = FALSE as BOOL;
    (*infoPtr).bMarqueeSelect = FALSE as BOOL;
    (*infoPtr).bScrolling = FALSE as BOOL;

    lvHitTestInfo.pt.x = x as LONG;
    lvHitTestInfo.pt.y = y as LONG;

    nItem = LISTVIEW_HitTest(
        infoPtr,
        &raw mut lvHitTestInfo,
        TRUE as BOOL,
        TRUE as BOOL,
    );
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_LButtonDown\0" as *const u8 as *const ::core::ffi::c_char,
            10407 as ::core::ffi::c_int,
            b"at %s, nItem=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_point(&raw mut pt),
            nItem,
        );
    }
    if nItem >= 0 && nItem < (*infoPtr).nItemCount {
        if ((*infoPtr).dwLvExStyle & (LVS_EX_CHECKBOXES as DWORD)) != 0
            && (lvHitTestInfo.flags & (LVHT_ONITEMSTATEICON as UINT)) != 0
        {
            notify_click(infoPtr, NM_CLICK as INT, &raw mut lvHitTestInfo);
            toggle_checkbox_state(infoPtr, nItem);
            (*infoPtr).bLButtonDown = FALSE as BOOL;
            return 0;
        }

        if ((*infoPtr).dwStyle & (LVS_SINGLESEL as DWORD)) != 0 {
            if LISTVIEW_GetItemState(infoPtr, nItem, LVIS_SELECTED as UINT) != 0 {
                (*infoPtr).nEditLabelItem = nItem;
            } else {
                LISTVIEW_SetSelection(infoPtr, nItem);
            }
        } else if ((wKey as UINT) & MK_CONTROL) != 0
            && ((wKey as UINT) & MK_SHIFT) != 0
        {
            if bGroupSelect != 0 {
                if LISTVIEW_AddGroupSelection(infoPtr, nItem) == 0 {
                    return 0;
                }
                LISTVIEW_SetItemFocus(infoPtr, nItem);
                (*infoPtr).nSelectionMark = nItem;
            } else {
                let mut item: LVITEMW = tagLVITEMW {
                    mask: 0,
                    iItem: 0,
                    iSubItem: 0,
                    state: 0,
                    stateMask: 0,
                    pszText: ::core::ptr::null_mut::<WCHAR>(),
                    cchTextMax: 0,
                    iImage: 0,
                    lParam: 0,
                    iIndent: 0,
                    iGroupId: 0,
                    cColumns: 0,
                    puColumns: ::core::ptr::null_mut::<::core::ffi::c_uint>(),
                };
                item.state = (LVIS_SELECTED | LVIS_FOCUSED) as UINT;
                item.stateMask = (LVIS_SELECTED | LVIS_FOCUSED) as UINT;

                LISTVIEW_SetItemState(infoPtr, nItem, &raw mut item);
                (*infoPtr).nSelectionMark = nItem;
            }
        } else if (wKey as UINT) & MK_CONTROL != 0 {
            let mut item_0: LVITEMW = tagLVITEMW {
                mask: 0,
                iItem: 0,
                iSubItem: 0,
                state: 0,
                stateMask: 0,
                pszText: ::core::ptr::null_mut::<WCHAR>(),
                cchTextMax: 0,
                iImage: 0,
                lParam: 0,
                iIndent: 0,
                iGroupId: 0,
                cColumns: 0,
                puColumns: ::core::ptr::null_mut::<::core::ffi::c_uint>(),
            };
            bGroupSelect = (LISTVIEW_GetItemState(infoPtr, nItem, LVIS_SELECTED as UINT)
                == 0 as UINT) as ::core::ffi::c_int as BOOL;

            item_0.state = ((if bGroupSelect != 0 {
                LVIS_SELECTED
            } else {
                0
            }) | LVIS_FOCUSED) as UINT;
            item_0.stateMask = (LVIS_SELECTED | LVIS_FOCUSED) as UINT;
            LISTVIEW_SetItemState(infoPtr, nItem, &raw mut item_0);
            (*infoPtr).nSelectionMark = nItem;
        } else if (wKey as UINT) & MK_SHIFT != 0 {
            LISTVIEW_SetGroupSelection(infoPtr, nItem);
        } else {
            if LISTVIEW_GetItemState(infoPtr, nItem, LVIS_SELECTED as UINT) != 0 {
                (*infoPtr).nEditLabelItem = nItem;
                (*infoPtr).nLButtonDownItem = nItem;

                LISTVIEW_SetItemFocus(infoPtr, nItem);
            } else {
                /* set selection (clears other pre-existing selections) */
                LISTVIEW_SetSelection(infoPtr, nItem);
            }
        }

        if (*infoPtr).bFocus == 0 {
            SetFocus((*infoPtr).hwndSelf);
        }

        if ((*infoPtr).dwLvExStyle & (LVS_EX_ONECLICKACTIVATE as DWORD)) != 0 {
            if lvHitTestInfo.iItem != -1 {
                notify_itemactivate(infoPtr, &raw mut lvHitTestInfo);
            }
        }
    } else {
        if (*infoPtr).bFocus == 0 {
            SetFocus((*infoPtr).hwndSelf);
        }

        /* remove all selections */
        if (wKey as UINT) & MK_CONTROL == 0
            && (wKey as UINT) & MK_SHIFT == 0
        {
            LISTVIEW_DeselectAll(infoPtr);
        }
        ReleaseCapture();
    }

    return 0;
}

unsafe extern "C" fn LISTVIEW_LButtonUp(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut wKey: WORD,
    mut x: INT,
    mut y: INT,
) -> LRESULT {
    let mut lvHitTestInfo: LVHITTESTINFO = tagLVHITTESTINFO {
        pt: tagPOINT { x: 0, y: 0 },
        flags: 0,
        iItem: 0,
        iSubItem: 0,
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
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_LButtonUp\0" as *const u8 as *const ::core::ffi::c_char,
            10512 as ::core::ffi::c_int,
            b"(key=%hu, X=%u, Y=%u)\n\0" as *const u8 as *const ::core::ffi::c_char,
            wKey as ::core::ffi::c_int,
            x,
            y,
        );
    }

    if (*infoPtr).bLButtonDown == 0 {
        return 0;
    }

    lvHitTestInfo.pt.x = x as LONG;
    lvHitTestInfo.pt.y = y as LONG;

    /* send NM_CLICK notification */
    LISTVIEW_HitTest(
        infoPtr,
        core::ptr::addr_of_mut!(lvHitTestInfo),
        TRUE,
        FALSE,
    );
    if notify_click(
        infoPtr,
        NM_CLICK as INT,
        core::ptr::addr_of_mut!(lvHitTestInfo),
    ) == 0
    {
        return 0;
    }

    /* set left button flag */
    (*infoPtr).bLButtonDown = FALSE;

    /* set a single selection, reset others */
    if lvHitTestInfo.iItem == (*infoPtr).nLButtonDownItem
        && lvHitTestInfo.iItem != -(1)
    {
        LISTVIEW_SetSelection(infoPtr, (*infoPtr).nLButtonDownItem);
    }
    (*infoPtr).nLButtonDownItem = -(1);

    if (*infoPtr).bDragging != 0 || (*infoPtr).bMarqueeSelect != 0 {
        /* Remove the marquee rectangle and release our mouse capture */
        if (*infoPtr).bMarqueeSelect != 0 {
            LISTVIEW_InvalidateRect(infoPtr, core::ptr::addr_of_mut!((*infoPtr).marqueeDrawRect));
            ReleaseCapture();
        }

        SetRectEmpty(core::ptr::addr_of_mut!((*infoPtr).marqueeRect));
        SetRectEmpty(core::ptr::addr_of_mut!((*infoPtr).marqueeDrawRect));

        (*infoPtr).bDragging = FALSE;
        (*infoPtr).bMarqueeSelect = FALSE;
        (*infoPtr).bScrolling = FALSE;

        KillTimer((*infoPtr).hwndSelf, infoPtr as UINT_PTR);
        return 0;
    }

    /* if we clicked on a selected item, edit the label */
    if lvHitTestInfo.iItem == (*infoPtr).nEditLabelItem
        && (lvHitTestInfo.flags & (LVHT_ONITEMLABEL as UINT)) != 0
    {
        /* we want to make sure the user doesn't want to do a double click. So we will
         * delay the edit. WM_LBUTTONDBLCLICK will cancel the timer
         */
        (*infoPtr).itemEdit.fEnabled = TRUE;
        (*infoPtr).itemEdit.iItem = lvHitTestInfo.iItem as INT;
        SetTimer(
            (*infoPtr).hwndSelf,
            core::ptr::addr_of_mut!((*infoPtr).itemEdit) as UINT_PTR,
            GetDoubleClickTime(),
            Some(
                LISTVIEW_DelayedEditItem,
            ),
        );
    }

    return 0;
}

unsafe extern "C" fn LISTVIEW_NCDestroy(infoPtr: *mut LISTVIEW_INFO) -> LRESULT {
    let mut i: INT;

    /* destroy data structure */
    DPA_Destroy((*infoPtr).hdpaItems);
    DPA_Destroy((*infoPtr).hdpaItemIds);
    DPA_Destroy((*infoPtr).hdpaPosX);
    DPA_Destroy((*infoPtr).hdpaPosY);
    /* columns */
    i = 0;
    while i < DPA_GetPtrCount((*infoPtr).hdpaColumns) {
        Free(DPA_GetPtr((*infoPtr).hdpaColumns, i));
        i += 1;
    }
    DPA_Destroy((*infoPtr).hdpaColumns);
    ranges_destroy((*infoPtr).selectionRanges);
    (*infoPtr).selectionRanges = core::ptr::null_mut();

    /* destroy image lists */
    if ((*infoPtr).dwStyle & LVS_SHAREIMAGELISTS) == 0 {
        ImageList_Destroy((*infoPtr).himlNormal);
        ImageList_Destroy((*infoPtr).himlSmall);
        ImageList_Destroy((*infoPtr).himlState);
    }

    /* destroy font, bkgnd brush */
    (*infoPtr).hFont = core::ptr::null_mut();
    if !(*infoPtr).hDefaultFont.is_null() {
        DeleteObject(((*infoPtr).hDefaultFont).cast());
    }
    if (*infoPtr).clrBk != CLR_NONE {
        DeleteObject(((*infoPtr).hBkBrush).cast());
    }

    SetWindowLongPtrW((*infoPtr).hwndSelf, 0, 0);

    /* free listview info pointer*/
    Free(infoPtr as LPVOID);

    return 0;
}

unsafe extern "C" fn LISTVIEW_Notify(infoPtr: *mut LISTVIEW_INFO, lpnmhdr: *mut NMHDR) -> LRESULT {
    let mut lpnmh: *mut NMHEADERW = core::ptr::null_mut();

    if lpnmhdr.is_null() || (*lpnmhdr).hwndFrom != (*infoPtr).hwndHeader {
        return 0 as LRESULT;
    }

    /* remember: HDN_LAST < HDN_FIRST */
    if (*lpnmhdr).code > HDN_FIRST as UINT || (*lpnmhdr).code < HDN_LAST as UINT {
        return 0 as LRESULT;
    }
    lpnmh = lpnmhdr as *mut NMHEADERW;

    if (*lpnmh).iItem < 0 as INT
        || (*lpnmh).iItem >= DPA_GetPtrCount((*infoPtr).hdpaColumns)
    {
        return 0 as LRESULT;
    }

    match ((*lpnmhdr).code) as i32 {
        HDN_TRACKW | HDN_TRACKA => {
            let mut lpColumnInfo: *mut COLUMN_INFO = core::ptr::null_mut();
            let mut ptOrigin: POINT = core::mem::zeroed();
            let mut x: INT = 0;

            if !((*lpnmh).pitem.is_null()
                || ((*(*lpnmh).pitem).mask & (HDI_WIDTH as UINT)) == 0)
            {
                /* remove the old line (if any) */
                LISTVIEW_DrawTrackLine(infoPtr);

                /* compute & draw the new line */
                lpColumnInfo = LISTVIEW_GetColumnInfo(infoPtr, (*lpnmh).iItem as INT);
                x = ((*lpColumnInfo).rcHeader.left + (*(*lpnmh).pitem).cxy) as INT;
                LISTVIEW_GetOrigin(infoPtr, &raw mut ptOrigin);
                (*infoPtr).xTrackLine = (x + ptOrigin.x) as INT;
                LISTVIEW_DrawTrackLine(infoPtr);
                return notify_forward_header(infoPtr, lpnmh);
            }
        }

        HDN_ENDTRACKA | HDN_ENDTRACKW => {
            /* remove the track line (if any) */
            LISTVIEW_DrawTrackLine(infoPtr);
            (*infoPtr).xTrackLine = -(1 as INT);
            return notify_forward_header(infoPtr, lpnmh);
        }

        HDN_BEGINDRAG => {
            if ((*infoPtr).dwLvExStyle & (LVS_EX_HEADERDRAGDROP as DWORD))
                == (0 as DWORD)
            {
                return 1 as LRESULT;
            }
            return notify_forward_header(infoPtr, lpnmh);
        }

        HDN_ENDDRAG => {
            (*infoPtr).colRectsDirty = TRUE as BOOL;
            LISTVIEW_InvalidateList(infoPtr);
            return notify_forward_header(infoPtr, lpnmh);
        }

        HDN_ITEMCHANGEDW | HDN_ITEMCHANGEDA => {
            let mut lpColumnInfo_0: *mut COLUMN_INFO = core::ptr::null_mut();
            let mut hdi: HDITEMW = core::mem::zeroed();
            let mut dx: INT = 0;
            let mut cxy: INT = 0;

            if (*lpnmh).pitem.is_null()
                || ((*(*lpnmh).pitem).mask & (HDI_WIDTH as UINT)) == 0
            {
                hdi.mask = HDI_WIDTH as UINT;
                if SendMessageW(
                    (*infoPtr).hwndHeader,
                    HDM_GETITEMW as UINT,
                    (*lpnmh).iItem as WPARAM,
                    &raw mut hdi as LPARAM,
                ) == 0
                {
                    return 0 as LRESULT;
                }
                cxy = hdi.cxy as INT;
            } else {
                cxy = (*(*lpnmh).pitem).cxy as INT;
            }

            /* determine how much we change since the last know position */
            lpColumnInfo_0 = LISTVIEW_GetColumnInfo(infoPtr, (*lpnmh).iItem as INT);
            dx = (cxy - ((*lpColumnInfo_0).rcHeader.right - (*lpColumnInfo_0).rcHeader.left)) as INT;
            if dx != 0 as INT {
                (*lpColumnInfo_0).rcHeader.right += dx as LONG;

                hdi.mask = HDI_ORDER as UINT;
                SendMessageW(
                    (*infoPtr).hwndHeader,
                    HDM_GETITEMW as UINT,
                    (*lpnmh).iItem as WPARAM,
                    &raw mut hdi as LPARAM,
                );

                /* not the rightmost one */
                if hdi.iOrder + (1 as INT)
                    < DPA_GetPtrCount((*infoPtr).hdpaColumns)
                {
                    let mut nIndex: INT = SendMessageW(
                        (*infoPtr).hwndHeader,
                        HDM_ORDERTOINDEX as UINT,
                        (hdi.iOrder + (1 as INT)) as WPARAM,
                        0 as LPARAM,
                    ) as INT;
                    LISTVIEW_ScrollColumns(infoPtr, nIndex, dx);
                } else {
                    /* only needs to update the scrolls */
                    (*infoPtr).nItemWidth += dx;
                    LISTVIEW_UpdateScroll(infoPtr);
                }
                LISTVIEW_UpdateItemSize(infoPtr);
                if (*infoPtr).uView == (LV_VIEW_DETAILS as DWORD)
                    && is_redrawing(infoPtr) != 0
                {
                    let mut ptOrigin_0: POINT = core::mem::zeroed();
                    let mut rcCol: RECT = (*lpColumnInfo_0).rcHeader;

                    LISTVIEW_GetOrigin(infoPtr, &raw mut ptOrigin_0);
                    OffsetRect(&raw mut rcCol, ptOrigin_0.x, 0 as INT);

                    rcCol.top = (*infoPtr).rcList.top;
                    rcCol.bottom = (*infoPtr).rcList.bottom;

                    /* resizing left-aligned columns leaves most of the left side untouched */
                    if ((*lpColumnInfo_0).fmt as INT & (LVCFMT_JUSTIFYMASK as INT))
                        == (LVCFMT_LEFT as INT)
                    {
                        let mut nMaxDirty: INT =
                            (*infoPtr).nEllipsisWidth + (*infoPtr).ntmMaxCharWidth;
                        if dx > 0 as INT {
                            nMaxDirty += dx;
                        }
                        rcCol.left = max(rcCol.left, rcCol.right - nMaxDirty);
                    }

                    /* when shrinking the last column clear the now unused field */
                    if hdi.iOrder
                        == DPA_GetPtrCount((*infoPtr).hdpaColumns) - (1 as INT)
                    {
                        let mut right: RECT = core::mem::zeroed();

                        rcCol.right -= dx as LONG;

                        /* deal with right from rightmost column area */
                        right.left = rcCol.right;
                        right.top = rcCol.top;
                        right.bottom = rcCol.bottom;
                        right.right = (*infoPtr).rcList.right;

                        LISTVIEW_InvalidateRect(infoPtr, &raw mut right);
                    }

                    LISTVIEW_InvalidateRect(infoPtr, &raw mut rcCol);
                }
            }
        }

        HDN_ITEMCLICKW | HDN_ITEMCLICKA => {
            /* Handle sorting by Header Column */
            let mut nmlv: NMLISTVIEW = core::mem::zeroed();

            ZeroMemory!(
                &raw mut nmlv as *mut ::core::ffi::c_void,
                core::mem::size_of::<NMLISTVIEW>(),
            );
            nmlv.iItem = -(1 as INT);
            nmlv.iSubItem = (*lpnmh).iItem;
            notify_listview(infoPtr, LVN_COLUMNCLICK as INT, &raw mut nmlv);
            return notify_forward_header(infoPtr, lpnmh);
        }

        HDN_DIVIDERDBLCLICKW | HDN_DIVIDERDBLCLICKA => {
            /* FIXME: for LVS_EX_HEADERINALLVIEWS and not LV_VIEW_DETAILS
                      we should use LVSCW_AUTOSIZE_USEHEADER, helper rework or
                      split needed for that */
            LISTVIEW_SetColumnWidth(infoPtr, (*lpnmh).iItem as INT, LVSCW_AUTOSIZE as INT);
            return notify_forward_header(infoPtr, lpnmh);
        }

        _ => {}
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn LISTVIEW_NCPaint(infoPtr: *const LISTVIEW_INFO, region: HRGN) -> BOOL {
    let mut theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    let mut dc: HDC = core::ptr::null_mut::<HDC__>();
    let mut r: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut cliprgn: HRGN = core::ptr::null_mut::<HRGN__>();
    let mut cxEdge: core::ffi::c_int = GetSystemMetrics((SM_CXEDGE) as i32);
    let mut cyEdge: core::ffi::c_int = GetSystemMetrics((SM_CYEDGE) as i32);

    if theme.is_null() {
        return DefWindowProcW(
            (*infoPtr).hwndSelf,
            WM_NCPAINT as UINT,
            region as WPARAM,
            0 as LPARAM,
        ) as BOOL;
    }

    GetWindowRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(r));

    cliprgn = CreateRectRgn(
        r.left + cxEdge,
        r.top + cyEdge,
        r.right - cxEdge,
        r.bottom - cyEdge,
    );
    if region != 1 as HRGN {
        CombineRgn(cliprgn, cliprgn, region, (RGN_AND as i32));
    }
    OffsetRect(core::ptr::addr_of_mut!(r), -r.left, -r.top);

    dc = GetWindowDC((*infoPtr).hwndSelf);
    /* Exclude client part */
    ExcludeClipRect(
        dc,
        r.left + cxEdge,
        r.top + cyEdge,
        r.right - cxEdge,
        r.bottom - cyEdge,
    );

    if IsThemeBackgroundPartiallyTransparent(theme, 0, 0) != 0 {
        DrawThemeParentBackground((*infoPtr).hwndSelf, dc, core::ptr::addr_of_mut!(r));
    }
    DrawThemeBackground(
        theme,
        dc,
        0,
        0,
        core::ptr::addr_of_mut!(r),
        0 as *const RECT,
    );
    ReleaseDC((*infoPtr).hwndSelf, dc);

    /* Call default proc to get the scrollbars etc. painted */
    DefWindowProcW(
        (*infoPtr).hwndSelf,
        WM_NCPAINT as UINT,
        cliprgn as WPARAM,
        0 as LPARAM,
    );
    DeleteObject(cliprgn as HGDIOBJ);

    return FALSE as BOOL;
}

unsafe extern "C" fn LISTVIEW_NotifyFormat(infoPtr: *mut LISTVIEW_INFO, hwndFrom: HWND, nCommand: INT) -> LRESULT {
    if (nCommand) as u32 == NF_REQUERY {
        (*infoPtr).notifyFormat = SendMessageW(
            (*infoPtr).hwndNotify,
            WM_NOTIFYFORMAT as UINT,
            (*infoPtr).hwndSelf as WPARAM,
            NF_QUERY as LPARAM,
        ) as SHORT;
    }
    return (*infoPtr).notifyFormat as LRESULT;
}

unsafe extern "C" fn LISTVIEW_Paint(infoPtr: *mut LISTVIEW_INFO, mut hdc: HDC) -> LRESULT {
    if (*infoPtr).bNoItemMetrics != 0 && (*infoPtr).nItemCount != 0 {
        (*infoPtr).bNoItemMetrics = FALSE;
        LISTVIEW_UpdateItemSize(infoPtr);
        if (*infoPtr).uView == LV_VIEW_ICON || (*infoPtr).uView == LV_VIEW_SMALLICON {
            LISTVIEW_Arrange(infoPtr, (LVA_DEFAULT) as INT);
        }
        LISTVIEW_UpdateScroll(infoPtr);
    }

    if !(*infoPtr).hwndHeader.is_null() {
        UpdateWindow((*infoPtr).hwndHeader);
    }

    if !hdc.is_null() {
        LISTVIEW_Refresh(infoPtr, hdc, core::ptr::null());
    } else {
        let mut ps: PAINTSTRUCT = core::mem::zeroed();

        hdc = BeginPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
        if hdc.is_null() {
            return 1;
        }
        LISTVIEW_Refresh(
            infoPtr,
            hdc,
            if ps.fErase != 0 {
                core::ptr::addr_of_mut!(ps.rcPaint)
            } else {
                core::ptr::null_mut()
            },
        );
        EndPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
    }

    return 0;
}

unsafe extern "C" fn LISTVIEW_WMPaint(infoPtr: *mut LISTVIEW_INFO, hdc: HDC) -> LRESULT {
    if is_redrawing(infoPtr) == 0 {
        return DefWindowProcW(
            (*infoPtr).hwndSelf,
            WM_PAINT as UINT,
            hdc as WPARAM,
            0 as LPARAM,
        );
    }

    return LISTVIEW_Paint(infoPtr, hdc);
}

unsafe extern "C" fn LISTVIEW_PrintClient(
    infoPtr: *mut LISTVIEW_INFO,
    hdc: HDC,
    options: DWORD,
) -> LRESULT {
    if (options & PRF_CHECKVISIBLE as DWORD) != 0 && IsWindowVisible((*infoPtr).hwndSelf) == 0 {
        return 0 as LRESULT;
    }
    if (options & !((PRF_ERASEBKGND | PRF_CLIENT) as DWORD)) != 0 {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int)
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/listview.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"LISTVIEW_PrintClient\0" as *const u8 as *const ::core::ffi::c_char,
                10946 as ::core::ffi::c_int,
                b"(hdc=%p options=0x%08x) partial stub\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                hdc,
                options,
            );
        }
    }
    if (options & PRF_ERASEBKGND as DWORD) != 0 {
        LISTVIEW_EraseBkgnd(infoPtr, hdc);
    }
    if (options & PRF_CLIENT as DWORD) != 0 {
        LISTVIEW_Paint(infoPtr, hdc);
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn LISTVIEW_RButtonDblClk(
    infoPtr: *const LISTVIEW_INFO,
    wKey: WORD,
    x: INT,
    y: INT,
) -> LRESULT {
    let mut lvHitTestInfo: LVHITTESTINFO = core::mem::zeroed();

    /* send NM_RELEASEDCAPTURE notification */
    if notify(infoPtr, (NM_RELEASEDCAPTURE) as INT) == 0 {
        return 0;
    }

    /* send NM_RDBLCLK notification */
    lvHitTestInfo.pt.x = x;
    lvHitTestInfo.pt.y = y;
    LISTVIEW_HitTest(
        infoPtr,
        core::ptr::addr_of_mut!(lvHitTestInfo),
        TRUE,
        FALSE,
    );
    notify_click(
        infoPtr,
        (NM_RDBLCLK) as INT,
        core::ptr::addr_of_mut!(lvHitTestInfo),
    );

    return 0;
}

unsafe extern "C" fn LISTVIEW_RButtonDown(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut wKey: WORD,
    mut x: INT,
    mut y: INT,
) -> LRESULT {
    let mut ht: LVHITTESTINFO = core::mem::zeroed();
    let mut item: INT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_RButtonDown\0" as *const u8 as *const ::core::ffi::c_char,
            11005 as ::core::ffi::c_int,
            b"(key=%hu, x=%d, y=%d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            wKey as ::core::ffi::c_int,
            x,
            y,
        );
    }

    /* send NM_RELEASEDCAPTURE notification */
    if notify(infoPtr, NM_RELEASEDCAPTURE as INT) == 0 {
        return 0 as LRESULT;
    }

    /* determine the index of the selected item */
    ht.pt.x = x as LONG;
    ht.pt.y = y as LONG;
    item = LISTVIEW_HitTest(infoPtr, core::ptr::addr_of_mut!(ht), TRUE as BOOL, TRUE as BOOL);

    /* make sure the listview control window has the focus */
    if (*infoPtr).bFocus == 0 {
        SetFocus((*infoPtr).hwndSelf);
    }

    if item >= 0 as INT && item < (*infoPtr).nItemCount {
        LISTVIEW_SetItemFocus(infoPtr, item);
        if !(((wKey as INT) & (MK_SHIFT as INT)) != 0
            || ((wKey as INT) & (MK_CONTROL as INT)) != 0)
            && LISTVIEW_GetItemState(infoPtr, item, LVIS_SELECTED as UINT) == 0
        {
            LISTVIEW_SetSelection(infoPtr, item);
        }
    } else {
        LISTVIEW_DeselectAll(infoPtr);
    }

    if LISTVIEW_TrackMouse(infoPtr, ht.pt) != 0 {
        if ht.iItem != -(1 as INT) {
            let mut nmlv: NMLISTVIEW = core::mem::zeroed();

            memset(
                core::ptr::addr_of_mut!(nmlv) as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                core::mem::size_of::<NMLISTVIEW>() as size_t,
            );
            nmlv.iItem = ht.iItem;
            nmlv.ptAction = ht.pt;

            notify_listview(infoPtr, LVN_BEGINRDRAG as INT, core::ptr::addr_of_mut!(nmlv));
        }
    } else {
        SetFocus((*infoPtr).hwndSelf);

        ht.pt.x = x as LONG;
        ht.pt.y = y as LONG;
        LISTVIEW_HitTest(infoPtr, core::ptr::addr_of_mut!(ht), TRUE as BOOL, FALSE as BOOL);

        if notify_click(infoPtr, NM_RCLICK as INT, core::ptr::addr_of_mut!(ht)) != 0 {
            /* Send a WM_CONTEXTMENU message in response to the WM_RBUTTONUP */
            SendMessageW(
                (*infoPtr).hwndSelf,
                WM_CONTEXTMENU as UINT,
                (*infoPtr).hwndSelf as WPARAM,
                GetMessagePos() as LPARAM,
            );
        }
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn LISTVIEW_SetCursor(infoPtr: *const LISTVIEW_INFO, wParam: WPARAM, lParam: LPARAM) -> BOOL {
    let mut lvHitTestInfo: LVHITTESTINFO = core::mem::zeroed();

    'forward: {
        if LISTVIEW_IsHotTracking(infoPtr) == 0 {
            break 'forward;
        }

        if (*infoPtr).hHotCursor.is_null() {
            break 'forward;
        }

        GetCursorPos(core::ptr::addr_of_mut!(lvHitTestInfo.pt));
        if LISTVIEW_HitTest(infoPtr, core::ptr::addr_of_mut!(lvHitTestInfo), FALSE, FALSE) < 0 {
            break 'forward;
        }

        SetCursor((*infoPtr).hHotCursor);

        return TRUE;
    }

    return DefWindowProcW((*infoPtr).hwndSelf, WM_SETCURSOR, wParam, lParam) as BOOL;
}

unsafe extern "C" fn LISTVIEW_SetFocus(infoPtr: *mut LISTVIEW_INFO, hwndLoseFocus: HWND) -> LRESULT {
    /* if we have the focus already, there's nothing to do */
    if (*infoPtr).bFocus != 0 {
        return 0;
    }

    /* send NM_SETFOCUS notification */
    if notify(infoPtr, NM_SETFOCUS as INT) == 0 {
        return 0;
    }

    /* set window focus flag */
    (*infoPtr).bFocus = TRUE;

    /* put the focus rect back on */
    LISTVIEW_ShowFocusRect(infoPtr, TRUE);

    /* redraw all visible selected items */
    LISTVIEW_InvalidateSelectedItems(infoPtr);

    return 0;
}

unsafe extern "C" fn LISTVIEW_SetFont(infoPtr: *mut LISTVIEW_INFO, hFont: HFONT, fRedraw: WORD) -> LRESULT {
    let oldFont: HFONT = (*infoPtr).hFont;
    let oldHeight: INT = (*infoPtr).nItemHeight;

    (*infoPtr).hFont = if !hFont.is_null() {
        hFont
    } else {
        (*infoPtr).hDefaultFont
    };
    if (*infoPtr).hFont == oldFont {
        return 0;
    }
    LISTVIEW_SaveTextMetrics(infoPtr);

    (*infoPtr).nItemHeight = LISTVIEW_CalculateItemHeight(infoPtr);

    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD {
        SendMessageW(
            (*infoPtr).hwndHeader,
            WM_SETFONT as UINT,
            hFont as WPARAM,
            MAKELPARAM!(fRedraw, 0) as LPARAM,
        );
        LISTVIEW_UpdateSize(infoPtr);
        LISTVIEW_UpdateScroll(infoPtr);
    } else if (*infoPtr).nItemHeight != oldHeight {
        LISTVIEW_UpdateScroll(infoPtr);
    }

    if fRedraw != 0 {
        LISTVIEW_InvalidateList(infoPtr);
    }

    return 0;
}

unsafe extern "C" fn LISTVIEW_SetRedraw(infoPtr: *mut LISTVIEW_INFO, redraw: BOOL) -> LRESULT {
    if (*infoPtr).redraw == (redraw != 0) as BOOL {
        return 0;
    }
    (*infoPtr).redraw = (redraw != 0) as BOOL;
    if (*infoPtr).redraw == 0 {
        return 0;
    }
    if is_autoarrange(infoPtr) != 0 {
        LISTVIEW_Arrange(infoPtr, (LVA_DEFAULT) as INT);
    }
    LISTVIEW_UpdateScroll(infoPtr);
    LISTVIEW_InvalidateList(infoPtr);
    return 0;
}

unsafe extern "C" fn LISTVIEW_Size(infoPtr: *mut LISTVIEW_INFO, Width: INT, Height: INT) -> LRESULT {
    let mut rcOld: RECT = (*infoPtr).rcList;

    LISTVIEW_UpdateSize(infoPtr);
    if EqualRect(
        core::ptr::addr_of_mut!(rcOld) as LPCRECT,
        core::ptr::addr_of_mut!((*infoPtr).rcList) as LPCRECT,
    ) != 0
    {
        return 0 as LRESULT;
    }

    /* do not bother with display related stuff if we're not redrawing */
    if is_redrawing(infoPtr) == 0 {
        return 0 as LRESULT;
    }

    if is_autoarrange(infoPtr) != 0 {
        LISTVIEW_Arrange(infoPtr, LVA_DEFAULT as INT);
    }

    LISTVIEW_UpdateScroll(infoPtr);

    /* refresh all only for lists whose height changed significantly */
    if (*infoPtr).uView == LV_VIEW_LIST as DWORD
        && (rcOld.bottom as INT - rcOld.top as INT) / (*infoPtr).nItemHeight
            != ((*infoPtr).rcList.bottom as INT - (*infoPtr).rcList.top as INT)
                / (*infoPtr).nItemHeight
    {
        LISTVIEW_InvalidateList(infoPtr);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn LISTVIEW_UpdateSize(infoPtr: *mut LISTVIEW_INFO) {
    GetClientRect((*infoPtr).hwndSelf, &raw mut (*infoPtr).rcList);

    if (*infoPtr).uView == LV_VIEW_LIST as DWORD {
        /* Apparently the "LIST" style is supposed to have the same
         * number of items in a column even if there is no scroll bar.
         * Since if a scroll bar already exists then the bottom is already
         * reduced, only reduce if the scroll bar does not currently exist.
         * The "2" is there to mimic the native control. I think it may be
         * related to either padding or edges.  (GLA 7/2002)
         */
        if GetWindowLongW((*infoPtr).hwndSelf, GWL_STYLE) as ::core::ffi::c_int
            & WS_HSCROLL as ::core::ffi::c_int
            == 0
        {
            (*infoPtr).rcList.bottom -= GetSystemMetrics((SM_CYHSCROLL) as i32);
        }
        (*infoPtr).rcList.bottom = max(
            (*infoPtr).rcList.bottom as ::core::ffi::c_int - 2,
            0,
        ) as LONG;
    }

    /* When ListView control is created invisible, header isn't created right away. */
    if !(*infoPtr).hwndHeader.is_null() {
        let mut origin: POINT = POINT { x: 0, y: 0 };
        let mut wp: WINDOWPOS = WINDOWPOS {
            hwnd: core::ptr::null_mut(),
            hwndInsertAfter: core::ptr::null_mut(),
            x: 0,
            y: 0,
            cx: 0,
            cy: 0,
            flags: 0,
        };
        let mut hl: HDLAYOUT = HDLAYOUT {
            prc: core::ptr::null_mut(),
            pwpos: core::ptr::null_mut(),
        };
        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        LISTVIEW_GetOrigin(infoPtr, &raw mut origin);

        rect = (*infoPtr).rcList;
        rect.left += origin.x;

        hl.prc = &raw mut rect;
        hl.pwpos = &raw mut wp;
        SendMessageW(
            (*infoPtr).hwndHeader,
            HDM_LAYOUT as UINT,
            0 as WPARAM,
            &raw mut hl as LPARAM,
        );

        if LISTVIEW_IsHeaderEnabled(infoPtr) != 0 {
            wp.flags |= SWP_SHOWWINDOW as UINT;
        } else {
            wp.flags |= SWP_HIDEWINDOW as UINT;
            wp.cy = 0 as ::core::ffi::c_int;
        }

        SetWindowPos(
            wp.hwnd,
            wp.hwndInsertAfter,
            wp.x,
            wp.y,
            wp.cx,
            wp.cy,
            wp.flags,
        );

        (*infoPtr).rcList.top = max(wp.cy, 0) as LONG;
    }

    /* extra padding for grid */
    if (*infoPtr).uView == LV_VIEW_DETAILS as DWORD
        && (*infoPtr).dwLvExStyle & LVS_EX_GRIDLINES as DWORD != 0
    {
        (*infoPtr).rcList.top += 2 as LONG;
    }
}

unsafe extern "C" fn LISTVIEW_StyleChanged(infoPtr: *mut LISTVIEW_INFO, wStyleType: WPARAM, lpss: *const STYLESTRUCT) -> INT {
    let mut uNewView: UINT = 0;
    let mut uOldView: UINT = 0;
    let mut style: UINT = 0;

    if wStyleType != ((GWL_STYLE) as u32) as _ || (*lpss).styleNew == (*infoPtr).dwStyle {
        return 0;
    }

    (*infoPtr).dwStyle = (*lpss).styleNew;

    if ((*lpss).styleOld & WS_HSCROLL) != 0
        && ((*lpss).styleNew & WS_HSCROLL) == 0
    {
        ShowScrollBar((*infoPtr).hwndSelf, (SB_HORZ) as i32, FALSE);
    }

    if ((*lpss).styleOld & WS_VSCROLL) != 0
        && ((*lpss).styleNew & WS_VSCROLL) == 0
    {
        ShowScrollBar((*infoPtr).hwndSelf, (SB_VERT) as i32, FALSE);
    }

    uNewView = (*lpss).styleNew & LVS_TYPEMASK;
    uOldView = (*lpss).styleOld & LVS_TYPEMASK;

    if uNewView != uOldView {
        let mut himl: HIMAGELIST = core::ptr::null_mut();

        map_style_view(infoPtr);
        SendMessageW((*infoPtr).hwndEdit, WM_KILLFOCUS, 0, 0);
        ShowWindow((*infoPtr).hwndHeader, (SW_HIDE) as i32);

        ShowScrollBar((*infoPtr).hwndSelf, (SB_BOTH) as i32, FALSE);
        SetRectEmpty(core::ptr::addr_of_mut!((*infoPtr).rcFocus));

        himl = if uNewView == LVS_ICON {
            (*infoPtr).himlNormal
        } else {
            (*infoPtr).himlSmall
        };
        set_icon_size(
            core::ptr::addr_of_mut!((*infoPtr).iconSize),
            himl,
            (uNewView != LVS_ICON) as i32,
        );

        if uNewView == LVS_REPORT {
            let mut hl: HDLAYOUT = core::mem::zeroed();
            let mut wp: WINDOWPOS = core::mem::zeroed();

            LISTVIEW_CreateHeader(infoPtr);

            hl.prc = core::ptr::addr_of_mut!((*infoPtr).rcList);
            hl.pwpos = core::ptr::addr_of_mut!(wp);
            SendMessageW(
                (*infoPtr).hwndHeader,
                HDM_LAYOUT,
                0,
                core::ptr::addr_of_mut!(hl) as LPARAM,
            );
            SetWindowPos(
                (*infoPtr).hwndHeader,
                (*infoPtr).hwndSelf,
                wp.x,
                wp.y,
                wp.cx,
                wp.cy,
                wp.flags
                    | (if ((*infoPtr).dwStyle & LVS_NOCOLUMNHEADER) != 0 {
                        SWP_HIDEWINDOW
                    } else {
                        SWP_SHOWWINDOW
                    }),
            );
        }

        LISTVIEW_UpdateItemSize(infoPtr);
    }

    if uNewView == LVS_REPORT || ((*infoPtr).dwLvExStyle & LVS_EX_HEADERINALLVIEWS) != 0 {
        if (((*lpss).styleOld ^ (*lpss).styleNew) & LVS_NOCOLUMNHEADER) != 0 {
            if ((*lpss).styleNew & LVS_NOCOLUMNHEADER) != 0 {
                style = GetWindowLongW((*infoPtr).hwndHeader, GWL_STYLE) as UINT;
                SetWindowLongW(
                    (*infoPtr).hwndHeader,
                    GWL_STYLE,
                    (style | HDS_HIDDEN) as LONG,
                );
            } else {
                style = GetWindowLongW((*infoPtr).hwndHeader, GWL_STYLE) as UINT;
                if (style & HDS_HIDDEN) != 0 {
                    SetWindowLongW(
                        (*infoPtr).hwndHeader,
                        GWL_STYLE,
                        ((style & !HDS_HIDDEN) | WS_VISIBLE) as LONG,
                    );
                }
            }
        }
    }

    if (uNewView == LVS_ICON || uNewView == LVS_SMALLICON)
        && (uNewView != uOldView || (((*lpss).styleNew ^ (*lpss).styleOld) & LVS_ALIGNMASK) != 0)
    {
        LISTVIEW_Arrange(infoPtr, (LVA_DEFAULT) as INT);
    }

    /* update the size of the client area */
    LISTVIEW_UpdateSize(infoPtr);

    /* add scrollbars if needed */
    LISTVIEW_UpdateScroll(infoPtr);

    /* invalidate client area + erase background */
    LISTVIEW_InvalidateList(infoPtr);

    return 0;
}

unsafe extern "C" fn LISTVIEW_StyleChanging(wStyleType: WPARAM, lpss: *mut STYLESTRUCT) -> INT {
    /* don't forward LVS_OWNERDATA only if not already set to */
    if (((*lpss).styleNew ^ (*lpss).styleOld) & LVS_OWNERDATA) != 0 {
        if ((*lpss).styleOld & LVS_OWNERDATA) != 0 {
            (*lpss).styleNew |= LVS_OWNERDATA;
        } else {
            (*lpss).styleNew &= !LVS_OWNERDATA;
        }
    }

    return 0;
}

unsafe extern "C" fn LISTVIEW_ShowWindow(mut infoPtr: *mut LISTVIEW_INFO, mut bShown: WPARAM, mut iStatus: LPARAM) -> LRESULT {
    /* header delayed creation */
    if ((*infoPtr).uView == LV_VIEW_DETAILS) && bShown != 0 {
        LISTVIEW_CreateHeader(infoPtr);

        if !((LVS_NOCOLUMNHEADER & (*infoPtr).dwStyle) != 0) {
            ShowWindow((*infoPtr).hwndHeader, (SW_SHOWNORMAL) as i32);
        }
    }

    return DefWindowProcW((*infoPtr).hwndSelf, WM_SHOWWINDOW, bShown, iStatus);
}

unsafe extern "C" fn LISTVIEW_GetVersion(infoPtr: *const LISTVIEW_INFO) -> LRESULT {
    ((*infoPtr).iVersion) as _
}

unsafe extern "C" fn LISTVIEW_SetVersion(mut infoPtr: *mut LISTVIEW_INFO, mut iVersion: DWORD) -> LRESULT {
    let mut iOldVersion: INT = (*infoPtr).iVersion as INT;
    if iVersion > COMCTL32_VERSION as DWORD {
        return -(1 as LRESULT);
    }
    (*infoPtr).iVersion = iVersion;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_SetVersion\0" as *const u8 as *const ::core::ffi::c_char,
            11505 as ::core::ffi::c_int,
            b"new version %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            iVersion,
        );
    }
    return iOldVersion as LRESULT;
}

unsafe extern "system" fn LISTVIEW_WindowProc(
    mut hwnd: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: *mut LISTVIEW_INFO =
        GetWindowLongPtrW(hwnd, 0) as *mut LISTVIEW_INFO;
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/listview.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"LISTVIEW_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            11520 as ::core::ffi::c_int,
            b"(hwnd=%p uMsg=%x wParam=%lx lParam=%lx)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hwnd,
            uMsg,
            wParam,
            lParam,
        );
    }

    if infoPtr.is_null() && (uMsg != WM_NCCREATE) {
        return DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }

    match uMsg {
        LVM_APPROXIMATEVIEWRECT => {
            return LISTVIEW_ApproximateViewRect(
                infoPtr,
                wParam as INT,
                LOWORD!(lParam),
                HIWORD!(lParam),
            ) as LRESULT;
        }
        LVM_ARRANGE => {
            return LISTVIEW_Arrange(infoPtr, wParam as INT) as LRESULT;
        }
        LVM_CANCELEDITLABEL => {
            return LISTVIEW_CancelEditLabel(infoPtr) as LRESULT;
        }
        LVM_CREATEDRAGIMAGE => {
            return LISTVIEW_CreateDragImage(infoPtr, wParam as INT, lParam as LPPOINT) as LRESULT;
        }
        LVM_DELETEALLITEMS => {
            return LISTVIEW_DeleteAllItems(infoPtr, FALSE) as LRESULT;
        }
        LVM_DELETECOLUMN => {
            return LISTVIEW_DeleteColumn(infoPtr, wParam as INT) as LRESULT;
        }
        LVM_DELETEITEM => {
            return LISTVIEW_DeleteItem(infoPtr, wParam as INT) as LRESULT;
        }
        LVM_EDITLABELA | LVM_EDITLABELW => {
            return LISTVIEW_EditLabelT(
                infoPtr,
                wParam as INT,
                (uMsg == LVM_EDITLABELW) as BOOL,
            ) as LRESULT;
        }
        LVM_ENSUREVISIBLE => {
            return LISTVIEW_EnsureVisible(infoPtr, wParam as INT, lParam as BOOL) as LRESULT;
        }
        LVM_FINDITEMW => {
            return LISTVIEW_FindItemW(infoPtr, wParam as INT, lParam as LPLVFINDINFOW)
                as LRESULT;
        }
        LVM_FINDITEMA => {
            return LISTVIEW_FindItemA(infoPtr, wParam as INT, lParam as LPLVFINDINFOA)
                as LRESULT;
        }
        LVM_GETBKCOLOR => {
            return (*infoPtr).clrBk as LRESULT;
        }
        LVM_GETCALLBACKMASK => {
            return (*infoPtr).uCallbackMask as LRESULT;
        }
        LVM_GETCOLUMNA | LVM_GETCOLUMNW => {
            return LISTVIEW_GetColumnT(
                infoPtr,
                wParam as INT,
                lParam as LPLVCOLUMNW,
                (uMsg == LVM_GETCOLUMNW) as BOOL,
            ) as LRESULT;
        }
        LVM_GETCOLUMNORDERARRAY => {
            return LISTVIEW_GetColumnOrderArray(infoPtr, wParam as INT, lParam as LPINT) as LRESULT;
        }
        LVM_GETCOLUMNWIDTH => {
            return LISTVIEW_GetColumnWidth(infoPtr, wParam as INT) as LRESULT;
        }
        LVM_GETCOUNTPERPAGE => {
            return LISTVIEW_GetCountPerPage(infoPtr) as LRESULT;
        }
        LVM_GETEDITCONTROL => {
            return (*infoPtr).hwndEdit as LRESULT;
        }
        LVM_GETEXTENDEDLISTVIEWSTYLE => {
            return (*infoPtr).dwLvExStyle as LRESULT;
        }
        LVM_GETHEADER => {
            return (*infoPtr).hwndHeader as LRESULT;
        }
        LVM_GETHOTCURSOR => {
            return (*infoPtr).hHotCursor as LRESULT;
        }
        LVM_GETHOTITEM => {
            return (*infoPtr).nHotItem as LRESULT;
        }
        LVM_GETHOVERTIME => {
            return (*infoPtr).dwHoverTime as LRESULT;
        }
        LVM_GETIMAGELIST => {
            return LISTVIEW_GetImageList(infoPtr, wParam as INT) as LRESULT;
        }
        LVM_GETISEARCHSTRINGA | LVM_GETISEARCHSTRINGW => {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int))
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/listview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"LISTVIEW_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                    11618 as ::core::ffi::c_int,
                    b"LVM_GETISEARCHSTRING: unimplemented\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            return FALSE as LRESULT;
        }
        LVM_GETITEMA | LVM_GETITEMW => {
            return LISTVIEW_GetItemExtT(
                infoPtr,
                lParam as LPLVITEMW,
                (uMsg == LVM_GETITEMW) as BOOL,
            ) as LRESULT;
        }
        LVM_GETITEMCOUNT => {
            return (*infoPtr).nItemCount as LRESULT;
        }
        LVM_GETITEMPOSITION => {
            return LISTVIEW_GetItemPosition(infoPtr, wParam as INT, lParam as LPPOINT) as LRESULT;
        }
        LVM_GETITEMRECT => {
            return LISTVIEW_GetItemRect(infoPtr, wParam as INT, lParam as LPRECT) as LRESULT;
        }
        LVM_GETITEMSPACING => {
            return LISTVIEW_GetItemSpacing(infoPtr, wParam as BOOL) as LRESULT;
        }
        LVM_GETITEMSTATE => {
            return LISTVIEW_GetItemState(infoPtr, wParam as INT, lParam as UINT) as LRESULT;
        }
        LVM_GETITEMTEXTA | LVM_GETITEMTEXTW => {
            return LISTVIEW_GetItemTextT(
                infoPtr,
                wParam as INT,
                lParam as LPLVITEMW,
                (uMsg == LVM_GETITEMTEXTW) as BOOL,
            ) as LRESULT;
        }
        LVM_GETNEXTITEM => {
            return LISTVIEW_GetNextItem(infoPtr, wParam as INT, (LOWORD!(lParam)) as UINT) as LRESULT;
        }
        LVM_GETNUMBEROFWORKAREAS => {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int))
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/listview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"LISTVIEW_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                    11649 as ::core::ffi::c_int,
                    b"LVM_GETNUMBEROFWORKAREAS: unimplemented\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            return 1 as LRESULT;
        }
        LVM_GETORIGIN => {
            if lParam == 0 {
                return FALSE as LRESULT;
            }
            if (*infoPtr).uView == LV_VIEW_DETAILS
                || (*infoPtr).uView == LV_VIEW_LIST
            {
                return FALSE as LRESULT;
            }
            LISTVIEW_GetOrigin(infoPtr, lParam as LPPOINT);
            return TRUE as LRESULT;
        }
        LVM_GETSELECTEDCOUNT => {
            return LISTVIEW_GetSelectedCount(infoPtr) as LRESULT;
        }
        LVM_GETSELECTIONMARK => {
            return (*infoPtr).nSelectionMark as LRESULT;
        }
        LVM_GETSTRINGWIDTHA | LVM_GETSTRINGWIDTHW => {
            return LISTVIEW_GetStringWidthT(
                infoPtr,
                lParam as LPCWSTR,
                (uMsg == LVM_GETSTRINGWIDTHW) as BOOL,
            ) as LRESULT;
        }
        LVM_GETSUBITEMRECT => {
            return LISTVIEW_GetSubItemRect(infoPtr, (wParam as UINT) as INT, lParam as LPRECT) as LRESULT;
        }
        LVM_GETTEXTBKCOLOR => {
            return (*infoPtr).clrTextBk as LRESULT;
        }
        LVM_GETTEXTCOLOR => {
            return (*infoPtr).clrText as LRESULT;
        }
        LVM_GETTOOLTIPS => {
            if (*infoPtr).hwndToolTip.is_null() {
                (*infoPtr).hwndToolTip = COMCTL32_CreateToolTip(hwnd);
            }
            return (*infoPtr).hwndToolTip as LRESULT;
        }
        LVM_GETTOPINDEX => {
            return LISTVIEW_GetTopIndex(infoPtr) as LRESULT;
        }
        LVM_GETUNICODEFORMAT => {
            return ((((*infoPtr).notifyFormat as INT)) as u32 == NFR_UNICODE) as LRESULT;
        }
        LVM_GETVIEW => {
            return (*infoPtr).uView as LRESULT;
        }
        LVM_GETVIEWRECT => {
            return LISTVIEW_GetViewRect(infoPtr, lParam as LPRECT) as LRESULT;
        }
        LVM_GETWORKAREAS => {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int))
                != 0
            {
                let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_2: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_2,
                    __dbch_2,
                    b"dll/win32/comctl32/listview.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"LISTVIEW_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                    11705 as ::core::ffi::c_int,
                    b"LVM_GETWORKAREAS: unimplemented\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            return FALSE as LRESULT;
        }
        LVM_HITTEST => {
            return LISTVIEW_HitTest(
                infoPtr,
                lParam as LPLVHITTESTINFO,
                FALSE,
                TRUE,
            ) as LRESULT;
        }
        LVM_INSERTCOLUMNA | LVM_INSERTCOLUMNW => {
            return LISTVIEW_InsertColumnT(
                infoPtr,
                wParam as INT,
                lParam as LPLVCOLUMNW,
                (uMsg == LVM_INSERTCOLUMNW) as BOOL,
            ) as LRESULT;
        }
        LVM_INSERTITEMA | LVM_INSERTITEMW => {
            return LISTVIEW_InsertItemT(
                infoPtr,
                lParam as LPLVITEMW,
                (uMsg == LVM_INSERTITEMW) as BOOL,
            ) as LRESULT;
        }
        LVM_ISITEMVISIBLE => {
            return LISTVIEW_IsItemVisible(infoPtr, wParam as INT) as LRESULT;
        }
        LVM_MAPIDTOINDEX => {
            return LISTVIEW_MapIdToIndex(infoPtr, wParam as UINT) as LRESULT;
        }
        LVM_MAPINDEXTOID => {
            return LISTVIEW_MapIndexToId(infoPtr, wParam as INT) as LRESULT;
        }
        LVM_REDRAWITEMS => {
            return LISTVIEW_RedrawItems(infoPtr, wParam as INT, lParam as INT) as LRESULT;
        }
        LVM_SCROLL => {
            return LISTVIEW_Scroll(infoPtr, wParam as INT, lParam as INT) as LRESULT;
        }
        LVM_SETBKCOLOR => {
            return LISTVIEW_SetBkColor(infoPtr, lParam as COLORREF) as LRESULT;
        }
        LVM_SETCALLBACKMASK => {
            (*infoPtr).uCallbackMask = wParam as UINT;
            return TRUE as LRESULT;
        }
        LVM_SETCOLUMNA | LVM_SETCOLUMNW => {
            return LISTVIEW_SetColumnT(
                infoPtr,
                wParam as INT,
                lParam as LPLVCOLUMNW,
                (uMsg == LVM_SETCOLUMNW) as BOOL,
            ) as LRESULT;
        }
        LVM_SETCOLUMNORDERARRAY => {
            return LISTVIEW_SetColumnOrderArray(infoPtr, wParam as INT, lParam as LPINT) as LRESULT;
        }
        LVM_SETCOLUMNWIDTH => {
            return LISTVIEW_SetColumnWidth(infoPtr, wParam as INT, (LOWORD!(lParam) as SHORT) as INT)
                as LRESULT;
        }
        LVM_SETEXTENDEDLISTVIEWSTYLE => {
            return LISTVIEW_SetExtendedListViewStyle(infoPtr, wParam as DWORD, lParam as DWORD) as LRESULT;
        }
        LVM_SETHOTCURSOR => {
            return LISTVIEW_SetHotCursor(infoPtr, lParam as HCURSOR) as LRESULT;
        }
        LVM_SETHOTITEM => {
            return LISTVIEW_SetHotItem(infoPtr, wParam as INT) as LRESULT;
        }
        LVM_SETHOVERTIME => {
            return LISTVIEW_SetHoverTime(infoPtr, lParam as DWORD) as LRESULT;
        }
        LVM_SETICONSPACING => {
            if lParam == -1 {
                return LISTVIEW_SetIconSpacing(
                    infoPtr,
                    -1,
                    -1,
                ) as LRESULT;
            }
            return LISTVIEW_SetIconSpacing(infoPtr, (LOWORD!(lParam)) as INT, (HIWORD!(lParam)) as INT) as LRESULT;
        }
        LVM_SETIMAGELIST => {
            return LISTVIEW_SetImageList(infoPtr, wParam as INT, lParam as HIMAGELIST)
                as LRESULT;
        }
        LVM_SETITEMA | LVM_SETITEMW => {
            {
                if ((*infoPtr).dwStyle & LVS_OWNERDATA) != 0 {
                    return FALSE as LRESULT;
                }
                return LISTVIEW_SetItemT(
                    infoPtr,
                    lParam as LPLVITEMW,
                    (uMsg == LVM_SETITEMW) as BOOL,
                ) as LRESULT;
            }
        }
        LVM_SETITEMCOUNT => {
            return LISTVIEW_SetItemCount(infoPtr, wParam as INT, lParam as DWORD) as LRESULT;
        }
        LVM_SETITEMPOSITION => {
            {
                let mut pt: POINT = core::mem::zeroed();
                pt.x = LOWORD!(lParam) as SHORT as LONG;
                pt.y = HIWORD!(lParam) as SHORT as LONG;
                return LISTVIEW_SetItemPosition(
                    infoPtr,
                    wParam as INT,
                    core::ptr::addr_of_mut!(pt),
                ) as LRESULT;
            }
        }
        LVM_SETITEMPOSITION32 => {
            return LISTVIEW_SetItemPosition(infoPtr, wParam as INT, lParam as *mut POINT)
                as LRESULT;
        }
        LVM_SETITEMSTATE => {
            return LISTVIEW_SetItemState(infoPtr, wParam as INT, lParam as LPLVITEMW)
                as LRESULT;
        }
        LVM_SETITEMTEXTA | LVM_SETITEMTEXTW => {
            return LISTVIEW_SetItemTextT(
                infoPtr,
                wParam as INT,
                lParam as LPLVITEMW,
                (uMsg == LVM_SETITEMTEXTW) as BOOL,
            ) as LRESULT;
        }
        LVM_SETSELECTIONMARK => {
            return LISTVIEW_SetSelectionMark(infoPtr, lParam as INT) as LRESULT;
        }
        LVM_SETTEXTBKCOLOR => {
            return LISTVIEW_SetTextBkColor(infoPtr, lParam as COLORREF) as LRESULT;
        }
        LVM_SETTEXTCOLOR => {
            return LISTVIEW_SetTextColor(infoPtr, lParam as COLORREF) as LRESULT;
        }
        LVM_SETTOOLTIPS => {
            return LISTVIEW_SetToolTips(infoPtr, lParam as HWND) as LRESULT;
        }
        LVM_SETUNICODEFORMAT => {
            return LISTVIEW_SetUnicodeFormat(infoPtr, (wParam) as BOOL) as LRESULT;
        }
        LVM_SETVIEW => {
            return LISTVIEW_SetView(infoPtr, (wParam) as DWORD) as LRESULT;
        }
        LVM_SORTITEMS | LVM_SORTITEMSEX => {
            return LISTVIEW_SortItems(
                infoPtr,
                core::ptr::read(core::ptr::addr_of!(lParam) as *const PFNLVCOMPARE),
                (wParam) as LPARAM,
                (uMsg == LVM_SORTITEMSEX) as BOOL,
            ) as LRESULT;
        }
        LVM_SUBITEMHITTEST => {
            return LISTVIEW_HitTest(
                infoPtr,
                lParam as LPLVHITTESTINFO,
                TRUE,
                FALSE,
            ) as LRESULT;
        }
        LVM_UPDATE => {
            return LISTVIEW_Update(infoPtr, wParam as INT) as LRESULT;
        }
        CCM_GETVERSION => {
            return LISTVIEW_GetVersion(infoPtr) as LRESULT;
        }
        CCM_SETVERSION => {
            return LISTVIEW_SetVersion(infoPtr, (wParam) as DWORD) as LRESULT;
        }
        WM_CHAR => {
            return LISTVIEW_ProcessLetterKeys(infoPtr, wParam, lParam) as LRESULT;
        }
        WM_COMMAND => {
            return LISTVIEW_Command(infoPtr, wParam, lParam) as LRESULT;
        }
        WM_NCCREATE => {
            return LISTVIEW_NCCreate(hwnd, wParam, lParam as LPCREATESTRUCTW) as LRESULT;
        }
        WM_CREATE => {
            return LISTVIEW_Create(hwnd, lParam as LPCREATESTRUCTW) as LRESULT;
        }
        WM_DESTROY => {
            return LISTVIEW_Destroy(infoPtr) as LRESULT;
        }
        WM_ENABLE => {
            return LISTVIEW_Enable(infoPtr) as LRESULT;
        }
        WM_ERASEBKGND => {
            return LISTVIEW_EraseBkgnd(infoPtr, wParam as HDC) as LRESULT;
        }
        WM_GETDLGCODE => {
            return (DLGC_WANTCHARS | DLGC_WANTARROWS) as LRESULT;
        }
        WM_GETFONT => {
            return (*infoPtr).hFont as LRESULT;
        }
        WM_HSCROLL => {
            return LISTVIEW_HScroll(
                infoPtr,
                LOWORD!(wParam) as INT,
                0,
            ) as LRESULT;
        }
        WM_KEYDOWN => {
            return LISTVIEW_KeyDown(infoPtr, wParam as INT, lParam as LONG) as LRESULT;
        }
        WM_KILLFOCUS => {
            return LISTVIEW_KillFocus(infoPtr) as LRESULT;
        }
        WM_LBUTTONDBLCLK => {
            return LISTVIEW_LButtonDblClk(
                infoPtr,
                wParam as WORD,
                (LOWORD!(lParam) as SHORT) as INT,
                (HIWORD!(lParam) as SHORT) as INT,
            ) as LRESULT;
        }
        WM_LBUTTONDOWN => {
            return LISTVIEW_LButtonDown(
                infoPtr,
                wParam as WORD,
                (LOWORD!(lParam) as SHORT) as INT,
                (HIWORD!(lParam) as SHORT) as INT,
            ) as LRESULT;
        }
        WM_LBUTTONUP => {
            return LISTVIEW_LButtonUp(
                infoPtr,
                wParam as WORD,
                (LOWORD!(lParam) as SHORT) as INT,
                (HIWORD!(lParam) as SHORT) as INT,
            ) as LRESULT;
        }
        WM_MOUSEMOVE => {
            return LISTVIEW_MouseMove(
                infoPtr,
                wParam as WORD,
                (LOWORD!(lParam) as SHORT) as INT,
                (HIWORD!(lParam) as SHORT) as INT,
            ) as LRESULT;
        }
        WM_MOUSEHOVER => {
            return LISTVIEW_MouseHover(
                infoPtr,
                (LOWORD!(lParam) as SHORT) as INT,
                (HIWORD!(lParam) as SHORT) as INT,
            ) as LRESULT;
        }
        WM_NCDESTROY => {
            return LISTVIEW_NCDestroy(infoPtr) as LRESULT;
        }
        WM_NCPAINT => {
            return LISTVIEW_NCPaint(infoPtr, wParam as HRGN) as LRESULT;
        }
        WM_NOTIFY => {
            return LISTVIEW_Notify(infoPtr, lParam as LPNMHDR) as LRESULT;
        }
        WM_NOTIFYFORMAT => {
            return LISTVIEW_NotifyFormat(infoPtr, wParam as HWND, lParam as INT) as LRESULT;
        }
        WM_PRINTCLIENT => {
            return LISTVIEW_PrintClient(infoPtr, wParam as HDC, lParam as DWORD) as LRESULT;
        }
        WM_PAINT => {
            return LISTVIEW_WMPaint(infoPtr, wParam as HDC) as LRESULT;
        }
        WM_RBUTTONDBLCLK => {
            return LISTVIEW_RButtonDblClk(
                infoPtr,
                wParam as WORD,
                (LOWORD!(lParam) as SHORT) as INT,
                (HIWORD!(lParam) as SHORT) as INT,
            ) as LRESULT;
        }
        WM_RBUTTONDOWN => {
            return LISTVIEW_RButtonDown(
                infoPtr,
                wParam as WORD,
                (LOWORD!(lParam) as SHORT) as INT,
                (HIWORD!(lParam) as SHORT) as INT,
            ) as LRESULT;
        }
        WM_SETCURSOR => {
            return LISTVIEW_SetCursor(infoPtr, wParam, lParam) as LRESULT;
        }
        WM_SETFOCUS => {
            return LISTVIEW_SetFocus(infoPtr, wParam as HWND) as LRESULT;
        }
        WM_SETFONT => {
            return LISTVIEW_SetFont(infoPtr, wParam as HFONT, lParam as WORD) as LRESULT;
        }
        WM_SETREDRAW => {
            return LISTVIEW_SetRedraw(infoPtr, wParam as BOOL) as LRESULT;
        }
        WM_SHOWWINDOW => {
            return LISTVIEW_ShowWindow(infoPtr, wParam, lParam) as LRESULT;
        }
        WM_STYLECHANGED => {
            return LISTVIEW_StyleChanged(infoPtr, wParam, lParam as LPSTYLESTRUCT) as LRESULT;
        }
        WM_STYLECHANGING => {
            return LISTVIEW_StyleChanging(wParam, lParam as LPSTYLESTRUCT) as LRESULT;
        }
        WM_SYSCOLORCHANGE => {
            COMCTL32_RefreshSysColors();
            if (*infoPtr).bDefaultBkColor != 0 {
                LISTVIEW_SetBkColor(infoPtr, comctl32_color.clrWindow);
                (*infoPtr).bDefaultBkColor = TRUE;
                LISTVIEW_InvalidateList(infoPtr);
            }
            return 0 as LRESULT;
        }
        /*	case WM_TIMER: */
        WM_THEMECHANGED => {
            return LISTVIEW_ThemeChanged(infoPtr) as LRESULT;
        }
        WM_VSCROLL => {
            return LISTVIEW_VScroll(
                infoPtr,
                LOWORD!(wParam) as INT,
                0,
            ) as LRESULT;
        }
        WM_MOUSEWHEEL => {
            if (wParam & ((MK_SHIFT | MK_CONTROL)) as WPARAM) != 0 {
                return DefWindowProcW(hwnd, uMsg, wParam, lParam);
            }
            return LISTVIEW_MouseWheel(infoPtr, HIWORD!(wParam) as SHORT as INT) as LRESULT;
        }
        WM_WINDOWPOSCHANGED => {
            if ((*(lParam as *mut WINDOWPOS)).flags & SWP_NOSIZE) == 0 {
                SetWindowPos(
                    (*infoPtr).hwndSelf,
                    core::ptr::null_mut(),
                    0,
                    0,
                    0,
                    0,
                    SWP_FRAMECHANGED
                        | SWP_NOACTIVATE
                        | SWP_NOZORDER
                        | SWP_NOMOVE
                        | SWP_NOSIZE,
                );

                if (((*infoPtr).dwStyle & LVS_OWNERDRAWFIXED) != 0)
                    && ((*infoPtr).uView == LV_VIEW_DETAILS)
                {
                    if notify_measureitem(infoPtr) != 0 {
                        LISTVIEW_InvalidateList(infoPtr);
                    }
                }
                LISTVIEW_Size(
                    infoPtr,
                    (*(lParam as *mut WINDOWPOS)).cx,
                    (*(lParam as *mut WINDOWPOS)).cy,
                );
            }
            return DefWindowProcW(hwnd, uMsg, wParam, lParam);
        }
        WM_SETTINGCHANGE => {
            /* Same as WM_WININICHANGE */
            if wParam == (SPI_SETICONMETRICS) as _
                || wParam == (SPI_SETICONTITLELOGFONT) as _
                || wParam == (SPI_SETNONCLIENTMETRICS) as _
                || (wParam == 0 && lParam == 0)
            {
                let mut hOldDefaultFont: HFONT = (*infoPtr).hDefaultFont;
                let mut bDefaultOrNullFont: BOOL = ((*infoPtr).hDefaultFont == (*infoPtr).hFont
                    || (*infoPtr).hFont.is_null())
                    as BOOL;
                let mut logFont: LOGFONTW = core::mem::zeroed();
                SystemParametersInfoW(
                    SPI_GETICONTITLELOGFONT,
                    0,
                    core::ptr::addr_of_mut!(logFont) as PVOID,
                    0,
                );

                if (*infoPtr).autoSpacing != 0 {
                    LISTVIEW_SetIconSpacing(infoPtr, -1, -1);
                }

                if !(*infoPtr).hDefaultFont.is_null() {
                    DeleteObject(((*infoPtr).hDefaultFont).cast());
                }
                (*infoPtr).hDefaultFont = CreateFontIndirectW(core::ptr::addr_of_mut!(logFont));

                if bDefaultOrNullFont != 0 {
                    (*infoPtr).hFont = (*infoPtr).hDefaultFont;
                    // Check if we just deleted the font the header is using
                    if !(*infoPtr).hwndHeader.is_null()
                        && hOldDefaultFont == GetWindowFont!((*infoPtr).hwndHeader)
                    {
                        SendMessageW(
                            (*infoPtr).hwndHeader,
                            WM_SETFONT,
                            (*infoPtr).hFont as WPARAM,
                            (TRUE) as LPARAM,
                        );
                    }
                }

                LISTVIEW_SaveTextMetrics(infoPtr);
                LISTVIEW_UpdateItemSize(infoPtr);
                LISTVIEW_UpdateScroll(infoPtr);
                LISTVIEW_InvalidateRect(infoPtr, core::ptr::null_mut());
            }
            return 0 as LRESULT;
        }
        _ => {
            if (uMsg >= WM_USER)
                && (uMsg < WM_APP)
                && COMCTL32_IsReflectedMessage(uMsg) == 0
            {
                if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_3: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_3,
                        __dbch_3,
                        b"dll/win32/comctl32/listview.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"LISTVIEW_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        12053 as ::core::ffi::c_int,
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
}
#[no_mangle]
pub unsafe extern "C" fn LISTVIEW_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(wndClass) as *mut ::core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>() as usize,
    );
    wndClass.style = (CS_GLOBALCLASS | CS_DBLCLKS) as UINT;
    wndClass.lpfnWndProc =
        Some(LISTVIEW_WindowProc);
    wndClass.cbClsExtra = 0 as ::core::ffi::c_int;
    wndClass.cbWndExtra = core::mem::size_of::<LISTVIEW_INFO>() as ::core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        0 as HINSTANCE,
        IDC_ARROW as usize as LPWSTR as LPCWSTR,
    );
    wndClass.hbrBackground = ((COLOR_WINDOW + 1) as usize) as HBRUSH;
    wndClass.lpszClassName = (WC_LISTVIEWW).as_ptr();
    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}

#[no_mangle]
pub unsafe extern "C" fn LISTVIEW_Unregister() {
    UnregisterClassW((WC_LISTVIEWW).as_ptr(), core::ptr::null_mut());
}

unsafe extern "C" fn LISTVIEW_Command(
    mut infoPtr: *mut LISTVIEW_INFO,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    if (*infoPtr).hwndEdit.is_null() {
        return 0;
    }
    match (HIWORD!(wParam as DWORD) as ::core::ffi::c_int) as u32 {
        EN_UPDATE => {
            /*
             * Adjust the edit window size
             */
            let mut buffer: [WCHAR; 1024] = core::mem::zeroed();
            let mut hdc: HDC = GetDC((*infoPtr).hwndEdit);
            let mut hFont: HFONT = core::ptr::null_mut();
            let mut hOldFont: HFONT = core::ptr::null_mut();
            let mut rect: RECT = core::mem::zeroed();
            let mut sz: SIZE = core::mem::zeroed();

            if (*infoPtr).hwndEdit.is_null() || hdc.is_null() {
                return 0;
            }
            GetWindowTextW(
                (*infoPtr).hwndEdit,
                core::ptr::addr_of_mut!(buffer) as LPWSTR,
                (::core::mem::size_of::<[WCHAR; 1024]>() as usize)
                    .wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
                    as ::core::ffi::c_int,
            );
            GetWindowRect((*infoPtr).hwndEdit, core::ptr::addr_of_mut!(rect));

            /* Select font to get the right dimension of the string */
            hFont = SendMessageW(
                (*infoPtr).hwndEdit,
                WM_GETFONT,
                0,
                0,
            ) as HFONT;
            if !hFont.is_null() {
                hOldFont = SelectObject(hdc, hFont as HGDIOBJ) as HFONT;
            }

            if GetTextExtentPoint32W(
                hdc,
                core::ptr::addr_of_mut!(buffer) as LPCWSTR,
                lstrlenW(core::ptr::addr_of_mut!(buffer) as LPCWSTR),
                core::ptr::addr_of_mut!(sz),
            ) != 0
            {
                let mut textMetric: TEXTMETRICW = core::mem::zeroed();

                /* Add Extra spacing for the next character */
                GetTextMetricsW(hdc, core::ptr::addr_of_mut!(textMetric));
                sz.cx += textMetric.tmMaxCharWidth * 2;

                SetWindowPos(
                    (*infoPtr).hwndEdit,
                    core::ptr::null_mut(),
                    0,
                    0,
                    sz.cx,
                    rect.bottom - rect.top,
                    (SWP_DRAWFRAME | SWP_NOMOVE | SWP_NOZORDER)
                        as UINT,
                );
            }
            if !hFont.is_null() {
                SelectObject(hdc, hOldFont as HGDIOBJ);
            }

            ReleaseDC((*infoPtr).hwndEdit, hdc);
        }
        EN_KILLFOCUS => {
            if (*infoPtr).notify_mask & NOTIFY_MASK_END_LABEL_EDIT as DWORD != 0 {
                LISTVIEW_CancelEditLabel(infoPtr);
            }
        }
        _ => {
            return SendMessageW(
                (*infoPtr).hwndNotify,
                WM_COMMAND as UINT,
                wParam,
                lParam,
            );
        }
    }

    return 0;
}
