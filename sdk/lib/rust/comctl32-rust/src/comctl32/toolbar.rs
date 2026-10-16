use crate::comctl32::commctrl::COMCTL32_DrawInsertMark;
use crate::comctl32::commctrl::COMCTL32_EnsureBitmapSize;
use crate::comctl32::commctrl::CreateMappedBitmap;
use crate::comctl32::draglist::DrawInsert;
use crate::comctl32::imagelist::ImageList_AddMasked;
use crate::comctl32::imagelist::ImageList_DrawIndirect;
use crate::comctl32::imagelist::ImageList_GetImageCount;
use crate::comctl32::imagelist::ImageList_Remove;
use crate::comctl32::imagelist::ImageList_SetIconSize;
use crate::comctl32::imagelist::ImageList_SetImageCount;
use crate::comctl32::draglist::LBItemFromPt;
use crate::comctl32::draglist::MakeDragList;
use crate::comctl32::comctl32undoc::ReAlloc;
use crate::comctl32::comctl32undoc::Str_GetPtrWtoA;
use crate::comctl32::comctl32undoc::Str_SetPtrAtoW;
use crate::comctl32::commctrl::_TrackMouseEvent;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_toolbar);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
// toolbar.c:3116 `#define MAX_RESOURCE_STRING_LENGTH 512` (an array length at its sites)
const MAX_RESOURCE_STRING_LENGTH: usize = 512;
pub const TOOLBARCLASSNAMEW: [::core::ffi::c_ushort; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_ushort; 16]>(
        *b"T\0o\0o\0l\0b\0a\0r\0W\0i\0n\0d\0o\0w\x003\x002\0\0\0",
    )
};

pub const TOOLTIPS_CLASSW: [::core::ffi::c_ushort; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_ushort; 17]>(
        *b"t\0o\0o\0l\0t\0i\0p\0s\0_\0c\0l\0a\0s\0s\x003\x002\0\0\0",
    )
};

static mut hCursorDrag: HCURSOR = ::core::ptr::null::<HICON__>() as *mut HICON__;

static mut themeClass: [WCHAR; 8] = [
    'T' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

// WINE_DEFAULT_DEBUG_CHANNEL(toolbar): debug-channel macro, no Rust declaration emitted here.

#[repr(C)]
pub struct TBUTTON_INFO {
    pub iBitmap: INT,
    pub idCommand: INT,
    pub fsState: BYTE,
    pub fsStyle: BYTE,
    pub bHot: BOOL,
    pub bDropDownPressed: BOOL,
    pub dwData: DWORD_PTR,
    pub iString: INT_PTR,
    pub nRow: INT,
    pub rect: RECT,
    pub cx: INT, // manually set size
}

#[repr(C)]
pub struct TBITMAP_INFO {
    pub nButtons: UINT,
    pub hInst: HINSTANCE,
    pub nID: UINT,
}

#[repr(C)]
pub struct IMLENTRY {
    pub himl: HIMAGELIST,
    pub id: INT,
}
pub type PIMLENTRY = *mut IMLENTRY;

#[repr(C)]
pub struct TOOLBAR_INFO {
    pub dwStructSize: DWORD, // size of TBBUTTON struct
    pub client_rect: RECT,
    pub rcBound: RECT, // bounding rectangle
    pub nButtonHeight: INT,
    pub nButtonWidth: INT,
    pub nBitmapHeight: INT,
    pub nBitmapWidth: INT,
    pub nIndent: INT,
    pub nRows: INT, // number of button rows
    pub nMaxTextRows: INT, // maximum number of text rows
    pub cxMin: INT, // minimum button width
    pub cxMax: INT, // maximum button width
    pub nNumButtons: INT, // number of buttons
    pub nNumBitmaps: INT, // number of bitmaps
    pub nNumStrings: INT, // number of strings
    pub nNumBitmapInfos: INT,
    pub nButtonDown: INT, // toolbar button being pressed or -1 if none
    pub nButtonDrag: INT, // toolbar button being dragged or -1 if none
    pub nOldHit: INT,
    pub nHotItem: INT, // index of the "hot" item
    pub szPadding: SIZE, // padding values around button
    pub szBarPadding: SIZE, // padding values around the toolbar (NOT USED BUT STORED)
    pub szSpacing: SIZE, // spacing values between buttons
    pub themeMargins: MARGINS,
    pub iTopMargin: INT, // the top margin
    pub iListGap: INT, // default gap between text and image for toolbar with list style
    pub hDefaultFont: HFONT,
    pub hFont: HFONT, // text font
    pub himlInt: HIMAGELIST, // image list created internally
    pub himlDef: *mut PIMLENTRY, // default image list array
    pub cimlDef: INT, // default image list array count
    pub himlHot: *mut PIMLENTRY, // hot image list array
    pub cimlHot: INT, // hot image list array count
    pub himlDis: *mut PIMLENTRY, // disabled image list array
    pub cimlDis: INT, // disabled image list array count
    pub hwndToolTip: HWND, // handle to tool tip control
    pub hwndNotify: HWND, // handle to the window that gets notifications
    pub hwndSelf: HWND, // my own handle
    pub bAnchor: BOOL, // anchor highlight enabled
    pub bDoRedraw: BOOL, // Redraw status
    pub bDragOutSent: BOOL, // has TBN_DRAGOUT notification been sent for this drag?
    pub bUnicode: BOOL, // Notifications are ASCII (FALSE) or Unicode (TRUE)?
    pub bCaptured: BOOL, // mouse captured?
    pub dwStyle: DWORD, // regular toolbar style
    pub dwExStyle: DWORD, // extended toolbar style
    pub dwDTFlags: DWORD, // DrawText flags

    pub clrInsertMark: COLORREF, // insert mark color
    pub clrBtnHighlight: COLORREF, // color for Flat Separator
    pub clrBtnShadow: COLORREF, // color for Flag Separator
    pub iVersion: INT,
    pub pszTooltipText: LPWSTR, // temporary store for a string > 80 characters
                                // for TTN_GETDISPINFOW notification
    pub tbim: TBINSERTMARK, // info on insertion mark
    pub buttons: *mut TBUTTON_INFO, // pointer to button array
    pub strings: *mut LPWSTR, // pointer to string array
    pub bitmaps: *mut TBITMAP_INFO,
}
pub type PTOOLBAR_INFO = *mut TOOLBAR_INFO;

#[repr(C)]
pub struct CUSTDLG_INFO {
    pub tbInfo: PTOOLBAR_INFO,
    pub tbHwnd: HWND,
}
pub type PCUSTDLG_INFO = *mut CUSTDLG_INFO;

#[repr(C)]
pub struct CUSTOMBUTTON {
    pub btn: TBBUTTON,
    pub bVirtual: BOOL,
    pub bRemovable: BOOL,
    pub text: [WCHAR; 64],
}
pub type PCUSTOMBUTTON = *mut CUSTOMBUTTON;

pub type IMAGE_LIST_TYPE = i32;
pub const IMAGE_LIST_DEFAULT: IMAGE_LIST_TYPE = 0;
pub const IMAGE_LIST_HOT: IMAGE_LIST_TYPE = 1;
pub const IMAGE_LIST_DISABLED: IMAGE_LIST_TYPE = 2;

pub const SEPARATOR_WIDTH: i32 = 8;

pub const TOP_BORDER: i32 = 2;

pub const BOTTOM_BORDER: i32 = 2;

pub const DDARROW_WIDTH: i32 = 11;

pub const ARROW_HEIGHT: i32 = 3;

pub const INSERTMARK_WIDTH: i32 = 2;

pub const DEFPAD_CX: i32 = 7;

pub const DEFPAD_CY: i32 = 6;

pub const DEFSPACE_CX: i32 = 7;

pub const DEFSPACE_CY: i32 = 6;

pub const DEFLISTGAP: i32 = 4;

pub const LISTPAD_CY: i32 = 2;

pub const NONLIST_NOTEXT_OFFSET: i32 = 2;

pub const TOOLBAR_NOWHERE: i32 = -1;

pub const TBSTYLE_EX_ALL: u32 = TBSTYLE_EX_DRAWDDARROWS
    | TBSTYLE_EX_VERTICAL
    | TBSTYLE_EX_MIXEDBUTTONS
    | TBSTYLE_EX_DOUBLEBUFFER
    | TBSTYLE_EX_HIDECLIPPEDBUTTONS;

pub const COMMON_STYLES: u32 = CCS_TOP
    | CCS_NOMOVEY
    | CCS_BOTTOM
    | CCS_NORESIZE
    | CCS_NOPARENTALIGN
    | CCS_ADJUSTABLE
    | CCS_NODIVIDER
    | CCS_VERT;

// Function-like macros GETIBITMAP, GETHIMLID, GETDEFIMAGELIST, GETHOTIMAGELIST,
// GETDISIMAGELIST are skipped (inlined by the functions).

// Prototypes of static functions TOOLBAR_* are skipped (defined by function pipeline).

pub static mut uDragListMessage: UINT = 0;


const VK_MENU: i32 = 0x12;

unsafe extern "C" fn default_top_margin(infoPtr: *const TOOLBAR_INFO) -> i32 {
    if (*infoPtr).iVersion == 6 {
        return (*infoPtr).szBarPadding.cy;
    }
    return if ((*infoPtr).dwStyle & (TBSTYLE_FLAT as DWORD)) != 0 {
        0
    } else {
        TOP_BORDER
    };
}

unsafe extern "C" fn TOOLBAR_HasDropDownArrows(exStyle: DWORD) -> BOOL {
    return ((exStyle & TBSTYLE_EX_DRAWDDARROWS) != 0) as BOOL;
}

unsafe extern "C" fn button_has_ddarrow(infoPtr: *const TOOLBAR_INFO, btnPtr: *const TBUTTON_INFO) -> BOOL {
    return ((TOOLBAR_HasDropDownArrows((*infoPtr).dwExStyle) != 0
        && ((*btnPtr).fsStyle as ::core::ffi::c_int & (BTNS_DROPDOWN as i32)) != 0)
        || ((*btnPtr).fsStyle as ::core::ffi::c_int & (BTNS_WHOLEDROPDOWN as i32)) != 0) as BOOL;
}

unsafe extern "C" fn TOOLBAR_GetButtonDTFlags(infoPtr: *const TOOLBAR_INFO, btnPtr: *const TBUTTON_INFO) -> DWORD {
    let mut dwDTFlags: DWORD = (*infoPtr).dwDTFlags;
    if ((*btnPtr).fsStyle as ::core::ffi::c_int & (BTNS_NOPREFIX as i32)) != 0 {
        dwDTFlags |= DT_NOPREFIX as DWORD;
    }
    return dwDTFlags;
}

unsafe extern "C" fn TOOLBAR_GetText(infoPtr: *const TOOLBAR_INFO, btnPtr: *const TBUTTON_INFO) -> LPWSTR {
    let mut lpText: LPWSTR = core::ptr::null_mut();
    /* NOTE: iString == -1 is undocumented */
    if IS_INTRESOURCE!((*btnPtr).iString as LPWSTR) == false && ((*btnPtr).iString != -1) {
        lpText = (*btnPtr).iString as LPWSTR;
    } else if ((*btnPtr).iString >= 0) && ((*btnPtr).iString < (*infoPtr).nNumStrings as INT_PTR) {
        lpText = *(*infoPtr).strings.offset((*btnPtr).iString as isize);
    }
    return lpText;
}

unsafe extern "C" fn TOOLBAR_DumpTBButton(
    tbb: *const TBBUTTON,
    fUnicode: BOOL,
) {
    // TRACE("TBBUTTON: id %d, bitmap=%d, state=%02x, style=%02x, data=%p, stringid=%p (%s)\n", tbb->idCommand, tbb->iBitmap, tbb->fsState, tbb->fsStyle, (void *)tbb->dwData, (void *)tbb->iString, tbb->iString != -1 ? (fUnicode ? debugstr_w((LPWSTR)tbb->iString) : debugstr_a((LPSTR)tbb->iString)) : "");
}

unsafe extern "C" fn TOOLBAR_DumpButton(
    infoPtr: *const TOOLBAR_INFO,
    bP: *const TBUTTON_INFO,
    btn_num: INT,
) {
    if __wine_dbch_toolbar.flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
        && __wine_dbg_get_channel_flags(core::ptr::addr_of_mut!(__wine_dbch_toolbar)) as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
    {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_DumpButton\0" as *const u8 as *const ::core::ffi::c_char,
                326 as ::core::ffi::c_int,
                b"button %d id %d, bitmap=%d, state=%02x, style=%02x, data=%08lx, stringid=0x%08lx\n\0"
                    as *const u8 as *const ::core::ffi::c_char,
                btn_num,
                (*bP).idCommand,
                if (*infoPtr).iVersion >= 5 as ::core::ffi::c_int {
                    ((*bP).iBitmap as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                        as ::core::ffi::c_int
                } else {
                    (*bP).iBitmap as ::core::ffi::c_int
                },
                (*bP).fsState as ::core::ffi::c_int,
                (*bP).fsStyle as ::core::ffi::c_int,
                (*bP).dwData,
                (*bP).iString,
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
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_DumpButton\0" as *const u8 as *const ::core::ffi::c_char,
                327 as ::core::ffi::c_int,
                b"string %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w(TOOLBAR_GetText(infoPtr, bP) as *const WCHAR),
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
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_DumpButton\0" as *const u8 as *const ::core::ffi::c_char,
                330 as ::core::ffi::c_int,
                b"button %d id %d, hot=%s, row=%d, rect=(%s)\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                btn_num,
                (*bP).idCommand,
                if (*bP).bHot != 0 {
                    b"TRUE\0" as *const u8 as *const ::core::ffi::c_char
                } else {
                    b"FALSE\0" as *const u8 as *const ::core::ffi::c_char
                },
                (*bP).nRow,
                wine_dbgstr_rect(core::ptr::addr_of!((*bP).rect)),
            );
        }
    }
}

unsafe extern "C" fn TOOLBAR_DumpToolbar(iP: *const TOOLBAR_INFO, line: INT) {
    if TRACE_ON!(toolbar) {
        let mut i: INT;
        i = 0;
        while i < (*iP).nNumButtons {
            TOOLBAR_DumpButton(iP, (*iP).buttons.offset(i as isize) as *const TBUTTON_INFO, i);
            i += 1;
        }
    }
}

unsafe extern "C" fn TOOLBAR_ButtonHasString(btnPtr: *const TBUTTON_INFO) -> BOOL {
    return (HIWORD!((*btnPtr).iString as DWORD_PTR) != 0 && (*btnPtr).iString != (-1) as INT_PTR) as BOOL;
}

unsafe extern "C" fn set_string_index(btn: *mut TBUTTON_INFO, str: INT_PTR, unicode: BOOL) {
    if IS_INTRESOURCE!(str as LPCWSTR) == false && str != -(1 as INT_PTR) {
        if TOOLBAR_ButtonHasString(btn) == 0 {
            (*btn).iString = 0 as INT_PTR;
        }

        if unicode != 0 {
            Str_SetPtrW(
                core::ptr::addr_of_mut!((*btn).iString) as *mut LPWSTR,
                str as *mut WCHAR as LPCWSTR,
            );
        } else {
            Str_SetPtrAtoW(
                core::ptr::addr_of_mut!((*btn).iString) as *mut LPWSTR,
                str as *mut i8 as LPCSTR,
            );
        }
    } else {
        if TOOLBAR_ButtonHasString(btn) != 0 {
            Free((*btn).iString as *mut WCHAR as LPVOID);
        }

        (*btn).iString = str;
    }
}

unsafe extern "C" fn set_stringT(btn: *mut TBUTTON_INFO, str: *const WCHAR, unicode: BOOL) {
    if IS_INTRESOURCE!(str as DWORD_PTR) != false || str as DWORD_PTR == -(1 as isize) as DWORD_PTR {
        return;
    }
    set_string_index(btn, str as DWORD_PTR as INT_PTR, unicode);
}

unsafe extern "C" fn free_string(btn: *mut TBUTTON_INFO) {
    set_string_index(btn, 0, TRUE);
}

unsafe extern "C" fn TOOLBAR_CheckStyle(infoPtr: *const TOOLBAR_INFO) {
    if ((*infoPtr).dwStyle & (TBSTYLE_REGISTERDROP as DWORD)) != 0 {
        if ((*(*core::ptr::addr_of_mut!(__wine_dbch___default))).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_FIXME as ::core::ffi::c_int)))
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_CheckStyle\0" as *const u8 as *const ::core::ffi::c_char,
                403 as ::core::ffi::c_int,
                b"[%p] TBSTYLE_REGISTERDROP not implemented\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (*infoPtr).hwndSelf,
            );
        }
    }
}

unsafe extern "C" fn TOOLBAR_SendNotify(nmhdr: *mut NMHDR, infoPtr: *const TOOLBAR_INFO, code: UINT) -> INT {
    if IsWindow((*infoPtr).hwndSelf) == 0 {
        return 0; /* we have just been destroyed */
    }

    (*nmhdr).idFrom = GetDlgCtrlID((*infoPtr).hwndSelf) as UINT_PTR;
    (*nmhdr).hwndFrom = (*infoPtr).hwndSelf;
    (*nmhdr).code = code;

    return SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        (*nmhdr).idFrom as WPARAM,
        nmhdr as LPARAM,
    ) as INT;
}

unsafe extern "C" fn TOOLBAR_GetBitmapIndex(infoPtr: *const TOOLBAR_INFO, btnPtr: *mut TBUTTON_INFO) -> INT {
    let mut ret: INT = (*btnPtr).iBitmap;

    if ret == I_IMAGECALLBACK {
        /* issue TBN_GETDISPINFO */
        let mut nmgd: NMTBDISPINFOW = core::mem::zeroed();

        memset(
            core::ptr::addr_of_mut!(nmgd) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<NMTBDISPINFOW>() as usize,
        );
        nmgd.idCommand = (*btnPtr).idCommand;
        nmgd.lParam = (*btnPtr).dwData;
        nmgd.dwMask = TBNF_IMAGE;
        nmgd.iImage = -(1) as INT;
        /* Windows also send TBN_GETDISPINFOW even if the control is ANSI */
        TOOLBAR_SendNotify(
            core::ptr::addr_of_mut!(nmgd.hdr),
            infoPtr,
            TBN_GETDISPINFOW as UINT,
        );
        if (nmgd.dwMask & TBNF_DI_SETITEM) != 0 {
            (*btnPtr).iBitmap = nmgd.iImage;
        }
        ret = nmgd.iImage;
    }

    if ret != I_IMAGENONE {
        ret = (GETIBITMAP!(infoPtr, (ret) as u16)) as i32;
    }

    return ret;
}

unsafe extern "C" fn TOOLBAR_IsValidBitmapIndex(infoPtr: *const TOOLBAR_INFO, index: INT) -> BOOL {
    let mut himl: HIMAGELIST = core::ptr::null_mut();
    let id: INT = GETHIMLID!(infoPtr, index) as INT;
    let iBitmap: INT = (GETIBITMAP!(infoPtr, (index) as u16)) as i32;

    if ({ himl = GETDEFIMAGELIST!(infoPtr, id); !himl.is_null() }
        && iBitmap >= 0 && iBitmap < ImageList_GetImageCount(himl)) ||
        (index == I_IMAGECALLBACK)
    {
        return TRUE;
    } else {
        return FALSE;
    }
}

unsafe extern "C" fn TOOLBAR_IsValidImageList(infoPtr: *const TOOLBAR_INFO, index: INT) -> BOOL {
    let himl: HIMAGELIST = GETDEFIMAGELIST!(infoPtr, GETHIMLID!(infoPtr, index) as INT);
    return ((himl != core::ptr::null_mut()) && (ImageList_GetImageCount(himl) > 0)) as BOOL;
}

unsafe extern "C" fn TOOLBAR_GetImageListForDrawing(
    infoPtr: *const TOOLBAR_INFO,
    btnPtr: *mut TBUTTON_INFO,
    imagelist: IMAGE_LIST_TYPE,
    index: *mut INT,
) -> HIMAGELIST {
    let mut himl: HIMAGELIST;

    if TOOLBAR_IsValidBitmapIndex(infoPtr, (*btnPtr).iBitmap) == 0 {
        if (*btnPtr).iBitmap == I_IMAGENONE {
            return core::ptr::null_mut();
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_GetImageListForDrawing\0" as *const u8
                    as *const ::core::ffi::c_char,
                500 as ::core::ffi::c_int,
                b"bitmap for ID %d, index %d is not valid, number of bitmaps in imagelist: %d\n\0"
                    as *const u8 as *const ::core::ffi::c_char,
                (((*btnPtr).iBitmap as DWORD_PTR >> 16 as ::core::ffi::c_int)
                    & 0xffff as DWORD_PTR) as WORD as ::core::ffi::c_int,
                (((*btnPtr).iBitmap as DWORD_PTR) & 0xffff as DWORD_PTR) as WORD
                    as ::core::ffi::c_int,
                (*infoPtr).nNumBitmaps,
            );
        }
        return core::ptr::null_mut();
    }

    *index = TOOLBAR_GetBitmapIndex(infoPtr, btnPtr);
    if *index < 0 {
        if *index == I_IMAGECALLBACK || *index == I_IMAGENONE {
            return core::ptr::null_mut();
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_GetImageListForDrawing\0" as *const u8
                    as *const ::core::ffi::c_char,
                508 as ::core::ffi::c_int,
                b"TBN_GETDISPINFO returned invalid index %d\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                *index,
            );
        }
        return core::ptr::null_mut();
    }

    match (imagelist as ::core::ffi::c_uint) as i32 {
        IMAGE_LIST_DEFAULT => {
            himl = GETDEFIMAGELIST!(infoPtr, GETHIMLID!(infoPtr, (*btnPtr).iBitmap) as i32);
        }
        IMAGE_LIST_HOT => {
            himl = GETHOTIMAGELIST!(infoPtr, GETHIMLID!(infoPtr, (*btnPtr).iBitmap));
        }
        IMAGE_LIST_DISABLED => {
            himl = GETDISIMAGELIST!(infoPtr, GETHIMLID!(infoPtr, (*btnPtr).iBitmap));
        }
        _ => {
            himl = core::ptr::null_mut();
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/toolbar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TOOLBAR_GetImageListForDrawing\0" as *const u8
                        as *const ::core::ffi::c_char,
                    525 as ::core::ffi::c_int,
                    b"Shouldn't reach here\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
        }
    }

    if himl.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_GetImageListForDrawing\0" as *const u8
                    as *const ::core::ffi::c_char,
                529 as ::core::ffi::c_int,
                b"no image list\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    }

    return himl;
}

unsafe extern "C" fn TOOLBAR_DrawFlatSeparator(lpRect: *const RECT, hdc: HDC, infoPtr: *const TOOLBAR_INFO) {
    let mut myrect: RECT = core::mem::zeroed();
    let mut oldcolor: COLORREF = 0;
    let mut newcolor: COLORREF = 0;
    myrect.left = (((*lpRect).left + (*lpRect).right) / 2 - 1) as LONG;
    myrect.right = (myrect.left + 1) as LONG;
    myrect.top = ((*lpRect).top + 2) as LONG;
    myrect.bottom = ((*lpRect).bottom - 2) as LONG;

    newcolor = if (*infoPtr).clrBtnShadow == CLR_DEFAULT {
        comctl32_color.clrBtnShadow
    } else {
        (*infoPtr).clrBtnShadow
    };
    oldcolor = SetBkColor(hdc, newcolor);
    ExtTextOutW(
        hdc,
        0,
        0,
        ETO_OPAQUE,
        core::ptr::addr_of_mut!(myrect),
        core::ptr::null(),
        0,
        core::ptr::null(),
    );

    myrect.left = myrect.right;
    myrect.right = (myrect.left + 1) as LONG;

    newcolor = if (*infoPtr).clrBtnHighlight == CLR_DEFAULT {
        comctl32_color.clrBtnHighlight
    } else {
        (*infoPtr).clrBtnHighlight
    };
    SetBkColor(hdc, newcolor);
    ExtTextOutW(
        hdc,
        0,
        0,
        ETO_OPAQUE,
        core::ptr::addr_of_mut!(myrect),
        core::ptr::null(),
        0,
        core::ptr::null(),
    );

    SetBkColor(hdc, oldcolor);
}

unsafe extern "C" fn TOOLBAR_DrawFlatHorizontalSeparator(
    lpRect: *const RECT,
    hdc: HDC,
    infoPtr: *const TOOLBAR_INFO,
) {
    let mut myrect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut oldcolor: COLORREF = 0;
    let mut newcolor: COLORREF = 0;

    myrect.left = (*lpRect).left;
    myrect.right = (*lpRect).right;
    myrect.top = (*lpRect).top + ((*lpRect).bottom - (*lpRect).top - 2) / 2;
    myrect.bottom = myrect.top + 1;

    InflateRect(core::ptr::addr_of_mut!(myrect), -2, 0);

    if (*__wine_dbch___default).flags & (1 << __WINE_DBCL_TRACE) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0".as_ptr() as *const i8,
            b"TOOLBAR_DrawFlatHorizontalSeparator\0".as_ptr() as *const i8,
            588,
            b"rect=(%s)\n\0".as_ptr() as *const i8,
            wine_dbgstr_rect(core::ptr::addr_of_mut!(myrect)),
        );
    }

    newcolor = if (*infoPtr).clrBtnShadow == CLR_DEFAULT {
        comctl32_color.clrBtnShadow
    } else {
        (*infoPtr).clrBtnShadow
    };
    oldcolor = SetBkColor(hdc, newcolor);
    ExtTextOutW(
        hdc,
        0,
        0,
        ETO_OPAQUE,
        core::ptr::addr_of_mut!(myrect),
        core::ptr::null(),
        0,
        core::ptr::null(),
    );

    myrect.top = myrect.bottom;
    myrect.bottom = myrect.top + 1;

    newcolor = if (*infoPtr).clrBtnHighlight == CLR_DEFAULT {
        comctl32_color.clrBtnHighlight
    } else {
        (*infoPtr).clrBtnHighlight
    };
    SetBkColor(hdc, newcolor);
    ExtTextOutW(
        hdc,
        0,
        0,
        ETO_OPAQUE,
        core::ptr::addr_of_mut!(myrect),
        core::ptr::null(),
        0,
        core::ptr::null(),
    );

    SetBkColor(hdc, oldcolor);
}

unsafe extern "C" fn TOOLBAR_DrawArrow(mut hdc: HDC, mut left: INT, mut top: INT, mut clr: COLORREF) {
    let mut x: INT = 0;
    let mut y: INT = 0;
    let mut hPen: HPEN = core::ptr::null_mut();
    let mut hOldPen: HPEN = core::ptr::null_mut();
    hPen = CreatePen((PS_SOLID as i32), 1, clr);
    if hPen.is_null() {
        return;
    }
    hOldPen = SelectObject(hdc, hPen as HGDIOBJ) as HPEN;
    x = left + 2;
    y = top;
    MoveToEx(hdc, x, y, core::ptr::null_mut());
    let fresh22 = y;
    y = y + 1;
    LineTo(hdc, x + 5, fresh22);
    x += 1;
    MoveToEx(hdc, x, y, core::ptr::null_mut());
    let fresh23 = y;
    y = y + 1;
    LineTo(hdc, x + 3, fresh23);
    x += 1;
    MoveToEx(hdc, x, y, core::ptr::null_mut());
    LineTo(hdc, x + 1, y);
    SelectObject(hdc, hOldPen as HGDIOBJ);
    DeleteObject(hPen as HGDIOBJ);
}

unsafe extern "C" fn TOOLBAR_DrawString(
    mut infoPtr: *const TOOLBAR_INFO,
    mut rcText: *mut RECT,
    mut lpText: LPCWSTR,
    mut btnPtr: *const TBUTTON_INFO,
    mut tbcd: *const NMTBCUSTOMDRAW,
    mut dwItemCDFlag: DWORD,
) {
    let mut hdc: HDC = (*tbcd).nmcd.hdc;
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut clrOld: COLORREF = 0;
    let mut clrOldBk: COLORREF = 0;
    let mut oldBkMode: ::core::ffi::c_int = 0;
    let mut state: UINT = (*tbcd).nmcd.uItemState;
    let mut theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    let mut dwDTFlags: DWORD = TOOLBAR_GetButtonDTFlags(infoPtr, btnPtr);

    /* draw text */
    if !lpText.is_null() && (*infoPtr).nMaxTextRows > 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_DrawString\0" as *const u8 as *const ::core::ffi::c_char,
                654 as ::core::ffi::c_int,
                b"string=%s rect=(%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w(lpText as *const WCHAR),
                wine_dbgstr_rect(rcText),
            );
        }

        hOldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;
        if !theme.is_null() {
            let mut dwDTFlags2: DWORD = 0;
            let mut partId: ::core::ffi::c_int = (TP_BUTTON) as i32;
            let mut stateId: ::core::ffi::c_int = (TS_NORMAL) as i32;

            if state & CDIS_DISABLED as UINT != 0 {
                stateId = (TS_DISABLED) as i32;
                dwDTFlags2 = DTT_GRAYED;
            } else if state & CDIS_SELECTED as UINT != 0 {
                stateId = (TS_PRESSED) as i32;
            } else if state & CDIS_CHECKED as UINT != 0 {
                stateId = if state & CDIS_HOT as UINT != 0 {
                    (TS_HOTCHECKED) as i32
                } else {
                    (TS_HOT) as i32
                };
            } else if state & CDIS_HOT as UINT != 0 {
                stateId = (TS_HOT) as i32;
            }

            DrawThemeText(
                theme,
                hdc,
                partId,
                stateId,
                lpText,
                -(1 as ::core::ffi::c_int),
                dwDTFlags,
                dwDTFlags2,
                rcText,
            );
            SelectObject(hdc, hOldFont as HGDIOBJ);
            return;
        }

        if state & CDIS_HOT as UINT != 0
            && dwItemCDFlag & TBCDRF_HILITEHOTTRACK as DWORD != 0
        {
            clrOld = SetTextColor(hdc, (*tbcd).clrTextHighlight);
        } else if state & CDIS_DISABLED as UINT != 0 {
            clrOld = SetTextColor(hdc, (*tbcd).clrBtnHighlight);
            OffsetRect(
                rcText as LPRECT,
                1 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
            );
            DrawTextW(
                hdc,
                lpText,
                -(1 as ::core::ffi::c_int),
                rcText as LPRECT,
                dwDTFlags as UINT,
            );
            SetTextColor(hdc, comctl32_color.clr3dShadow);
            OffsetRect(
                rcText as LPRECT,
                -(1 as ::core::ffi::c_int),
                -(1 as ::core::ffi::c_int),
            );
        } else if state & CDIS_INDETERMINATE as UINT != 0 {
            clrOld = SetTextColor(hdc, comctl32_color.clr3dShadow);
        } else if state & CDIS_MARKED as UINT != 0
            && !(dwItemCDFlag & TBCDRF_NOMARK as DWORD != 0)
        {
            clrOld = SetTextColor(hdc, (*tbcd).clrTextHighlight);
            clrOldBk = SetBkColor(hdc, (*tbcd).clrMark);
            oldBkMode = SetBkMode(hdc, (*tbcd).nHLStringBkMode);
        } else {
            clrOld = SetTextColor(hdc, (*tbcd).clrText);
        }

        DrawTextW(
            hdc,
            lpText,
            -(1 as ::core::ffi::c_int),
            rcText as LPRECT,
            dwDTFlags as UINT,
        );
        SetTextColor(hdc, clrOld);
        if state & CDIS_MARKED as UINT != 0
            && !(dwItemCDFlag & TBCDRF_NOMARK as DWORD != 0)
        {
            SetBkColor(hdc, clrOldBk);
            SetBkMode(hdc, oldBkMode);
        }
        SelectObject(hdc, hOldFont as HGDIOBJ);
    }
}

unsafe extern "C" fn TOOLBAR_DrawPattern(lpRect: *const RECT, tbcd: *const NMTBCUSTOMDRAW) {
    let hdc: HDC = (*tbcd).nmcd.hdc;
    let hbr: HBRUSH = SelectObject(hdc, (*tbcd).hbrMonoDither as HGDIOBJ) as HBRUSH;
    let mut clrTextOld: COLORREF = 0;
    let mut clrBkOld: COLORREF = 0;
    let cx: INT = (*lpRect).right - (*lpRect).left;
    let cy: INT = (*lpRect).bottom - (*lpRect).top;
    let cxEdge: INT = GetSystemMetrics((SM_CXEDGE) as i32);
    let cyEdge: INT = GetSystemMetrics((SM_CYEDGE) as i32);
    clrTextOld = SetTextColor(hdc, (*tbcd).clrBtnHighlight);
    clrBkOld = SetBkColor(hdc, (*tbcd).clrBtnFace);
    PatBlt(
        hdc,
        (*lpRect).left + cxEdge,
        (*lpRect).top + cyEdge,
        cx - (2 * cxEdge),
        cy - (2 * cyEdge),
        PATCOPY,
    );
    SetBkColor(hdc, clrBkOld);
    SetTextColor(hdc, clrTextOld);
    SelectObject(hdc, hbr as HGDIOBJ);
}

unsafe extern "C" fn TOOLBAR_DrawMasked(himl: HIMAGELIST, index: INT, hdc: HDC, x: INT, y: INT, draw_flags: UINT) {
    let mut cx: INT = 0;
    let mut cy: INT = 0;
    let mut hbmMask: HBITMAP = core::ptr::null_mut();
    let mut hbmImage: HBITMAP = core::ptr::null_mut();
    let mut hdcMask: HDC = core::ptr::null_mut();
    let mut hdcImage: HDC = core::ptr::null_mut();

    ImageList_GetIconSize(himl, core::ptr::addr_of_mut!(cx), core::ptr::addr_of_mut!(cy));

    /* Create src image */
    hdcImage = CreateCompatibleDC(hdc);
    hbmImage = CreateCompatibleBitmap(hdc, cx, cy);
    SelectObject(hdcImage, hbmImage as HGDIOBJ);
    ImageList_DrawEx(himl, index, hdcImage, 0, 0, cx, cy,
                     RGB!(0xff, 0xff, 0xff), RGB!(0, 0, 0), draw_flags);

    /* Create Mask */
    hdcMask = CreateCompatibleDC(core::ptr::null_mut());
    hbmMask = CreateBitmap(cx, cy, 1, 1, core::ptr::null());
    SelectObject(hdcMask, hbmMask as HGDIOBJ);

    /* Remove the background and all white pixels */
    ImageList_DrawEx(himl, index, hdcMask, 0, 0, cx, cy,
                     RGB!(0xff, 0xff, 0xff), RGB!(0, 0, 0), ILD_MASK);
    SetBkColor(hdcImage, RGB!(0xff, 0xff, 0xff));
    BitBlt(hdcMask, 0, 0, cx, cy, hdcImage, 0, 0, NOTSRCERASE);

    /* draw the new mask 'etched' to hdc */
    SetBkColor(hdc, RGB!(255, 255, 255));
    SelectObject(hdc, GetSysColorBrush((COLOR_3DHILIGHT) as i32) as HGDIOBJ);
    /* E20746 op code is (Dst ^ (Src & (Pat ^ Dst))) */
    BitBlt(hdc, x + 1, y + 1, cx, cy, hdcMask, 0, 0, 0xE20746);
    SelectObject(hdc, GetSysColorBrush((COLOR_3DSHADOW) as i32) as HGDIOBJ);
    BitBlt(hdc, x, y, cx, cy, hdcMask, 0, 0, 0xE20746);

    /* Cleanup */
    DeleteObject(hbmImage as HGDIOBJ);
    DeleteDC(hdcImage);
    DeleteObject(hbmMask as HGDIOBJ);
    DeleteDC(hdcMask);
}

unsafe extern "C" fn TOOLBAR_TranslateState(
    mut btnPtr: *const TBUTTON_INFO,
    mut captured: BOOL,
) -> UINT {
    let mut retstate: UINT = 0 as UINT;
    retstate |= (if (*btnPtr).fsState as ::core::ffi::c_int & (TBSTATE_CHECKED as i32) != 0 {
        CDIS_CHECKED as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as UINT;
    retstate |= (if (*btnPtr).fsState as ::core::ffi::c_int & (TBSTATE_PRESSED as i32) != 0 {
        CDIS_SELECTED as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as UINT;
    retstate |= (if (*btnPtr).fsState as ::core::ffi::c_int & (TBSTATE_ENABLED as i32) != 0 {
        0 as ::core::ffi::c_int
    } else {
        CDIS_DISABLED as ::core::ffi::c_int
    }) as UINT;
    retstate |= (if (*btnPtr).fsState as ::core::ffi::c_int & (TBSTATE_MARKED as i32) != 0 {
        CDIS_MARKED as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as UINT;
    retstate |=
        (if (*btnPtr).bHot as ::core::ffi::c_int & (captured == 0) as ::core::ffi::c_int != 0 {
            CDIS_HOT as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as UINT;
    retstate |= (if (((*btnPtr).fsState as ::core::ffi::c_int
        & ((TBSTATE_ENABLED | TBSTATE_INDETERMINATE) as ::core::ffi::c_int)))
        == ((TBSTATE_ENABLED | TBSTATE_INDETERMINATE) as ::core::ffi::c_int)
    {
        CDIS_INDETERMINATE as ::core::ffi::c_int
    } else {
        0 as ::core::ffi::c_int
    }) as UINT;
    return retstate;
}

unsafe extern "C" fn TOOLBAR_DrawImage(
    mut infoPtr: *const TOOLBAR_INFO,
    mut btnPtr: *mut TBUTTON_INFO,
    mut left: INT,
    mut top: INT,
    mut tbcd: *const NMTBCUSTOMDRAW,
    mut dwItemCDFlag: DWORD,
) {
    let mut himl: HIMAGELIST = ::core::ptr::null_mut::<_IMAGELIST>();
    let mut draw_masked: BOOL = FALSE as BOOL;
    let mut draw_desaturated: BOOL = FALSE as BOOL;
    let mut index: INT = 0;
    let mut offset: INT = 0 as INT;
    let mut draw_flags: UINT = ILD_TRANSPARENT as UINT;
    let mut info: IMAGEINFO = _IMAGEINFO {
        hbmImage: ::core::ptr::null_mut::<HBITMAP__>(),
        hbmMask: ::core::ptr::null_mut::<HBITMAP__>(),
        Unused1: 0,
        Unused2: 0,
        rcImage: tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
    };
    let mut bm: BITMAP = tagBITMAP {
        bmType: 0 as LONG,
        bmWidth: 0,
        bmHeight: 0,
        bmWidthBytes: 0,
        bmPlanes: 0,
        bmBitsPixel: 0,
        bmBits: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    if ((*tbcd).nmcd.uItemState & (CDIS_DISABLED | CDIS_INDETERMINATE) as UINT) != 0 {
        himl = TOOLBAR_GetImageListForDrawing(infoPtr, btnPtr, IMAGE_LIST_DISABLED as INT, &raw mut index);
        if himl.is_null() {
            himl = TOOLBAR_GetImageListForDrawing(infoPtr, btnPtr, IMAGE_LIST_DEFAULT as INT, &raw mut index);

            ImageList_GetImageInfo(himl, index as ::core::ffi::c_int, &raw mut info);
            GetObjectW(
                info.hbmImage as HANDLE,
                ::core::mem::size_of::<BITMAP>() as ::core::ffi::c_int,
                &raw mut bm as LPVOID,
            );

            if bm.bmBitsPixel as ::core::ffi::c_int == 32 as ::core::ffi::c_int {
                draw_desaturated = TRUE as BOOL;
            } else {
                draw_masked = TRUE as BOOL;
            }
        }
    } else if ((*tbcd).nmcd.uItemState & CDIS_CHECKED as UINT) != 0
        || (((*tbcd).nmcd.uItemState & CDIS_HOT as UINT) != 0
            && (((*infoPtr).dwStyle & TBSTYLE_FLAT as DWORD) != 0
                || !GetWindowTheme((*infoPtr).hwndSelf).is_null()))
    {
        /* if hot, attempt to draw with hot image list, if fails,
           use default image list */
        himl = TOOLBAR_GetImageListForDrawing(infoPtr, btnPtr, IMAGE_LIST_HOT as INT, &raw mut index);
        if himl.is_null() {
            himl = TOOLBAR_GetImageListForDrawing(infoPtr, btnPtr, IMAGE_LIST_DEFAULT as INT, &raw mut index);
        }
    } else {
        himl = TOOLBAR_GetImageListForDrawing(infoPtr, btnPtr, IMAGE_LIST_DEFAULT as INT, &raw mut index);
    }

    if himl.is_null() {
        return;
    }

    if (dwItemCDFlag & (TBCDRF_NOOFFSET as DWORD)) == 0
        && ((*tbcd).nmcd.uItemState & (CDIS_SELECTED | CDIS_CHECKED) as UINT) != 0
    {
        offset = 1 as INT;
    }

    if (dwItemCDFlag & (TBCDRF_NOMARK as DWORD)) == 0
        && ((*tbcd).nmcd.uItemState & CDIS_MARKED as UINT) != 0
    {
        draw_flags |= ILD_BLEND50 as UINT;
    }

    if draw_masked != 0 {
        /* code path for drawing flat disabled icons without alpha channel */
        TOOLBAR_DrawMasked(
            himl,
            index as ::core::ffi::c_int,
            (*tbcd).nmcd.hdc,
            left + offset,
            top + offset,
            draw_flags,
        );
    } else if draw_desaturated != 0 {
        /* code path for drawing disabled, alpha-blended (32bpp) icons */
        let mut imldp: IMAGELISTDRAWPARAMS = _IMAGELISTDRAWPARAMS {
            cbSize: 0 as DWORD,
            himl: (::core::ptr::null_mut::<_IMAGELIST>()).cast(),
            i: 0,
            hdcDst: ::core::ptr::null_mut::<HDC__>(),
            x: 0,
            y: 0,
            cx: 0,
            cy: 0,
            xBitmap: 0,
            yBitmap: 0,
            rgbBk: 0,
            rgbFg: 0,
            fStyle: 0,
            dwRop: 0,
            fState: 0,
            Frame: 0,
            crEffect: 0,
        };

        imldp.cbSize = ::core::mem::size_of::<IMAGELISTDRAWPARAMS>() as DWORD;
        imldp.himl = (himl).cast();
        imldp.i = index as ::core::ffi::c_int;
        imldp.hdcDst = (*tbcd).nmcd.hdc;
        imldp.x = (offset + left) as ::core::ffi::c_int;
        imldp.y = (offset + top) as ::core::ffi::c_int;
        imldp.rgbBk = CLR_NONE as COLORREF;
        imldp.rgbFg = CLR_DEFAULT as COLORREF;
        imldp.fStyle = ILD_TRANSPARENT as UINT;
        imldp.fState = (ILS_ALPHA | ILS_SATURATE) as DWORD;
        imldp.Frame = 192 as DWORD;

        ImageList_DrawIndirect(&raw mut imldp);
    } else {
        /* code path for drawing standard icons as-is */
        ImageList_Draw(
            himl,
            index as ::core::ffi::c_int,
            (*tbcd).nmcd.hdc,
            left + offset,
            top + offset,
            draw_flags,
        );
    }
}

unsafe extern "C" fn TOOLBAR_DrawFrame(infoPtr: *const TOOLBAR_INFO, tbcd: *const NMTBCUSTOMDRAW, rect: *const RECT, dwItemCDFlag: DWORD) {
    let hdc: HDC = (*tbcd).nmcd.hdc;
    let mut rc: RECT = *rect;
    /* if the state is disabled or indeterminate then the button
     * cannot have an interactive look like pressed or hot */
    let non_interactive_state: BOOL = ((((*tbcd).nmcd.uItemState & (CDIS_DISABLED as UINT)) != 0)
        || (((*tbcd).nmcd.uItemState & (CDIS_INDETERMINATE as UINT)) != 0)) as i32;
    let pressed_look: BOOL = ((non_interactive_state == 0)
        && ((((*tbcd).nmcd.uItemState & (CDIS_SELECTED as UINT)) != 0)
            || (((*tbcd).nmcd.uItemState & (CDIS_CHECKED as UINT)) != 0))) as i32;

    /* app don't want us to draw any edges */
    if (dwItemCDFlag & (TBCDRF_NOEDGES as DWORD)) != 0 {
        return;
    }

    if ((*infoPtr).dwStyle & (TBSTYLE_FLAT as DWORD)) != 0 {
        if pressed_look != 0 {
            DrawEdge(hdc, core::ptr::addr_of_mut!(rc), BDR_SUNKENOUTER as UINT, BF_RECT as UINT);
        } else if (((*tbcd).nmcd.uItemState & (CDIS_HOT as UINT)) != 0) && non_interactive_state == 0 {
            DrawEdge(hdc, core::ptr::addr_of_mut!(rc), BDR_RAISEDINNER as UINT, BF_RECT as UINT);
        }
    } else {
        if pressed_look != 0 {
            DrawEdge(hdc, core::ptr::addr_of_mut!(rc), EDGE_SUNKEN as UINT, (BF_RECT | BF_MIDDLE) as UINT);
        } else {
            DrawEdge(hdc, core::ptr::addr_of_mut!(rc), EDGE_RAISED as UINT,
                (BF_SOFT | BF_RECT | BF_MIDDLE) as UINT);
        }
    }
}

unsafe extern "C" fn TOOLBAR_DrawSepDDArrow(
    infoPtr: *const TOOLBAR_INFO,
    tbcd: *const NMTBCUSTOMDRAW,
    rcArrow: *mut RECT,
    bDropDownPressed: BOOL,
    dwItemCDFlag: DWORD,
) {
    let hdc: HDC = (*tbcd).nmcd.hdc;
    let mut offset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut pressed: BOOL = (bDropDownPressed != 0
        || ((*tbcd).nmcd.uItemState & ((CDIS_SELECTED | CDIS_CHECKED) as UINT)) != 0)
        as ::core::ffi::c_int;

    if ((*infoPtr).dwStyle & (TBSTYLE_FLAT as DWORD)) != 0 {
        if pressed != 0 {
            DrawEdge(
                hdc,
                rcArrow as LPRECT,
                (BDR_SUNKENOUTER) as UINT,
                (BF_RECT) as UINT,
            );
        } else if ((*tbcd).nmcd.uItemState & (CDIS_HOT as UINT)) != 0
            && ((*tbcd).nmcd.uItemState & (CDIS_DISABLED as UINT)) == 0
            && ((*tbcd).nmcd.uItemState & (CDIS_INDETERMINATE as UINT)) == 0
        {
            DrawEdge(
                hdc,
                rcArrow as LPRECT,
                (BDR_RAISEDINNER) as UINT,
                (BF_RECT) as UINT,
            );
        }
    } else {
        if pressed != 0 {
            DrawEdge(
                hdc,
                rcArrow as LPRECT,
                (EDGE_SUNKEN) as UINT,
                ((BF_RECT | BF_MIDDLE) as UINT),
            );
        } else {
            DrawEdge(
                hdc,
                rcArrow as LPRECT,
                (EDGE_RAISED) as UINT,
                ((BF_SOFT | BF_RECT | BF_MIDDLE) as UINT),
            );
        }
    }

    if pressed != 0 {
        offset = if (dwItemCDFlag & (TBCDRF_NOOFFSET as DWORD)) != 0 {
            0 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    }

    if ((*tbcd).nmcd.uItemState & ((CDIS_DISABLED | CDIS_INDETERMINATE) as UINT)) != 0 {
        TOOLBAR_DrawArrow(
            hdc,
            (*rcArrow).left as INT + (1) as INT,
            (*rcArrow).top as INT
                + (1) as INT
                + ((*rcArrow).bottom as INT - (*rcArrow).top as INT - (ARROW_HEIGHT) as INT)
                    / (2) as INT,
            comctl32_color.clrBtnHighlight,
        );
        TOOLBAR_DrawArrow(
            hdc,
            (*rcArrow).left as INT,
            (*rcArrow).top as INT
                + ((*rcArrow).bottom as INT - (*rcArrow).top as INT - (ARROW_HEIGHT) as INT)
                    / (2) as INT,
            comctl32_color.clr3dShadow,
        );
    } else {
        TOOLBAR_DrawArrow(
            hdc,
            (*rcArrow).left as INT + offset as INT,
            (*rcArrow).top as INT
                + offset as INT
                + ((*rcArrow).bottom as INT - (*rcArrow).top as INT - (ARROW_HEIGHT) as INT)
                    / (2) as INT,
            comctl32_color.clrBtnText,
        );
    }
}

unsafe extern "C" fn TOOLBAR_DrawButton(
    infoPtr: *const TOOLBAR_INFO,
    btnPtr: *mut TBUTTON_INFO,
    hdc: HDC,
    dwBaseCustDraw: DWORD,
) {
    let dwStyle: DWORD = (*infoPtr).dwStyle;
    let hasDropDownArrow: BOOL = button_has_ddarrow(infoPtr, btnPtr);
    let drawSepDropDownArrow: BOOL = (hasDropDownArrow != 0
        && ((!((*btnPtr).fsStyle as DWORD)) & BTNS_WHOLEDROPDOWN) != 0) as BOOL;
    let mut rc: RECT = core::mem::zeroed();
    let mut rcArrow: RECT = core::mem::zeroed();
    let mut rcBitmap: RECT = core::mem::zeroed();
    let mut rcText: RECT = core::mem::zeroed();
    let mut lpText: LPWSTR = core::ptr::null_mut();
    let mut tbcd: NMTBCUSTOMDRAW = core::mem::zeroed();
    let mut ntfret: DWORD = 0;
    let mut offset: INT = 0;
    let mut oldBkMode: INT = 0;
    let mut dwItemCustDraw: DWORD = 0;
    let mut dwItemCDFlag: DWORD = 0;
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);

    rc = (*btnPtr).rect;
    rcArrow = rc;

    /* separator - doesn't send NM_CUSTOMDRAW */
    if ((*btnPtr).fsStyle as DWORD) & BTNS_SEP != 0 {
        if !theme.is_null() {
            DrawThemeBackground(
                theme,
                hdc,
                if dwStyle & CCS_VERT != 0 {
                    (TP_SEPARATORVERT) as i32
                } else {
                    (TP_SEPARATOR) as i32
                },
                0,
                core::ptr::addr_of_mut!(rc),
                core::ptr::null(),
            );
        } else if (dwStyle & TBSTYLE_FLAT) != 0 {
            if (dwStyle & CCS_VERT) != 0 {
                let mut rcsep: RECT = rc;
                InflateRect(
                    core::ptr::addr_of_mut!(rcsep),
                    -(*infoPtr).szPadding.cx,
                    -(*infoPtr).szPadding.cy,
                );
                TOOLBAR_DrawFlatHorizontalSeparator(core::ptr::addr_of_mut!(rcsep), hdc, infoPtr);
            } else {
                TOOLBAR_DrawFlatSeparator(core::ptr::addr_of_mut!(rc), hdc, infoPtr);
            }
        } else if (*btnPtr).fsStyle as DWORD != BTNS_SEP {
        }
        return;
    }

    /* get a pointer to the text */
    lpText = TOOLBAR_GetText(infoPtr, btnPtr);

    if hasDropDownArrow != 0 {
        let mut right: INT = 0;

        if (dwStyle & TBSTYLE_FLAT) != 0 {
            right = max(rc.left, rc.right - DDARROW_WIDTH) as INT;
        } else {
            right = max(
                rc.left,
                rc.right - DDARROW_WIDTH - 2,
            ) as INT;
        }

        if drawSepDropDownArrow != 0 {
            rc.right = right;
        }

        rcArrow.left = right;
    }

    /* copy text & bitmap rects after adjusting for drop-down arrow
     * so that text & bitmap is centered in the rectangle not containing
     * the arrow */
    rcText = rc;
    rcBitmap = rc;

    /* Center the bitmap horizontally and vertically */
    if (dwStyle & TBSTYLE_LIST) != 0 {
        if !lpText.is_null()
            && (*infoPtr).nMaxTextRows > 0
            && (((*infoPtr).dwExStyle & TBSTYLE_EX_MIXEDBUTTONS) == 0
                || ((*btnPtr).fsStyle as DWORD & BTNS_SHOWTEXT) != 0)
        {
            rcBitmap.left += GetSystemMetrics((SM_CXEDGE) as i32)
                + (*infoPtr).szPadding.cx / 2;
        } else {
            rcBitmap.left += GetSystemMetrics((SM_CXEDGE) as i32)
                + (*infoPtr).iListGap / 2;
        }
    } else {
        rcBitmap.left += ((rc.right - rc.left) - (*infoPtr).nBitmapWidth) / 2;
    }

    rcBitmap.top += (*infoPtr).szPadding.cy / 2;
    rcBitmap.top += (*infoPtr).themeMargins.cyTopHeight;

    /* calculate text position */
    if !lpText.is_null() {
        InflateRect(
            core::ptr::addr_of_mut!(rcText),
            -GetSystemMetrics((SM_CXEDGE) as i32),
            0,
        );
        if (dwStyle & TBSTYLE_LIST) != 0 {
            rcText.left += (*infoPtr).nBitmapWidth + (*infoPtr).iListGap + 2;
        } else {
            if ImageList_GetImageCount(GETDEFIMAGELIST!(infoPtr, 0))
                > 0
            {
                rcText.top += (*infoPtr).szPadding.cy / 2
                    + (*infoPtr).nBitmapHeight
                    + 1;
            } else {
                rcText.top += (*infoPtr).szPadding.cy / 2
                    + 2;
            }
        }
    }

    /* Initialize fields in all cases, because we use these later
     * NOTE: applications can and do alter these to customize their
     * toolbars */
    ZeroMemory!(
        core::ptr::addr_of_mut!(tbcd) as *mut core::ffi::c_void,
        core::mem::size_of::<NMTBCUSTOMDRAW>() as usize,
    );
    tbcd.clrText = comctl32_color.clrBtnText;
    tbcd.clrTextHighlight = comctl32_color.clrHighlightText;
    tbcd.clrBtnFace = comctl32_color.clrBtnFace;
    tbcd.clrBtnHighlight = comctl32_color.clrBtnHighlight;
    tbcd.clrMark = comctl32_color.clrHighlight;
    tbcd.clrHighlightHotTrack = 0;
    tbcd.nStringBkMode = (TRANSPARENT as i32);
    tbcd.nHLStringBkMode = (OPAQUE as i32);
    tbcd.rcText.left = 0;
    tbcd.rcText.top = 0;
    tbcd.rcText.right = rcText.right - rc.left;
    tbcd.rcText.bottom = rcText.bottom - rc.top;
    tbcd.nmcd.uItemState = TOOLBAR_TranslateState(btnPtr, (*infoPtr).bCaptured);
    tbcd.nmcd.hdc = hdc;
    tbcd.nmcd.rc = (*btnPtr).rect;
    tbcd.hbrMonoDither = COMCTL32_hPattern55AABrush;

    /* FIXME: what are these used for? */
    tbcd.hbrLines = core::ptr::null_mut();
    tbcd.hpenLines = core::ptr::null_mut();

    /* Issue Item Prepaint notify */
    dwItemCustDraw = 0;
    dwItemCDFlag = 0;
    if (dwBaseCustDraw & CDRF_NOTIFYITEMDRAW) != 0 {
        tbcd.nmcd.dwDrawStage = CDDS_ITEMPREPAINT;
        tbcd.nmcd.dwItemSpec = (*btnPtr).idCommand as DWORD_PTR;
        tbcd.nmcd.lItemlParam = (*btnPtr).dwData as LPARAM;
        ntfret = TOOLBAR_SendNotify(
            core::ptr::addr_of_mut!(tbcd.nmcd.hdr),
            infoPtr,
            NM_CUSTOMDRAW,
        ) as DWORD;
        /* reset these fields so the user can't alter the behaviour like native */
        tbcd.nmcd.hdc = hdc;
        tbcd.nmcd.rc = (*btnPtr).rect;

        dwItemCustDraw = ntfret & 0xffff;
        dwItemCDFlag = ntfret & 0xffff0000;
        if (dwItemCustDraw & CDRF_SKIPDEFAULT) != 0 {
            return;
        }
        /* save the only part of the rect that the user can change */
        rcText.right = tbcd.rcText.right + rc.left;
        rcText.bottom = tbcd.rcText.bottom + rc.top;
    }

    if (dwItemCDFlag & TBCDRF_NOOFFSET) == 0
        && ((*btnPtr).fsState as DWORD & (TBSTATE_PRESSED | TBSTATE_CHECKED)) != 0
    {
        OffsetRect(
            core::ptr::addr_of_mut!(rcText),
            1,
            1,
        );
    }

    if (tbcd.nmcd.uItemState & CDIS_HOT) == 0
        && ((tbcd.nmcd.uItemState & CDIS_CHECKED) != 0
            || (tbcd.nmcd.uItemState & CDIS_INDETERMINATE) != 0)
    {
        TOOLBAR_DrawPattern(core::ptr::addr_of_mut!(rc), core::ptr::addr_of_mut!(tbcd));
    }

    if (((*infoPtr).dwStyle & TBSTYLE_FLAT) != 0
        || !GetWindowTheme((*infoPtr).hwndSelf).is_null())
        && (tbcd.nmcd.uItemState & CDIS_HOT) != 0
    {
        if (dwItemCDFlag & TBCDRF_HILITEHOTTRACK) != 0 {
            let mut oldclr: COLORREF = 0;

            oldclr = SetBkColor(hdc, tbcd.clrHighlightHotTrack);
            ExtTextOutW(
                hdc,
                0,
                0,
                ETO_OPAQUE,
                core::ptr::addr_of_mut!(rc),
                core::ptr::null(),
                0,
                core::ptr::null(),
            );
            if hasDropDownArrow != 0 {
                ExtTextOutW(
                    hdc,
                    0,
                    0,
                    ETO_OPAQUE,
                    core::ptr::addr_of_mut!(rcArrow),
                    core::ptr::null(),
                    0,
                    core::ptr::null(),
                );
            }
            SetBkColor(hdc, oldclr);
        }
    }

    if !theme.is_null() && (dwItemCDFlag & TBCDRF_NOBACKGROUND) == 0 {
        let partId: INT = if drawSepDropDownArrow != 0 {
            (TP_SPLITBUTTON) as i32
        } else {
            (TP_BUTTON) as i32
        };
        let mut stateId: INT = (TS_NORMAL) as i32;

        if (tbcd.nmcd.uItemState & CDIS_DISABLED) != 0 {
            stateId = (TS_DISABLED) as i32;
        } else if (tbcd.nmcd.uItemState & CDIS_SELECTED) != 0 {
            stateId = (TS_PRESSED) as i32;
        } else if (tbcd.nmcd.uItemState & CDIS_CHECKED) != 0 {
            stateId = if (tbcd.nmcd.uItemState & CDIS_HOT) != 0 {
                (TS_HOTCHECKED) as i32
            } else {
                (TS_CHECKED) as i32
            };
        } else if (tbcd.nmcd.uItemState & CDIS_HOT) != 0
            || (drawSepDropDownArrow != 0 && (*btnPtr).bDropDownPressed != 0)
        {
            stateId = (TS_HOT) as i32;
        }

        DrawThemeBackground(
            theme,
            hdc,
            partId,
            stateId,
            core::ptr::addr_of_mut!(rc),
            core::ptr::null(),
        );
    }

    if theme.is_null() {
        TOOLBAR_DrawFrame(
            infoPtr,
            core::ptr::addr_of_mut!(tbcd),
            core::ptr::addr_of_mut!(rc),
            dwItemCDFlag,
        );
    }

    if drawSepDropDownArrow != 0 {
        if !theme.is_null() {
            let mut stateId: INT = (TS_NORMAL) as i32;

            if (tbcd.nmcd.uItemState & CDIS_DISABLED) != 0 {
                stateId = (TS_DISABLED) as i32;
            } else if (*btnPtr).bDropDownPressed != 0
                || (tbcd.nmcd.uItemState & CDIS_SELECTED) != 0
            {
                stateId = (TS_PRESSED) as i32;
            } else if (tbcd.nmcd.uItemState & CDIS_CHECKED) != 0 {
                stateId = if (tbcd.nmcd.uItemState & CDIS_HOT) != 0 {
                    (TS_HOTCHECKED) as i32
                } else {
                    (TS_CHECKED) as i32
                };
            } else if (tbcd.nmcd.uItemState & CDIS_HOT) != 0 {
                stateId = (TS_HOT) as i32;
            }

            DrawThemeBackground(
                theme,
                hdc,
                (TP_DROPDOWNBUTTON) as i32,
                stateId,
                core::ptr::addr_of_mut!(rcArrow),
                core::ptr::null(),
            );
            DrawThemeBackground(
                theme,
                hdc,
                (TP_SPLITBUTTONDROPDOWN) as i32,
                stateId,
                core::ptr::addr_of_mut!(rcArrow),
                core::ptr::null(),
            );
        } else {
            TOOLBAR_DrawSepDDArrow(
                infoPtr,
                core::ptr::addr_of_mut!(tbcd),
                core::ptr::addr_of_mut!(rcArrow),
                (*btnPtr).bDropDownPressed,
                dwItemCDFlag,
            );
        }
    }

    oldBkMode = SetBkMode(hdc, tbcd.nStringBkMode);
    if ((*infoPtr).dwExStyle & TBSTYLE_EX_MIXEDBUTTONS) == 0
        || ((*btnPtr).fsStyle as DWORD & BTNS_SHOWTEXT) != 0
    {
        TOOLBAR_DrawString(
            infoPtr,
            core::ptr::addr_of_mut!(rcText),
            lpText as LPCWSTR,
            btnPtr,
            core::ptr::addr_of_mut!(tbcd),
            dwItemCDFlag,
        );
    }
    SetBkMode(hdc, oldBkMode);

    TOOLBAR_DrawImage(
        infoPtr,
        btnPtr,
        rcBitmap.left,
        rcBitmap.top,
        core::ptr::addr_of_mut!(tbcd),
        dwItemCDFlag,
    );

    if hasDropDownArrow != 0 && drawSepDropDownArrow == 0 {
        if (tbcd.nmcd.uItemState & (CDIS_DISABLED | CDIS_INDETERMINATE)) != 0 {
            TOOLBAR_DrawArrow(
                hdc,
                rcArrow.left + 1,
                rcArrow.top
                    + 1
                    + (rcArrow.bottom - rcArrow.top - ARROW_HEIGHT) / 2,
                comctl32_color.clrBtnHighlight,
            );
            TOOLBAR_DrawArrow(
                hdc,
                rcArrow.left,
                rcArrow.top
                    + (rcArrow.bottom - rcArrow.top - ARROW_HEIGHT) / 2,
                comctl32_color.clr3dShadow,
            );
        } else {
            let mut clr: COLORREF = comctl32_color.clrBtnText;
            if !theme.is_null() {
                GetThemeColor(
                    theme,
                    (TP_BUTTON) as i32,
                    (TS_NORMAL) as i32,
                    (TMT_TEXTCOLOR as i32),
                    core::ptr::addr_of_mut!(clr),
                );
            }

            if (tbcd.nmcd.uItemState & (CDIS_SELECTED | CDIS_CHECKED)) != 0 {
                offset = if (dwItemCDFlag & TBCDRF_NOOFFSET) != 0 {
                    0
                } else {
                    1
                };
                TOOLBAR_DrawArrow(
                    hdc,
                    rcArrow.left + offset,
                    rcArrow.top
                        + offset
                        + (rcArrow.bottom - rcArrow.top - ARROW_HEIGHT) / 2,
                    clr,
                );
            } else {
                TOOLBAR_DrawArrow(
                    hdc,
                    rcArrow.left,
                    rcArrow.top
                        + (rcArrow.bottom - rcArrow.top - ARROW_HEIGHT) / 2,
                    clr,
                );
            }
        }
    }

    if (dwItemCustDraw & CDRF_NOTIFYPOSTPAINT) != 0 {
        tbcd.nmcd.dwDrawStage = CDDS_ITEMPOSTPAINT;
        TOOLBAR_SendNotify(
            core::ptr::addr_of_mut!(tbcd.nmcd.hdr),
            infoPtr,
            NM_CUSTOMDRAW,
        );
    }
}

unsafe extern "C" fn TOOLBAR_Refresh(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut hdc: HDC,
    mut ps: *const PAINTSTRUCT,
) {
    let mut btnPtr: *mut TBUTTON_INFO = ::core::ptr::null_mut::<TBUTTON_INFO>();
    let mut i: INT = 0;
    let mut rcTemp: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut rcClient: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut tbcd: NMTBCUSTOMDRAW = ::core::mem::zeroed::<NMTBCUSTOMDRAW>();
    let mut ntfret: DWORD = 0;
    let mut dwBaseCustDraw: DWORD = 0;

    /* the app has told us not to redraw the toolbar */
    if (*infoPtr).bDoRedraw == 0 {
        return;
    }

    /* if imagelist belongs to the app, it can be changed
       by the app after setting it */
    if TOOLBAR_GetImageList((*infoPtr).himlDef, (*infoPtr).cimlDef, 0 as INT)
        != (*infoPtr).himlInt
    {
        (*infoPtr).nNumBitmaps = 0 as INT;
        i = 0 as INT;
        while i < (*infoPtr).cimlDef {
            (*infoPtr).nNumBitmaps +=
                ImageList_GetImageCount((**(*infoPtr).himlDef.offset(i as isize)).himl);
            i += 1;
        }
    }

    TOOLBAR_DumpToolbar(infoPtr, line!() as INT);

    /* change the imagelist icon size if we manage the list and it is necessary */
    TOOLBAR_CheckImageListIconSize(infoPtr);

    /* Send initial notify */
    ZeroMemory!(
        &raw mut tbcd as *mut ::core::ffi::c_void,
        core::mem::size_of::<NMTBCUSTOMDRAW>() as size_t,
    );
    tbcd.nmcd.dwDrawStage = CDDS_PREPAINT as DWORD;
    tbcd.nmcd.hdc = hdc;
    tbcd.nmcd.rc = (*ps).rcPaint;
    ntfret = TOOLBAR_SendNotify(
        &raw mut tbcd.nmcd.hdr,
        infoPtr,
        NM_CUSTOMDRAW as UINT,
    ) as DWORD;
    dwBaseCustDraw = ntfret & 0xffff as DWORD;

    GetClientRect((*infoPtr).hwndSelf, &raw mut rcClient);

    /* redraw necessary buttons */
    btnPtr = (*infoPtr).buttons;
    i = 0 as INT;
    while i < (*infoPtr).nNumButtons {
        let mut bDraw: BOOL = 0;
        if !(RectVisible(hdc, &raw mut (*btnPtr).rect as LPCRECT) == 0) {
            if (*infoPtr).dwExStyle & TBSTYLE_EX_HIDECLIPPEDBUTTONS as DWORD != 0 {
                IntersectRect(
                    &raw mut rcTemp,
                    &raw mut rcClient as LPCRECT,
                    &raw mut (*btnPtr).rect as LPCRECT,
                );
                bDraw = EqualRect(
                    &raw mut rcTemp as LPCRECT,
                    &raw mut (*btnPtr).rect as LPCRECT,
                );
            } else {
                bDraw = TRUE as BOOL;
            }
            bDraw &= IntersectRect(
                &raw mut rcTemp,
                &raw const (*ps).rcPaint,
                &raw mut (*btnPtr).rect as LPCRECT,
            );
            bDraw = (if (*btnPtr).fsState as ::core::ffi::c_int & TBSTATE_HIDDEN as ::core::ffi::c_int != 0 {
                FALSE as ::core::ffi::c_int
            } else {
                bDraw as ::core::ffi::c_int
            }) as BOOL;
            if bDraw != 0 {
                TOOLBAR_DrawButton(infoPtr, btnPtr, hdc, dwBaseCustDraw);
            }
        }
        i += 1;
        btnPtr = btnPtr.offset(1);
    }

    /* draw insert mark if required */
    if (*infoPtr).tbim.iButton != -(1 as ::core::ffi::c_int) {
        let mut rcButton: RECT =
            (*(*infoPtr).buttons.offset((*infoPtr).tbim.iButton as isize)).rect;
        let mut rcInsertMark: RECT = tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        rcInsertMark.top = rcButton.top;
        rcInsertMark.bottom = rcButton.bottom;
        if (*infoPtr).tbim.dwFlags & TBIMHT_AFTER as DWORD != 0 {
            rcInsertMark.right = rcButton.right;
            rcInsertMark.left = rcInsertMark.right;
        } else {
            rcInsertMark.right =
                (rcButton.left as ::core::ffi::c_int - INSERTMARK_WIDTH as ::core::ffi::c_int)
                    as LONG;
            rcInsertMark.left = rcInsertMark.right;
        }
        COMCTL32_DrawInsertMark(
            hdc,
            &raw mut rcInsertMark,
            (*infoPtr).clrInsertMark,
            FALSE as BOOL,
        );
    }

    if dwBaseCustDraw & CDRF_NOTIFYPOSTPAINT as DWORD != 0 {
        ZeroMemory!(
            &raw mut tbcd as *mut ::core::ffi::c_void,
            core::mem::size_of::<NMTBCUSTOMDRAW>() as size_t,
        );
        tbcd.nmcd.dwDrawStage = CDDS_POSTPAINT as DWORD;
        tbcd.nmcd.hdc = hdc;
        tbcd.nmcd.rc = (*ps).rcPaint;
        TOOLBAR_SendNotify(
            &raw mut tbcd.nmcd.hdr,
            infoPtr,
            NM_CUSTOMDRAW as UINT,
        );
    }
}

unsafe extern "C" fn TOOLBAR_MeasureString(
    infoPtr: *const TOOLBAR_INFO,
    btnPtr: *const TBUTTON_INFO,
    hdc: HDC,
    lpSize: LPSIZE,
) {
    let mut myrect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    (*lpSize).cx = 0 as LONG;
    (*lpSize).cy = 0 as LONG;

    if (*infoPtr).nMaxTextRows > 0 as INT
        && !((*btnPtr).fsState as i32 & TBSTATE_HIDDEN as i32 != 0)
        && (!((*infoPtr).dwExStyle & TBSTYLE_EX_MIXEDBUTTONS as DWORD != 0)
            || ((*btnPtr).fsStyle as i32 & BTNS_SHOWTEXT as i32 != 0))
    {
        let lpText: LPWSTR = TOOLBAR_GetText(infoPtr, btnPtr);

        if !lpText.is_null() {
            /* first get size of all the text */
            GetTextExtentPoint32W(
                hdc,
                lpText as LPCWSTR,
                lstrlenW(lpText as LPCWSTR),
                lpSize,
            );

            /* feed above size into the rectangle for DrawText */
            SetRect(
                core::ptr::addr_of_mut!(myrect),
                0 as INT,
                0 as INT,
                (*lpSize).cx as INT,
                (*lpSize).cy as INT,
            );

            /* Use DrawText to get true size as drawn (less pesky "&") */
            DrawTextW(
                hdc,
                lpText as LPCWSTR,
                -(1 as INT),
                core::ptr::addr_of_mut!(myrect),
                (DT_VCENTER | DT_SINGLELINE
                    | DT_CALCRECT
                    | (if (*btnPtr).fsStyle as i32 & BTNS_NOPREFIX as i32 != 0 {
                        DT_NOPREFIX
                    } else {
                        0
                    })) as UINT,
            );

            /* feed back to caller  */
            (*lpSize).cx = myrect.right;
            (*lpSize).cy = myrect.bottom;
        }
    }

    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const i8,
            b"TOOLBAR_MeasureString\0" as *const u8 as *const i8,
            1400 as i32,
            b"string size %d x %d!\n\0" as *const u8 as *const i8,
            (*lpSize).cx,
            (*lpSize).cy,
        );
    }
}

unsafe extern "C" fn TOOLBAR_CalcStrings(infoPtr: *const TOOLBAR_INFO, lpSize: LPSIZE) {
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut();
    let mut i: INT = 0;
    let mut sz: SIZE = SIZE { cx: 0, cy: 0 };
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hOldFont: HFONT = core::ptr::null_mut();

    (*lpSize).cx = 0;
    (*lpSize).cy = 0;

    if (*infoPtr).nMaxTextRows == 0 {
        return;
    }

    hdc = GetDC((*infoPtr).hwndSelf);
    hOldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;

    if (*infoPtr).nNumButtons == 0 && (*infoPtr).nNumStrings > 0 {
        let mut tm: TEXTMETRICW = core::mem::zeroed();

        GetTextMetricsW(hdc, &raw mut tm);
        (*lpSize).cy = tm.tmHeight;
    }

    btnPtr = (*infoPtr).buttons;
    i = 0;
    while i < (*infoPtr).nNumButtons {
        if !TOOLBAR_GetText(infoPtr, btnPtr).is_null() {
            TOOLBAR_MeasureString(infoPtr, btnPtr, hdc, &raw mut sz);
            if sz.cx > (*lpSize).cx {
                (*lpSize).cx = sz.cx;
            }
            if sz.cy > (*lpSize).cy {
                (*lpSize).cy = sz.cy;
            }
        }
        i += 1;
        btnPtr = btnPtr.offset(1);
    }

    SelectObject(hdc, hOldFont as HGDIOBJ);
    ReleaseDC((*infoPtr).hwndSelf, hdc);

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_CalcStrings\0" as *const u8 as *const ::core::ffi::c_char,
            1450 as ::core::ffi::c_int,
            b"max string size %d x %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*lpSize).cx,
            (*lpSize).cy,
        );
    }
}

unsafe extern "C" fn TOOLBAR_WrapToolbar(infoPtr: *mut TOOLBAR_INFO) {
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut();
    let mut x: INT = 0;
    let mut cx: INT = 0;
    let mut i: INT = 0;
    let mut j: INT = 0;
    let mut width: INT = 0;
    let mut bButtonWrap: BOOL = 0;

    if ((*infoPtr).dwStyle & (TBSTYLE_WRAPABLE as DWORD)) == 0
        && ((*infoPtr).dwExStyle & (TBSTYLE_EX_VERTICAL as DWORD)) == 0
    {
        return;
    }

    if (*infoPtr).client_rect.right == (0 as INT) {
        return;
    }

    btnPtr = (*infoPtr).buttons;
    x = (*infoPtr).nIndent;
    width = ((*infoPtr).client_rect.right - (*infoPtr).client_rect.left) as INT;

    bButtonWrap = FALSE as BOOL;

    i = 0 as INT;
    while i < (*infoPtr).nNumButtons {
        (*btnPtr.offset(i as isize)).fsState =
            ((*btnPtr.offset(i as isize)).fsState as INT & !(TBSTATE_WRAP as INT)) as BYTE;

        if ((*btnPtr.offset(i as isize)).fsState as INT & (TBSTATE_HIDDEN as INT)) != 0 {
            i += 1;
            continue;
        }

        if (*btnPtr.offset(i as isize)).cx > (0 as INT) {
            cx = (*btnPtr.offset(i as isize)).cx;
        }
        /* horizontal separators are treated as buttons for width    */
        else if ((*btnPtr.offset(i as isize)).fsStyle as INT & (BTNS_SEP as INT)) != 0
            && ((*infoPtr).dwStyle & (CCS_VERT as DWORD)) == 0
        {
            cx = if (*btnPtr.offset(i as isize)).iBitmap > (0 as INT) {
                (*btnPtr.offset(i as isize)).iBitmap
            } else {
                SEPARATOR_WIDTH as INT
            };
        } else {
            cx = (*infoPtr).nButtonWidth;
        }

        if (*btnPtr.offset(i as isize)).cx == 0
            && button_has_ddarrow(infoPtr, btnPtr.offset(i as isize)) != 0
        {
            cx += DDARROW_WIDTH as INT;
        }

        /* Two or more adjacent separators form a separator group.   */
        /* The first separator in a group should be wrapped to the   */
        /* next row if the previous wrapping is on a button.	     */
        if bButtonWrap != 0
            && ((*btnPtr.offset(i as isize)).fsStyle as INT & (BTNS_SEP as INT)) != 0
            && (i + (1 as INT)) < (*infoPtr).nNumButtons
            && ((*btnPtr.offset((i + (1 as INT)) as isize)).fsStyle as INT
                & (BTNS_SEP as INT))
                != 0
        {
            (*btnPtr.offset(i as isize)).fsState =
                ((*btnPtr.offset(i as isize)).fsState as INT | (TBSTATE_WRAP as INT)) as BYTE;
            x = (*infoPtr).nIndent;
            i += 1;
            bButtonWrap = FALSE as BOOL;
            i += 1;
            continue;
        }

        /* The layout makes sure the bitmap is visible, but not the button. */
        /* Test added to also wrap after a button that starts a row but     */
        /* is bigger than the area.  - GA  8/01                             */
        if (x + cx - ((*infoPtr).nButtonWidth - (*infoPtr).nBitmapWidth) / (2 as INT)
            > width)
            || ((x == (*infoPtr).nIndent) && (cx > width))
        {
            let mut bFound: BOOL = FALSE as BOOL;

            /* 	If the current button is a separator and not hidden,  */
            /*	go to the next until it reaches a non separator.      */
            /*	Wrap the last separator if it is before a button.     */
            while ((((*btnPtr.offset(i as isize)).fsStyle as INT & (BTNS_SEP as INT)) != 0
                && ((*btnPtr.offset(i as isize)).fsStyle as INT & (BTNS_DROPDOWN as INT)) == 0)
                || ((*btnPtr.offset(i as isize)).fsState as INT & (TBSTATE_HIDDEN as INT)) != 0)
                && i < (*infoPtr).nNumButtons
            {
                i += 1;
                bFound = TRUE as BOOL;
            }

            if bFound != 0 && i < (*infoPtr).nNumButtons {
                i -= 1;
                (*btnPtr.offset(i as isize)).fsState =
                    ((*btnPtr.offset(i as isize)).fsState as INT | (TBSTATE_WRAP as INT)) as BYTE;
                x = (*infoPtr).nIndent;
                bButtonWrap = FALSE as BOOL;
                i += 1;
                continue;
            } else if i >= (*infoPtr).nNumButtons {
                break;
            }

            /* 	If the current button is not a separator, find the last  */
            /*	separator and wrap it.   				 */
            j = i - (1 as INT);
            while j >= (0 as INT)
                && ((*btnPtr.offset(j as isize)).fsState as INT & (TBSTATE_WRAP as INT)) == 0
            {
                if ((*btnPtr.offset(j as isize)).fsStyle as INT & (BTNS_SEP as INT)) != 0
                    && ((*btnPtr.offset(j as isize)).fsState as INT & (TBSTATE_HIDDEN as INT)) == 0
                {
                    bFound = TRUE as BOOL;
                    i = j;
                    x = (*infoPtr).nIndent;
                    (*btnPtr.offset(j as isize)).fsState =
                        ((*btnPtr.offset(j as isize)).fsState as INT | (TBSTATE_WRAP as INT)) as BYTE;
                    bButtonWrap = FALSE as BOOL;
                    break;
                }
                j -= 1;
            }

            /* 	If no separator available for wrapping, wrap one of 	*/
            /*  non-hidden previous button.  			     	*/
            if bFound == 0 {
                j = i - (1 as INT);
                while j >= (0 as INT)
                    && ((*btnPtr.offset(j as isize)).fsState as INT & (TBSTATE_WRAP as INT)) == 0
                {
                    if ((*btnPtr.offset(j as isize)).fsState as INT & (TBSTATE_HIDDEN as INT)) != 0 {
                        j -= 1;
                        continue;
                    }

                    bFound = TRUE as BOOL;
                    i = j;
                    x = (*infoPtr).nIndent;
                    (*btnPtr.offset(j as isize)).fsState =
                        ((*btnPtr.offset(j as isize)).fsState as INT | (TBSTATE_WRAP as INT)) as BYTE;
                    bButtonWrap = TRUE as BOOL;
                    break;
                }
            }

            /* If all above failed, wrap the current button. */
            if bFound == 0 {
                (*btnPtr.offset(i as isize)).fsState =
                    ((*btnPtr.offset(i as isize)).fsState as INT | (TBSTATE_WRAP as INT)) as BYTE;
                x = (*infoPtr).nIndent;
                if ((*btnPtr.offset(i as isize)).fsStyle as INT & (BTNS_SEP as INT)) != 0 {
                    bButtonWrap = FALSE as BOOL;
                } else {
                    bButtonWrap = TRUE as BOOL;
                }
            }
        } else {
            x += cx;
        }

        i += 1;
    }
}

unsafe extern "C" fn TOOLBAR_MeasureButton(
    infoPtr: *const TOOLBAR_INFO,
    sizeString: SIZE,
    bHasBitmap: BOOL,
    bValidImageList: BOOL,
) -> SIZE {
    let mut sizeButton: SIZE = SIZE { cx: 0, cy: 0 };
    if ((*infoPtr).dwStyle & TBSTYLE_LIST as DWORD) != 0 {
        /* set button height from bitmap / text height... */
        sizeButton.cy = max(
            if bHasBitmap != 0 {
                (*infoPtr).nBitmapHeight as LONG
            } else {
                0 as LONG
            },
            sizeString.cy,
        );

        /* ... add on the necessary padding */
        if bValidImageList != 0 {
            sizeButton.cy += (*infoPtr).szPadding.cy;
            if bHasBitmap == 0 {
                sizeButton.cy += LISTPAD_CY as LONG;
            }
        } else {
            sizeButton.cy += (*infoPtr).szPadding.cy;
        }

        /* calculate button width */
        sizeButton.cx = (2 * GetSystemMetrics((SM_CXEDGE) as i32) as INT
            + (*infoPtr).nBitmapWidth
            + (*infoPtr).iListGap) as LONG;
        if sizeString.cx > 0 as LONG {
            sizeButton.cx += sizeString.cx + (*infoPtr).szPadding.cx;
        }
    } else {
        if bHasBitmap != 0 {
            sizeButton.cy = (*infoPtr).nBitmapHeight as LONG + (*infoPtr).szPadding.cy;
            if sizeString.cy > 0 as LONG {
                sizeButton.cy += 1 as LONG + sizeString.cy;
            }
            sizeButton.cx = (*infoPtr).szPadding.cx
                + max(sizeString.cx, (*infoPtr).nBitmapWidth as LONG);
        } else {
            sizeButton.cy = sizeString.cy + (*infoPtr).szPadding.cy
                + NONLIST_NOTEXT_OFFSET as LONG;
            sizeButton.cx = (*infoPtr).szPadding.cx
                + max(
                    2 * GetSystemMetrics((SM_CXEDGE) as i32) as LONG + sizeString.cx,
                    (*infoPtr).nBitmapWidth as LONG,
                );
        }
    }

    sizeButton.cx += (*infoPtr).themeMargins.cxLeftWidth + (*infoPtr).themeMargins.cxRightWidth;
    sizeButton.cy += (*infoPtr).themeMargins.cyTopHeight + (*infoPtr).themeMargins.cyBottomHeight;

    return sizeButton;
}

unsafe extern "C" fn TOOLBAR_CalcToolbar(mut infoPtr: *mut TOOLBAR_INFO) {
    let mut sizeString: SIZE = SIZE { cx: 0, cy: 0 };
    let mut sizeButton: SIZE = SIZE { cx: 0, cy: 0 };
    let mut validImageList: BOOL = FALSE;

    TOOLBAR_CalcStrings(infoPtr, core::ptr::addr_of_mut!(sizeString));

    TOOLBAR_DumpToolbar(infoPtr, line!() as INT);

    if TOOLBAR_IsValidImageList(infoPtr, 0) != 0 {
        validImageList = TRUE;
    }
    sizeButton = TOOLBAR_MeasureButton(infoPtr, sizeString, TRUE, validImageList);
    (*infoPtr).nButtonWidth = sizeButton.cx;
    (*infoPtr).nButtonHeight = sizeButton.cy;
    (*infoPtr).iTopMargin = default_top_margin(infoPtr);

    if (*infoPtr).cxMin >= 0 && (*infoPtr).nButtonWidth < (*infoPtr).cxMin {
        (*infoPtr).nButtonWidth = (*infoPtr).cxMin;
    }
    if (*infoPtr).cxMax > 0 && (*infoPtr).nButtonWidth > (*infoPtr).cxMax {
        (*infoPtr).nButtonWidth = (*infoPtr).cxMax;
    }

    TOOLBAR_LayoutToolbar(infoPtr);
}

unsafe extern "C" fn TOOLBAR_LayoutToolbar(infoPtr: *mut TOOLBAR_INFO) {
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut();
    let mut sizeButton: SIZE = core::mem::zeroed();
    let mut i: INT = 0;
    let mut nRows: INT = 0;
    let mut nSepRows: INT = 0;
    let mut x: INT = 0;
    let mut y: INT = 0;
    let mut cx: INT = 0;
    let mut cy: INT = 0;
    let mut bWrap: BOOL = 0;
    let validImageList: BOOL = TOOLBAR_IsValidImageList(infoPtr, 0);

    TOOLBAR_WrapToolbar(infoPtr);

    x = (*infoPtr).nIndent;
    y = (*infoPtr).iTopMargin;
    cx = (*infoPtr).nButtonWidth;
    cy = (*infoPtr).nButtonHeight;

    nSepRows = 0;
    nRows = nSepRows;

    (*infoPtr).rcBound.top = y;
    (*infoPtr).rcBound.left = x;
    (*infoPtr).rcBound.bottom = y + cy;
    (*infoPtr).rcBound.right = x;

    btnPtr = (*infoPtr).buttons;

    i = 0;
    while i < (*infoPtr).nNumButtons {
        'body: {
            bWrap = FALSE;
            if (((*btnPtr).fsState as INT) & (TBSTATE_HIDDEN as INT)) != 0 {
                SetRectEmpty(core::ptr::addr_of_mut!((*btnPtr).rect));
                TOOLBAR_TooltipSetRect(infoPtr, btnPtr);
                break 'body;
            }

            cy = (*infoPtr).nButtonHeight;

            if (((*btnPtr).fsStyle as INT) & (BTNS_SEP as INT)) != 0 {
                if ((*infoPtr).dwStyle & (CCS_VERT)) != 0 {
                    cy = if (*btnPtr).iBitmap > 0 {
                        (*btnPtr).iBitmap
                    } else {
                        SEPARATOR_WIDTH
                    };
                    cx = if (*btnPtr).cx > 0 {
                        (*btnPtr).cx
                    } else {
                        (*infoPtr).nButtonWidth
                    };
                } else {
                    cx = if (*btnPtr).cx > 0 {
                        (*btnPtr).cx
                    } else if (*btnPtr).iBitmap > 0 {
                        (*btnPtr).iBitmap
                    } else {
                        SEPARATOR_WIDTH
                    };
                }
            } else {
                if (*btnPtr).cx != 0 {
                    cx = (*btnPtr).cx;
                } else if ((((*infoPtr).dwExStyle) & (TBSTYLE_EX_MIXEDBUTTONS)) != 0)
                    || ((((*btnPtr).fsStyle as INT) & (BTNS_AUTOSIZE as INT)) != 0)
                {
                    let mut sz: SIZE = core::mem::zeroed();
                    let mut hdc: HDC = core::ptr::null_mut();
                    let mut hOldFont: HFONT = core::ptr::null_mut();

                    hdc = GetDC((*infoPtr).hwndSelf);
                    hOldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;

                    TOOLBAR_MeasureString(infoPtr, btnPtr, hdc, core::ptr::addr_of_mut!(sz));

                    SelectObject(hdc, hOldFont as HGDIOBJ);
                    ReleaseDC((*infoPtr).hwndSelf, hdc);

                    sizeButton = TOOLBAR_MeasureButton(
                        infoPtr,
                        sz,
                        TOOLBAR_IsValidBitmapIndex(
                            infoPtr,
                            (*(*infoPtr).buttons.offset(i as isize)).iBitmap,
                        ),
                        validImageList,
                    );
                    cx = sizeButton.cx;
                } else {
                    cx = (*infoPtr).nButtonWidth;
                }

                if (*btnPtr).cx == 0 && button_has_ddarrow(infoPtr, btnPtr) != 0 {
                    cx += DDARROW_WIDTH;
                }
            }

            if (((*btnPtr).fsState as INT) & (TBSTATE_WRAP as INT)) != 0 {
                bWrap = TRUE;
            }

            SetRect(
                core::ptr::addr_of_mut!((*btnPtr).rect),
                x,
                y,
                x + cx,
                y + cy,
            );

            if (*infoPtr).rcBound.left > x {
                (*infoPtr).rcBound.left = x;
            }
            if (*infoPtr).rcBound.right < x + cx {
                (*infoPtr).rcBound.right = x + cx;
            }
            if (*infoPtr).rcBound.bottom < y + cy {
                (*infoPtr).rcBound.bottom = y + cy;
            }

            TOOLBAR_TooltipSetRect(infoPtr, btnPtr);

            (*btnPtr).nRow = nRows + nSepRows;

            if bWrap != 0 {
                if (((*btnPtr).fsStyle as INT) & (BTNS_SEP as INT)) == 0 {
                    y += cy + (*infoPtr).szSpacing.cy;
                } else {
                    if ((*infoPtr).dwStyle & (CCS_VERT)) == 0 {
                        y += cy
                            + (if (*btnPtr).cx > 0 {
                                (*btnPtr).cx
                            } else {
                                SEPARATOR_WIDTH
                            }) * 2
                                / 3;
                    } else {
                        y += cy + (*infoPtr).szSpacing.cy;
                    }
                    nSepRows += 1;
                }
                x = (*infoPtr).nIndent;

                if i != (*infoPtr).nNumButtons - 1 {
                    nRows += 1;
                }
            } else {
                x += cx + (*infoPtr).szSpacing.cx;
            }
        }
        i += 1;
        btnPtr = btnPtr.offset(1);
    }

    (*infoPtr).nRows = nRows + nSepRows + 1;
}

unsafe extern "C" fn TOOLBAR_InternalHitTest(
    infoPtr: *const TOOLBAR_INFO,
    lpPt: *const POINT,
    button: *mut BOOL,
) -> INT {
    let mut btnPtr: *mut TBUTTON_INFO;
    let mut i: INT;

    if !button.is_null() {
        *button = FALSE;
    }

    btnPtr = (*infoPtr).buttons;
    i = 0;
    while i < (*infoPtr).nNumButtons {
        'next: {
            if ((*btnPtr).fsState as INT & (TBSTATE_HIDDEN as i32)) != 0 {
                break 'next;
            }

            if ((*btnPtr).fsStyle as INT & (BTNS_SEP as i32)) != 0 {
                if PtInRect(core::ptr::addr_of_mut!((*btnPtr).rect) as LPCRECT, *lpPt) != 0 {
                    return -i;
                }
            } else {
                if PtInRect(core::ptr::addr_of_mut!((*btnPtr).rect) as LPCRECT, *lpPt) != 0 {
                    if !button.is_null() {
                        *button = TRUE;
                    }
                    return i;
                }
            }
        }
        i += 1;
        btnPtr = btnPtr.add(1);
    }

    return TOOLBAR_NOWHERE;
}

unsafe extern "C" fn TOOLBAR_InternalInsertButtonsT(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut iIndex: INT,
    mut nAddButtons: UINT,
    mut lpTbb: *const TBBUTTON,
    mut fUnicode: BOOL,
) -> BOOL {
    let mut nOldButtons: INT = 0;
    let mut nNewButtons: INT = 0;
    let mut iButton: INT = 0;
    let mut fHasString: BOOL = FALSE;
    if iIndex < 0 {
        iIndex = (*infoPtr).nNumButtons;
    }
    nOldButtons = (*infoPtr).nNumButtons;
    nNewButtons = (nOldButtons as UINT).wrapping_add(nAddButtons) as INT;
    (*infoPtr).buttons = ReAlloc(
        (*infoPtr).buttons as LPVOID,
        (::core::mem::size_of::<TBUTTON_INFO>() as usize).wrapping_mul(nNewButtons as usize)
            as DWORD,
    ) as *mut TBUTTON_INFO;
    memmove(
        (*infoPtr)
            .buttons
            .offset((iIndex as UINT).wrapping_add(nAddButtons) as isize)
            as *mut TBUTTON_INFO as *mut ::core::ffi::c_void,
        (*infoPtr).buttons.offset(iIndex as isize) as *mut TBUTTON_INFO
            as *const ::core::ffi::c_void,
        ((nOldButtons - iIndex) as usize)
            .wrapping_mul(::core::mem::size_of::<TBUTTON_INFO>() as usize),
    );
    (*infoPtr).nNumButtons =
        ((*infoPtr).nNumButtons as UINT).wrapping_add(nAddButtons) as INT;

    iButton = 0;
    while (iButton as UINT) < nAddButtons {
        let mut btnPtr: *mut TBUTTON_INFO =
            (*infoPtr).buttons.offset((iIndex + iButton) as isize) as *mut TBUTTON_INFO;
        let mut str: INT_PTR = 0;

        TOOLBAR_DumpTBButton(lpTbb.offset(iButton as isize), fUnicode);

        ZeroMemory!(
            btnPtr as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<TBUTTON_INFO>() as usize,
        );

        (*btnPtr).iBitmap = (*lpTbb.offset(iButton as isize)).iBitmap as INT;
        (*btnPtr).idCommand = (*lpTbb.offset(iButton as isize)).idCommand as INT;
        (*btnPtr).fsState = (*lpTbb.offset(iButton as isize)).fsState;
        (*btnPtr).fsStyle = (*lpTbb.offset(iButton as isize)).fsStyle;
        (*btnPtr).dwData = (*lpTbb.offset(iButton as isize)).dwData;

        if ((*btnPtr).fsStyle as ::core::ffi::c_int & (BTNS_SEP as i32)) != 0 {
            str = -1 as INT_PTR;
        } else {
            str = (*lpTbb.offset(iButton as isize)).iString;
        }
        set_string_index(btnPtr, str, fUnicode);
        fHasString |= TOOLBAR_ButtonHasString(btnPtr);

        TOOLBAR_TooltipAddTool(infoPtr, btnPtr);
        iButton += 1;
    }

    if (*infoPtr).nNumStrings > 0 || fHasString != 0 {
        TOOLBAR_CalcToolbar(infoPtr);
    } else {
        TOOLBAR_LayoutToolbar(infoPtr);
    }
    TOOLBAR_AutoSize(infoPtr);

    TOOLBAR_DumpToolbar(infoPtr, line!() as INT);
    InvalidateRect((*infoPtr).hwndSelf, ::core::ptr::null::<RECT>(), TRUE);
    return TRUE;
}

unsafe extern "C" fn TOOLBAR_GetButtonIndex(
    mut infoPtr: *const TOOLBAR_INFO,
    mut idCommand: INT,
    mut CommandIsIndex: BOOL,
) -> INT {
    let mut btnPtr: *mut TBUTTON_INFO = ::core::ptr::null_mut::<TBUTTON_INFO>();
    let mut i: INT = 0;
    if CommandIsIndex != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_GetButtonIndex\0" as *const u8 as *const ::core::ffi::c_char,
                2085 as ::core::ffi::c_int,
                b"command is really index command=%d\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                idCommand,
            );
        }
        if idCommand >= (*infoPtr).nNumButtons {
            return -(1 as INT);
        }
        return idCommand;
    }
    btnPtr = (*infoPtr).buttons;
    i = 0 as INT;
    while i < (*infoPtr).nNumButtons {
        if (*btnPtr).idCommand == idCommand {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/toolbar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TOOLBAR_GetButtonIndex\0" as *const u8
                        as *const ::core::ffi::c_char,
                    2092 as ::core::ffi::c_int,
                    b"command=%d index=%d\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    idCommand,
                    i,
                );
            }
            return i;
        }
        i += 1;
        btnPtr = btnPtr.offset(1);
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
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_GetButtonIndex\0" as *const u8 as *const ::core::ffi::c_char,
            2096 as ::core::ffi::c_int,
            b"no index found for command=%d\n\0" as *const u8
                as *const ::core::ffi::c_char,
            idCommand,
        );
    }
    return -(1 as INT);
}

unsafe extern "C" fn TOOLBAR_GetCheckedGroupButtonIndex(infoPtr: *const TOOLBAR_INFO, nIndex: INT) -> INT {
    let mut btnPtr: *mut TBUTTON_INFO = ::core::ptr::null_mut::<TBUTTON_INFO>();
    let mut nRunIndex: INT = 0;

    if nIndex < 0 || nIndex > (*infoPtr).nNumButtons {
        return -1;
    }

    /* check index button */
    btnPtr = (*infoPtr).buttons.offset(nIndex as isize) as *mut TBUTTON_INFO;
    if ((*btnPtr).fsStyle as INT & (BTNS_CHECKGROUP as i32)) == (BTNS_CHECKGROUP as i32) {
        if (*btnPtr).fsState as INT & (TBSTATE_CHECKED as i32) != 0 {
            return nIndex;
        }
    }

    /* check previous buttons */
    nRunIndex = nIndex - 1;
    while nRunIndex >= 0 {
        btnPtr = (*infoPtr).buttons.offset(nRunIndex as isize) as *mut TBUTTON_INFO;
        if ((*btnPtr).fsStyle as INT & (BTNS_GROUP as i32)) == (BTNS_GROUP as i32) {
            if (*btnPtr).fsState as INT & (TBSTATE_CHECKED as i32) != 0 {
                return nRunIndex;
            }
        } else {
            break;
        }
        nRunIndex -= 1;
    }

    /* check next buttons */
    nRunIndex = nIndex + 1;
    while nRunIndex < (*infoPtr).nNumButtons {
        btnPtr = (*infoPtr).buttons.offset(nRunIndex as isize) as *mut TBUTTON_INFO;
        if ((*btnPtr).fsStyle as INT & (BTNS_GROUP as i32)) == (BTNS_GROUP as i32) {
            if (*btnPtr).fsState as INT & (TBSTATE_CHECKED as i32) != 0 {
                return nRunIndex;
            }
        } else {
            break;
        }
        nRunIndex += 1;
    }

    return -1;
}

unsafe extern "C" fn TOOLBAR_RelayEvent(
    hwndTip: HWND,
    hwndMsg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) {
    let mut msg: MSG = core::mem::zeroed::<MSG>();

    msg.hwnd = hwndMsg;
    msg.message = uMsg;
    msg.wParam = wParam;
    msg.lParam = lParam;
    msg.time = GetMessageTime() as DWORD;
    msg.pt.x = (LOWORD!(GetMessagePos()) as i16) as LONG;
    msg.pt.y = (HIWORD!(GetMessagePos()) as i16) as LONG;

    SendMessageW(
        hwndTip,
        TTM_RELAYEVENT as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(msg) as LPARAM,
    );
}

unsafe extern "C" fn TOOLBAR_TooltipAddTool(infoPtr: *const TOOLBAR_INFO, button: *const TBUTTON_INFO) {
    if !(*infoPtr).hwndToolTip.is_null()
        && ((*button).fsStyle as ::core::ffi::c_int & (BTNS_SEP as i32)) == 0
    {
        let mut ti: TTTOOLINFOW = core::mem::zeroed();

        ZeroMemory!(
            &raw mut ti as *mut ::core::ffi::c_void,
            ::core::mem::size_of::<TTTOOLINFOW>() as size_t,
        );
        ti.cbSize = ::core::mem::size_of::<TTTOOLINFOW>() as UINT;
        ti.hwnd = (*infoPtr).hwndSelf;
        ti.uId = (*button).idCommand as UINT_PTR;
        ti.hinst = core::ptr::null_mut();
        ti.lpszText = LPSTR_TEXTCALLBACKW;
        /* ti.lParam = random value from the stack? */

        SendMessageW(
            (*infoPtr).hwndToolTip,
            TTM_ADDTOOLW,
            0 as WPARAM,
            &raw mut ti as LPARAM,
        );
    }
}

unsafe extern "C" fn TOOLBAR_TooltipDelTool(infoPtr: *const TOOLBAR_INFO, button: *const TBUTTON_INFO) {
    if !(*infoPtr).hwndToolTip.is_null() && ((*button).fsStyle as u32 & BTNS_SEP) == 0 {
        let mut ti: TTTOOLINFOW = core::mem::zeroed();

        ZeroMemory!(
            core::ptr::addr_of_mut!(ti) as *mut core::ffi::c_void,
            core::mem::size_of::<TTTOOLINFOW>(),
        );
        ti.cbSize = core::mem::size_of::<TTTOOLINFOW>() as UINT;
        ti.hwnd = (*infoPtr).hwndSelf;
        ti.uId = (*button).idCommand as UINT_PTR;

        SendMessageW(
            (*infoPtr).hwndToolTip,
            TTM_DELTOOLW,
            0,
            core::ptr::addr_of_mut!(ti) as LPARAM,
        );
    }
}

unsafe extern "C" fn TOOLBAR_TooltipSetRect(infoPtr: *const TOOLBAR_INFO, button: *const TBUTTON_INFO) {
    /* Set the toolTip only for non-hidden, non-separator button */
    if !(*infoPtr).hwndToolTip.is_null() && !(((*button).fsStyle as i32 & (BTNS_SEP as i32)) != 0) {
        let mut ti: TTTOOLINFOW = core::mem::zeroed();

        ZeroMemory!(&raw mut ti as *mut core::ffi::c_void, core::mem::size_of::<TTTOOLINFOW>());
        ti.cbSize = core::mem::size_of::<TTTOOLINFOW>() as UINT;
        ti.hwnd = (*infoPtr).hwndSelf;
        ti.uId = (*button).idCommand as UINT_PTR;
        ti.rect = (*button).rect;
        SendMessageW(
            (*infoPtr).hwndToolTip,
            TTM_NEWTOOLRECTW as UINT,
            0 as WPARAM,
            &raw mut ti as LPARAM,
        );
    }
}

unsafe extern "C" fn TOOLBAR_TooltipCreateControl(infoPtr: *mut TOOLBAR_INFO) {
    let mut i: i32;
    let mut nmttc: NMTOOLTIPSCREATED = core::mem::zeroed();

    (*infoPtr).hwndToolTip = CreateWindowExW(0, (TOOLTIPS_CLASSW).as_ptr(), core::ptr::null_mut(), WS_POPUP,
            CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT, CW_USEDEFAULT,
            (*infoPtr).hwndSelf, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut());

    if (*infoPtr).hwndToolTip.is_null() {
        return;
    }

    /* Send NM_TOOLTIPSCREATED notification */
    nmttc.hwndToolTips = (*infoPtr).hwndToolTip;
    TOOLBAR_SendNotify(core::ptr::addr_of_mut!(nmttc.hdr), infoPtr, NM_TOOLTIPSCREATED);

    i = 0;
    while i < (*infoPtr).nNumButtons {
        TOOLBAR_TooltipAddTool(infoPtr, (*infoPtr).buttons.offset(i as isize));
        TOOLBAR_TooltipSetRect(infoPtr, (*infoPtr).buttons.offset(i as isize));
        i += 1;
    }
}

unsafe extern "C" fn TOOLBAR_Cust_InsertAvailButton(hwnd: HWND, btnInfoNew: PCUSTOMBUTTON) {
    let mut i: ::core::ffi::c_int = 0;
    let mut count: ::core::ffi::c_int = 0;
    let mut btnInfo: PCUSTOMBUTTON = ::core::ptr::null_mut::<CUSTOMBUTTON>();
    let hwndAvail: HWND = GetDlgItem(hwnd, (IDC_AVAILBTN_LBOX as i32));

    count = SendMessageW(
        hwndAvail,
        LB_GETCOUNT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as ::core::ffi::c_int;

    /* position 0 is always separator */
    i = 1 as ::core::ffi::c_int;
    while i < count {
        btnInfo = SendMessageW(
            hwndAvail,
            LB_GETITEMDATA as UINT,
            i as WPARAM,
            0 as LPARAM,
        ) as PCUSTOMBUTTON;
        if (*btnInfoNew).btn.idCommand < (*btnInfo).btn.idCommand {
            i = SendMessageW(
                hwndAvail,
                LB_INSERTSTRING as UINT,
                i as WPARAM,
                0 as LPARAM,
            ) as ::core::ffi::c_int;
            SendMessageW(
                hwndAvail,
                LB_SETITEMDATA as UINT,
                i as WPARAM,
                btnInfoNew as LPARAM,
            );
            return;
        }
        i += 1;
    }
    /* id higher than all others add to end */
    i = SendMessageW(
        hwndAvail,
        LB_ADDSTRING as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as ::core::ffi::c_int;
    SendMessageW(
        hwndAvail,
        LB_SETITEMDATA as UINT,
        i as WPARAM,
        btnInfoNew as LPARAM,
    );
}

unsafe extern "C" fn TOOLBAR_Cust_MoveButton(
    custInfo: *const CUSTDLG_INFO,
    hwnd: HWND,
    nIndexFrom: INT,
    nIndexTo: INT,
) {
    let mut nmtb: NMTOOLBARW = tagNMTOOLBARW {
        hdr: tagNMHDR {
            hwndFrom: core::ptr::null_mut::<HWND__>(),
            idFrom: 0,
            code: 0,
        },
        iItem: 0,
        tbButton: unsafe { core::mem::zeroed::<_TBBUTTON>() },
        cchText: 0,
        pszText: core::ptr::null_mut::<WCHAR>(),
        rcButton: tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
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
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_Cust_MoveButton\0" as *const u8 as *const ::core::ffi::c_char,
            2271 as ::core::ffi::c_int,
            b"index from %d, index to %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            nIndexFrom,
            nIndexTo,
        );
    }
    if nIndexFrom == nIndexTo {
        return;
    }
    nmtb.iItem = nIndexFrom;
    if TOOLBAR_SendNotify(
        &raw mut nmtb.hdr,
        (*custInfo).tbInfo as *const TOOLBAR_INFO,
        (TBN_QUERYINSERT) as UINT,
    ) != 0
    {
        let mut btnInfo: PCUSTOMBUTTON = core::ptr::null_mut::<CUSTOMBUTTON>();
        let mut hdr: NMHDR = tagNMHDR {
            hwndFrom: core::ptr::null_mut::<HWND__>(),
            idFrom: 0,
            code: 0,
        };
        let hwndList: HWND = GetDlgItem(hwnd, (IDC_TOOLBARBTN_LBOX as i32));
        let count: INT = SendMessageW(
            hwndList,
            LB_GETCOUNT as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as INT;

        btnInfo = SendMessageW(
            hwndList,
            LB_GETITEMDATA as UINT,
            nIndexFrom as WPARAM,
            0 as LPARAM,
        ) as PCUSTOMBUTTON;

        SendMessageW(
            hwndList,
            LB_DELETESTRING as UINT,
            nIndexFrom as WPARAM,
            0 as LPARAM,
        );
        SendMessageW(
            hwndList,
            LB_INSERTSTRING as UINT,
            nIndexTo as WPARAM,
            0 as LPARAM,
        );
        SendMessageW(
            hwndList,
            LB_SETITEMDATA as UINT,
            nIndexTo as WPARAM,
            btnInfo as LPARAM,
        );
        SendMessageW(
            hwndList,
            LB_SETCURSEL as UINT,
            nIndexTo as WPARAM,
            0 as LPARAM,
        );

        if nIndexTo <= 0 {
            EnableWindow(
                GetDlgItem(hwnd, (IDC_MOVEUP_BTN as i32)),
                FALSE,
            );
        } else {
            EnableWindow(
                GetDlgItem(hwnd, (IDC_MOVEUP_BTN as i32)),
                TRUE,
            );
        }

        /* last item is always separator, so -2 instead of -1 */
        if nIndexTo >= (count - 2) {
            EnableWindow(
                GetDlgItem(hwnd, (IDC_MOVEDN_BTN as i32)),
                FALSE,
            );
        } else {
            EnableWindow(
                GetDlgItem(hwnd, (IDC_MOVEDN_BTN as i32)),
                TRUE,
            );
        }

        SendMessageW(
            (*custInfo).tbHwnd,
            TB_DELETEBUTTON as UINT,
            nIndexFrom as WPARAM,
            0 as LPARAM,
        );
        SendMessageW(
            (*custInfo).tbHwnd,
            TB_INSERTBUTTONW as UINT,
            nIndexTo as WPARAM,
            &raw mut (*btnInfo).btn as LPARAM,
        );

        TOOLBAR_SendNotify(
            &raw mut hdr,
            (*custInfo).tbInfo as *const TOOLBAR_INFO,
            (TBN_TOOLBARCHANGE) as UINT,
        );
    }
}

unsafe extern "C" fn TOOLBAR_Cust_AddButton(custInfo: *const CUSTDLG_INFO, hwnd: HWND, nIndexAvail: INT, nIndexTo: INT) {
    let mut nmtb: NMTOOLBARW = core::mem::zeroed();

    /* MSDN states that iItem is the index of the button, rather than the
     * command ID as used by every other NMTOOLBAR notification */
    nmtb.iItem = nIndexAvail;
    if TOOLBAR_SendNotify(core::ptr::addr_of_mut!(nmtb.hdr), (*custInfo).tbInfo, (TBN_QUERYINSERT) as UINT) != 0 {
        let mut btnInfo: PCUSTOMBUTTON = core::ptr::null_mut();
        let mut hdr: NMHDR = core::mem::zeroed();
        let hwndList: HWND = GetDlgItem(hwnd, IDC_TOOLBARBTN_LBOX as INT);
        let hwndAvail: HWND = GetDlgItem(hwnd, IDC_AVAILBTN_LBOX as INT);
        let count: INT = SendMessageW(hwndAvail, LB_GETCOUNT as UINT, 0 as WPARAM, 0 as LPARAM) as INT;

        btnInfo = SendMessageW(hwndAvail, LB_GETITEMDATA as UINT, nIndexAvail as WPARAM, 0 as LPARAM) as PCUSTOMBUTTON;

        if nIndexAvail != 0 {
            /* remove from 'available buttons' list */
            SendMessageW(hwndAvail, LB_DELETESTRING as UINT, nIndexAvail as WPARAM, 0 as LPARAM);
            if nIndexAvail == count - 1 {
                SendMessageW(hwndAvail, LB_SETCURSEL as UINT, (nIndexAvail - 1) as WPARAM, 0 as LPARAM);
            } else {
                SendMessageW(hwndAvail, LB_SETCURSEL as UINT, nIndexAvail as WPARAM, 0 as LPARAM);
            }
        } else {
            let mut btnNew: PCUSTOMBUTTON;

            /* duplicate 'separator' button */
            btnNew = Alloc(core::mem::size_of::<CUSTOMBUTTON>() as DWORD) as PCUSTOMBUTTON;
            core::ptr::write(btnNew, core::ptr::read(btnInfo));
            btnInfo = btnNew;
        }

        /* insert into 'toolbar button' list */
        SendMessageW(hwndList, LB_INSERTSTRING as UINT, nIndexTo as WPARAM, 0 as LPARAM);
        SendMessageW(hwndList, LB_SETITEMDATA as UINT, nIndexTo as WPARAM, btnInfo as LPARAM);

        SendMessageW((*custInfo).tbHwnd, TB_INSERTBUTTONW as UINT, nIndexTo as WPARAM, core::ptr::addr_of_mut!((*btnInfo).btn) as LPARAM);

        TOOLBAR_SendNotify(core::ptr::addr_of_mut!(hdr), (*custInfo).tbInfo, (TBN_TOOLBARCHANGE) as UINT);
    }
}

unsafe extern "C" fn TOOLBAR_Cust_RemoveButton(custInfo: *const CUSTDLG_INFO, hwnd: HWND, index: INT) {
    let mut btnInfo: PCUSTOMBUTTON = core::ptr::null_mut();
    let mut hwndList: HWND = GetDlgItem(hwnd, (IDC_TOOLBARBTN_LBOX as i32));

    btnInfo = SendMessageW(
        hwndList,
        LB_GETITEMDATA as UINT,
        index as WPARAM,
        0 as LPARAM,
    ) as PCUSTOMBUTTON;

    /* send TBN_QUERYDELETE notification */
    if TOOLBAR_IsButtonRemovable(
        (*custInfo).tbInfo as *const TOOLBAR_INFO,
        index as core::ffi::c_int,
        btnInfo as *const CUSTOMBUTTON,
    ) != 0
    {
        let mut hdr: NMHDR = core::mem::zeroed();

        SendMessageW(
            hwndList,
            LB_DELETESTRING as UINT,
            index as WPARAM,
            0 as LPARAM,
        );
        SendMessageW(
            hwndList,
            LB_SETCURSEL as UINT,
            index as WPARAM,
            0 as LPARAM,
        );

        SendMessageW(
            (*custInfo).tbHwnd,
            TB_DELETEBUTTON as UINT,
            index as WPARAM,
            0 as LPARAM,
        );

        /* insert into 'available button' list */
        if ((*btnInfo).btn.fsStyle as INT & (BTNS_SEP as i32)) == 0 {
            TOOLBAR_Cust_InsertAvailButton(hwnd, btnInfo);
        } else {
            Free(btnInfo as LPVOID);
        }

        TOOLBAR_SendNotify(
            &raw mut hdr,
            (*custInfo).tbInfo as *const TOOLBAR_INFO,
            TBN_TOOLBARCHANGE as UINT,
        );
    }
}

unsafe extern "C" fn TOOLBAR_Cust_ToolbarDragListNotification(
    custInfo: *const CUSTDLG_INFO,
    hwnd: HWND,
    pDLI: *const DRAGLISTINFO,
) -> LRESULT {
    let hwndList: HWND = GetDlgItem(hwnd, IDC_TOOLBARBTN_LBOX as INT);
    match (*pDLI).uNotification {
        DL_BEGINDRAG => {
            let nCurrentItem: INT = LBItemFromPt(hwndList, (*pDLI).ptCursor, TRUE as BOOL) as INT;
            let nCount: INT = SendMessageW(hwndList, LB_GETCOUNT as UINT, 0 as WPARAM, 0 as LPARAM) as INT;
            /* no dragging for last item (separator) */
            if nCurrentItem >= nCount - 1 {
                return FALSE as LRESULT;
            }
            return TRUE as LRESULT;
        }
        DL_DRAGGING => {
            let nCurrentItem: INT = LBItemFromPt(hwndList, (*pDLI).ptCursor, TRUE as BOOL) as INT;
            let nCount: INT = SendMessageW(hwndList, LB_GETCOUNT as UINT, 0 as WPARAM, 0 as LPARAM) as INT;
            /* no dragging past last item (separator) */
            if (nCurrentItem >= 0) && (nCurrentItem < nCount - 1) {
                DrawInsert(hwnd, hwndList, nCurrentItem);
                /* FIXME: native uses "move button" cursor */
                return DL_COPYCURSOR as LRESULT;
            }

            /* not over toolbar buttons list */
            if nCurrentItem < 0 {
                let mut ptWindow: POINT = (*pDLI).ptCursor;
                let hwndListAvail: HWND = GetDlgItem(hwnd, IDC_AVAILBTN_LBOX as INT);
                MapWindowPoints(core::ptr::null_mut(), hwnd, core::ptr::addr_of_mut!(ptWindow), 1 as UINT);
                /* over available buttons list? */
                if ChildWindowFromPoint(hwnd, ptWindow) == hwndListAvail {
                    /* FIXME: native uses "move button" cursor */
                    return DL_COPYCURSOR as LRESULT;
                }
            }
            /* clear drag arrow */
            DrawInsert(hwnd, hwndList, -1);
            return DL_STOPCURSOR as LRESULT;
        }
        DL_DROPPED => {
            let nIndexTo: INT = LBItemFromPt(hwndList, (*pDLI).ptCursor, TRUE as BOOL) as INT;
            let nIndexFrom: INT = SendMessageW(hwndList, LB_GETCURSEL as UINT, 0 as WPARAM, 0 as LPARAM) as INT;
            let nCount: INT = SendMessageW(hwndList, LB_GETCOUNT as UINT, 0 as WPARAM, 0 as LPARAM) as INT;
            if (nIndexTo >= 0) && (nIndexTo < nCount - 1) {
                /* clear drag arrow */
                DrawInsert(hwnd, hwndList, -1);
                /* move item */
                TOOLBAR_Cust_MoveButton(custInfo, hwnd, nIndexFrom, nIndexTo);
            }
            /* not over toolbar buttons list */
            if nIndexTo < 0 {
                let mut ptWindow: POINT = (*pDLI).ptCursor;
                let hwndListAvail: HWND = GetDlgItem(hwnd, IDC_AVAILBTN_LBOX as INT);
                MapWindowPoints(core::ptr::null_mut(), hwnd, core::ptr::addr_of_mut!(ptWindow), 1 as UINT);
                /* over available buttons list? */
                if ChildWindowFromPoint(hwnd, ptWindow) == hwndListAvail {
                    TOOLBAR_Cust_RemoveButton(custInfo, hwnd, nIndexFrom);
                }
            }
        }
        DL_CANCELDRAG => {
            /* Clear drag arrow */
            DrawInsert(hwnd, hwndList, -1);
        }
        _ => {}
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_Cust_AvailDragListNotification(custInfo: *const CUSTDLG_INFO, hwnd: HWND, pDLI: *const DRAGLISTINFO) -> LRESULT {
    let mut hwndList: HWND = GetDlgItem(hwnd, (IDC_TOOLBARBTN_LBOX as i32));
    match (*pDLI).uNotification {
        DL_BEGINDRAG => {
            return TRUE as LRESULT;
        }
        DL_DRAGGING => {
            let mut nCurrentItem: INT = LBItemFromPt(hwndList, (*pDLI).ptCursor, TRUE) as INT;
            let mut nCount: INT = SendMessageW(hwndList, LB_GETCOUNT, 0, 0) as INT;
            /* no dragging past last item (separator) */
            if (nCurrentItem >= 0) && (nCurrentItem < nCount) {
                DrawInsert(hwnd, hwndList, nCurrentItem);
                /* FIXME: native uses "move button" cursor */
                return DL_COPYCURSOR as LRESULT;
            }

            /* not over toolbar buttons list */
            if nCurrentItem < 0 {
                let mut ptWindow: POINT = (*pDLI).ptCursor;
                let mut hwndListAvail: HWND = GetDlgItem(hwnd, (IDC_AVAILBTN_LBOX as i32));
                MapWindowPoints(core::ptr::null_mut(), hwnd, core::ptr::addr_of_mut!(ptWindow), 1);
                /* over available buttons list? */
                if ChildWindowFromPoint(hwnd, ptWindow) == hwndListAvail {
                    /* FIXME: native uses "move button" cursor */
                    return DL_COPYCURSOR as LRESULT;
                }
            }
            /* clear drag arrow */
            DrawInsert(hwnd, hwndList, -1);
            return DL_STOPCURSOR as LRESULT;
        }
        DL_DROPPED => {
            let mut nIndexTo: INT = LBItemFromPt(hwndList, (*pDLI).ptCursor, TRUE) as INT;
            let mut nCount: INT = SendMessageW(hwndList, LB_GETCOUNT, 0, 0) as INT;
            let mut nIndexFrom: INT = SendDlgItemMessageW(hwnd, (IDC_AVAILBTN_LBOX as i32), LB_GETCURSEL, 0, 0) as INT;
            if (nIndexTo >= 0) && (nIndexTo < nCount) {
                /* clear drag arrow */
                DrawInsert(hwnd, hwndList, -1);
                /* add item */
                TOOLBAR_Cust_AddButton(custInfo, hwnd, nIndexFrom, nIndexTo);
            }
            /* fall through to the clear drag arrow of the next case */
            /* Clear drag arrow */
            DrawInsert(hwnd, hwndList, -1);
        }
        DL_CANCELDRAG => {
            /* Clear drag arrow */
            DrawInsert(hwnd, hwndList, -1);
        }
        _ => {}
    }
    return 0 as LRESULT;
}

unsafe extern "system" fn TOOLBAR_CustomizeDialogProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut custInfo: PCUSTDLG_INFO = GetWindowLongPtrW(hwnd, DWLP_USER) as PCUSTDLG_INFO;
    let mut btnInfo: PCUSTOMBUTTON = core::ptr::null_mut();
    let mut nmtb: NMTOOLBARA = core::mem::zeroed();
    let mut infoPtr: *mut TOOLBAR_INFO = if !custInfo.is_null() {
        (*custInfo).tbInfo as *mut TOOLBAR_INFO
    } else {
        core::ptr::null_mut()
    };

    match uMsg {
        WM_INITDIALOG => {
            custInfo = lParam as PCUSTDLG_INFO;
            SetWindowLongPtrW(hwnd, DWLP_USER, custInfo as LONG_PTR);

            if !custInfo.is_null() {
                let mut Buffer: [WCHAR; 256] = core::mem::zeroed();
                let mut i: i32 = 0;
                let mut index: i32 = 0;
                let mut nmtbic: NMTBINITCUSTOMIZE = core::mem::zeroed();

                infoPtr = (*custInfo).tbInfo as *mut TOOLBAR_INFO;

                /* send TBN_QUERYINSERT notification */
                nmtb.iItem = (*(*custInfo).tbInfo).nNumButtons;

                if TOOLBAR_SendNotify(core::ptr::addr_of_mut!(nmtb.hdr), infoPtr, (TBN_QUERYINSERT) as UINT) == 0 {
                    return (FALSE) as INT_PTR;
                }

                nmtbic.hwndDialog = hwnd;
                /* Send TBN_INITCUSTOMIZE notification */
                if TOOLBAR_SendNotify(core::ptr::addr_of_mut!(nmtbic.hdr), infoPtr, (TBN_INITCUSTOMIZE) as UINT) ==
                    (TBNRF_HIDEHELP as i32)
                {
                    ShowWindow(GetDlgItem(hwnd, (IDC_HELP_BTN as i32)), (SW_HIDE) as i32);
                }

                /* add items to 'toolbar buttons' list and check if removable */
                i = 0;
                while i < (*(*custInfo).tbInfo).nNumButtons {
                    btnInfo = Alloc(core::mem::size_of::<CUSTOMBUTTON>() as DWORD) as PCUSTOMBUTTON;
                    memset(
                        core::ptr::addr_of_mut!((*btnInfo).btn) as *mut core::ffi::c_void,
                        0,
                        core::mem::size_of::<TBBUTTON>() as usize,
                    );
                    (*btnInfo).btn.fsStyle = BTNS_SEP as BYTE;
                    (*btnInfo).bVirtual = FALSE;
                    LoadStringW(
                        COMCTL32_hModule as HINSTANCE,
                        IDS_SEPARATOR,
                        core::ptr::addr_of_mut!((*btnInfo).text) as LPWSTR,
                        64,
                    );

                    /* send TBN_QUERYDELETE notification */
                    (*btnInfo).bRemovable = TOOLBAR_IsButtonRemovable(infoPtr, i, btnInfo as *const CUSTOMBUTTON);

                    index = SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_ADDSTRING, 0, 0) as i32;
                    SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_SETITEMDATA, index as WPARAM, btnInfo as LPARAM);
                    i += 1;
                }

                SendDlgItemMessageW(
                    hwnd,
                    (IDC_TOOLBARBTN_LBOX as i32),
                    LB_SETITEMHEIGHT,
                    0,
                    ((*infoPtr).nBitmapHeight + 8) as LPARAM,
                );

                /* insert separator button into 'available buttons' list */
                btnInfo = Alloc(core::mem::size_of::<CUSTOMBUTTON>() as DWORD) as PCUSTOMBUTTON;
                memset(
                    core::ptr::addr_of_mut!((*btnInfo).btn) as *mut core::ffi::c_void,
                    0,
                    core::mem::size_of::<TBBUTTON>() as usize,
                );
                (*btnInfo).btn.fsStyle = BTNS_SEP as BYTE;
                (*btnInfo).bVirtual = FALSE;
                (*btnInfo).bRemovable = TRUE;
                LoadStringW(
                    COMCTL32_hModule as HINSTANCE,
                    IDS_SEPARATOR,
                    core::ptr::addr_of_mut!((*btnInfo).text) as LPWSTR,
                    64,
                );
                index = SendDlgItemMessageW(hwnd, (IDC_AVAILBTN_LBOX as i32), LB_ADDSTRING, 0, btnInfo as LPARAM) as i32;
                SendDlgItemMessageW(hwnd, (IDC_AVAILBTN_LBOX as i32), LB_SETITEMDATA, index as WPARAM, btnInfo as LPARAM);

                /* insert all buttons into dsa */
                i = 0;
                loop {
                    /* send TBN_GETBUTTONINFO notification */
                    let mut nmtb: NMTOOLBARW = core::mem::zeroed();
                    nmtb.iItem = i;
                    nmtb.pszText = core::ptr::addr_of_mut!(Buffer) as *mut WCHAR as LPWSTR;
                    nmtb.cchText = 256;

                    /* Clear previous button's text */
                    ZeroMemory!(nmtb.pszText as *mut core::ffi::c_void, nmtb.cchText as usize * core::mem::size_of::<WCHAR>());

                    if TOOLBAR_GetButtonInfo(infoPtr, core::ptr::addr_of_mut!(nmtb)) == 0 {
                        break;
                    }

                    /* insert button into the appropriate list */
                    index = TOOLBAR_GetButtonIndex(
                        (*custInfo).tbInfo as *const TOOLBAR_INFO,
                        nmtb.tbButton.idCommand as INT,
                        FALSE,
                    );
                    if index == -1 {
                        btnInfo = Alloc(core::mem::size_of::<CUSTOMBUTTON>() as DWORD) as PCUSTOMBUTTON;
                        (*btnInfo).bVirtual = FALSE;
                        (*btnInfo).bRemovable = TRUE;
                    } else {
                        btnInfo = SendDlgItemMessageW(
                            hwnd,
                            (IDC_TOOLBARBTN_LBOX as i32),
                            LB_GETITEMDATA,
                            index as WPARAM,
                            0,
                        ) as PCUSTOMBUTTON;
                    }

                    (*btnInfo).btn = nmtb.tbButton;
                    if (nmtb.tbButton.fsStyle as i32 & (BTNS_SEP as i32)) == 0 {
                        if *nmtb.pszText != 0 {
                            lstrcpyW(
                                core::ptr::addr_of_mut!((*btnInfo).text) as LPWSTR,
                                nmtb.pszText as LPCWSTR,
                            );
                        } else if nmtb.tbButton.iString >= 0 &&
                            nmtb.tbButton.iString < (*infoPtr).nNumStrings as INT_PTR
                        {
                            lstrcpyW(
                                core::ptr::addr_of_mut!((*btnInfo).text) as LPWSTR,
                                *(*infoPtr).strings.offset(nmtb.tbButton.iString as isize) as LPCWSTR,
                            );
                        }
                    }

                    if index == -1 {
                        TOOLBAR_Cust_InsertAvailButton(hwnd, btnInfo);
                    }
                    i += 1;
                }

                SendDlgItemMessageW(
                    hwnd,
                    (IDC_AVAILBTN_LBOX as i32),
                    LB_SETITEMHEIGHT,
                    0,
                    ((*infoPtr).nBitmapHeight + 8) as LPARAM,
                );

                /* select first item in the 'available' list */
                SendDlgItemMessageW(hwnd, (IDC_AVAILBTN_LBOX as i32), LB_SETCURSEL, 0, 0);

                /* append 'virtual' separator button to the 'toolbar buttons' list */
                btnInfo = Alloc(core::mem::size_of::<CUSTOMBUTTON>() as DWORD) as PCUSTOMBUTTON;
                memset(
                    core::ptr::addr_of_mut!((*btnInfo).btn) as *mut core::ffi::c_void,
                    0,
                    core::mem::size_of::<TBBUTTON>() as usize,
                );
                (*btnInfo).btn.fsStyle = BTNS_SEP as BYTE;
                (*btnInfo).bVirtual = TRUE;
                (*btnInfo).bRemovable = FALSE;
                LoadStringW(
                    COMCTL32_hModule as HINSTANCE,
                    IDS_SEPARATOR,
                    core::ptr::addr_of_mut!((*btnInfo).text) as LPWSTR,
                    64,
                );
                index = SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_ADDSTRING, 0, btnInfo as LPARAM) as i32;
                SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_SETITEMDATA, index as WPARAM, btnInfo as LPARAM);

                /* select last item in the 'toolbar' list */
                SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_SETCURSEL, index as WPARAM, 0);
                SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_SETTOPINDEX, index as WPARAM, 0);

                MakeDragList(GetDlgItem(hwnd, (IDC_TOOLBARBTN_LBOX as i32)));
                MakeDragList(GetDlgItem(hwnd, (IDC_AVAILBTN_LBOX as i32)));

                /* set focus and disable buttons */
                PostMessageW(hwnd, WM_USER, 0, 0);
            }
            return (TRUE) as INT_PTR;
        }

        WM_USER => {
            EnableWindow(GetDlgItem(hwnd, (IDC_MOVEUP_BTN as i32)), FALSE);
            EnableWindow(GetDlgItem(hwnd, (IDC_MOVEDN_BTN as i32)), FALSE);
            EnableWindow(GetDlgItem(hwnd, (IDC_REMOVE_BTN as i32)), FALSE);
            SetFocus(GetDlgItem(hwnd, (IDC_TOOLBARBTN_LBOX as i32)));
            return (TRUE) as INT_PTR;
        }

        WM_CLOSE => {
            EndDialog(hwnd, (FALSE) as INT_PTR);
            return (TRUE) as INT_PTR;
        }

        WM_COMMAND => {
            'sw: {
                match (LOWORD!(wParam) as i32) as u32 {
                    IDC_TOOLBARBTN_LBOX => {
                        if (HIWORD!(wParam)) as u32 == LBN_SELCHANGE {
                            let mut btnInfo: PCUSTOMBUTTON = core::ptr::null_mut();
                            let mut nmtb: NMTOOLBARA = core::mem::zeroed();
                            let mut count: i32;
                            let mut index: i32;

                            count = SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_GETCOUNT, 0, 0) as i32;
                            index = SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_GETCURSEL, 0, 0) as i32;

                            /* send TBN_QUERYINSERT notification */
                            nmtb.iItem = index;
                            TOOLBAR_SendNotify(core::ptr::addr_of_mut!(nmtb.hdr), infoPtr, (TBN_QUERYINSERT) as UINT);

                            /* get list box item */
                            btnInfo = SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_GETITEMDATA, index as WPARAM, 0) as PCUSTOMBUTTON;

                            if index == count - 1 {
                                /* last item (virtual separator) */
                                EnableWindow(GetDlgItem(hwnd, (IDC_MOVEUP_BTN as i32)), FALSE);
                                EnableWindow(GetDlgItem(hwnd, (IDC_MOVEDN_BTN as i32)), FALSE);
                            } else if index == count - 2 {
                                /* second last item (last non-virtual item) */
                                EnableWindow(GetDlgItem(hwnd, (IDC_MOVEUP_BTN as i32)), TRUE);
                                EnableWindow(GetDlgItem(hwnd, (IDC_MOVEDN_BTN as i32)), FALSE);
                            } else if index == 0 {
                                /* first item */
                                EnableWindow(GetDlgItem(hwnd, (IDC_MOVEUP_BTN as i32)), FALSE);
                                EnableWindow(GetDlgItem(hwnd, (IDC_MOVEDN_BTN as i32)), TRUE);
                            } else {
                                EnableWindow(GetDlgItem(hwnd, (IDC_MOVEUP_BTN as i32)), TRUE);
                                EnableWindow(GetDlgItem(hwnd, (IDC_MOVEDN_BTN as i32)), TRUE);
                            }

                            EnableWindow(GetDlgItem(hwnd, (IDC_REMOVE_BTN as i32)), (*btnInfo).bRemovable);
                        }
                    }

                    IDC_MOVEUP_BTN => {
                        let index = SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_GETCURSEL, 0, 0) as i32;
                        TOOLBAR_Cust_MoveButton(custInfo as *const CUSTDLG_INFO, hwnd, index, index - 1);
                    }

                    IDC_MOVEDN_BTN => { /* move down */
                        let index = SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_GETCURSEL, 0, 0) as i32;
                        TOOLBAR_Cust_MoveButton(custInfo as *const CUSTDLG_INFO, hwnd, index, index + 1);
                    }

                    IDC_REMOVE_BTN => { /* remove button */
                        let index = SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_GETCURSEL, 0, 0) as i32;

                        if LB_ERR == index {
                            break 'sw;
                        }

                        TOOLBAR_Cust_RemoveButton(custInfo as *const CUSTDLG_INFO, hwnd, index);
                    }
                    IDC_HELP_BTN => {
                        TOOLBAR_SendNotify(core::ptr::addr_of_mut!(nmtb.hdr), infoPtr, (TBN_CUSTHELP) as UINT);
                    }
                    IDC_RESET_BTN => {
                        TOOLBAR_SendNotify(core::ptr::addr_of_mut!(nmtb.hdr), infoPtr, (TBN_RESET) as UINT);
                    }

                    IDOK => { /* Add button */
                        let mut index: i32;
                        let mut indexto: i32;

                        index = SendDlgItemMessageW(hwnd, (IDC_AVAILBTN_LBOX as i32), LB_GETCURSEL, 0, 0) as i32;
                        indexto = SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_GETCURSEL, 0, 0) as i32;

                        TOOLBAR_Cust_AddButton(custInfo as *const CUSTDLG_INFO, hwnd, index, indexto);
                    }

                    IDCANCEL => {
                        EndDialog(hwnd, (FALSE) as INT_PTR);
                    }

                    _ => {}
                }
            }
            return (TRUE) as INT_PTR;
        }

        WM_DESTROY => {
            let mut count: i32;
            let mut i: i32;

            /* delete items from 'toolbar buttons' listbox*/
            count = SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_GETCOUNT, 0, 0) as i32;
            i = 0;
            while i < count {
                btnInfo = SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_GETITEMDATA, i as WPARAM, 0) as PCUSTOMBUTTON;
                Free((btnInfo).cast());
                SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_SETITEMDATA, 0, 0);
                i += 1;
            }
            SendDlgItemMessageW(hwnd, (IDC_TOOLBARBTN_LBOX as i32), LB_RESETCONTENT, 0, 0);

            /* delete items from 'available buttons' listbox*/
            count = SendDlgItemMessageW(hwnd, (IDC_AVAILBTN_LBOX as i32), LB_GETCOUNT, 0, 0) as i32;
            i = 0;
            while i < count {
                btnInfo = SendDlgItemMessageW(hwnd, (IDC_AVAILBTN_LBOX as i32), LB_GETITEMDATA, i as WPARAM, 0) as PCUSTOMBUTTON;
                Free((btnInfo).cast());
                SendDlgItemMessageW(hwnd, (IDC_AVAILBTN_LBOX as i32), LB_SETITEMDATA, i as WPARAM, 0);
                i += 1;
            }
            SendDlgItemMessageW(hwnd, (IDC_AVAILBTN_LBOX as i32), LB_RESETCONTENT, 0, 0);
            return (TRUE) as INT_PTR;
        }

        WM_DRAWITEM => {
            if wParam == (IDC_AVAILBTN_LBOX as WPARAM) || wParam == (IDC_TOOLBARBTN_LBOX as WPARAM) {
                let mut lpdis: LPDRAWITEMSTRUCT = lParam as LPDRAWITEMSTRUCT;
                let mut rcButton: RECT = core::mem::zeroed();
                let mut rcText: RECT = core::mem::zeroed();
                let mut hPen: HPEN = core::ptr::null_mut();
                let mut hOldPen: HPEN = core::ptr::null_mut();
                let mut hOldBrush: HBRUSH = core::ptr::null_mut();
                let mut oldText: COLORREF = 0 as COLORREF;
                let mut oldBk: COLORREF = 0 as COLORREF;

                /* get item data */
                btnInfo = SendDlgItemMessageW(hwnd, wParam as i32, LB_GETITEMDATA, (*lpdis).itemID as WPARAM, 0) as PCUSTOMBUTTON;
                if btnInfo.is_null() {
                    return (TRUE) as INT_PTR;
                }

                /* set colors and select objects */
                oldBk = SetBkColor(
                    (*lpdis).hDC,
                    if ((*lpdis).itemState & ODS_FOCUS as UINT) != 0 {
                        comctl32_color.clrHighlight
                    } else {
                        comctl32_color.clrWindow
                    },
                );
                if (*btnInfo).bVirtual != 0 {
                    oldText = SetTextColor((*lpdis).hDC, comctl32_color.clrGrayText);
                } else {
                    oldText = SetTextColor(
                        (*lpdis).hDC,
                        if ((*lpdis).itemState & ODS_FOCUS as UINT) != 0 {
                            comctl32_color.clrHighlightText
                        } else {
                            comctl32_color.clrWindowText
                        },
                    );
                }
                hPen = CreatePen(
                    (PS_SOLID as i32),
                    1,
                    if ((*lpdis).itemState & ODS_SELECTED as UINT) != 0 {
                        comctl32_color.clrHighlight
                    } else {
                        comctl32_color.clrWindow
                    },
                );
                hOldPen = SelectObject((*lpdis).hDC, (hPen).cast()) as HPEN;
                hOldBrush = SelectObject(
                    (*lpdis).hDC,
                    GetSysColorBrush(
                        if ((*lpdis).itemState & ODS_FOCUS as UINT) != 0 {
                            (COLOR_HIGHLIGHT) as i32
                        } else {
                            (COLOR_WINDOW) as i32
                        },
                    ) as HGDIOBJ,
                ) as HBRUSH;

                /* fill background rectangle */
                Rectangle(
                    (*lpdis).hDC,
                    (*lpdis).rcItem.left,
                    (*lpdis).rcItem.top,
                    (*lpdis).rcItem.right,
                    (*lpdis).rcItem.bottom,
                );

                /* calculate button and text rectangles */
                rcButton = (*lpdis).rcItem;
                InflateRect(core::ptr::addr_of_mut!(rcButton), -1, -1);
                rcText = rcButton;
                rcButton.right = rcButton.left + (*(*custInfo).tbInfo).nBitmapWidth + 6;
                rcText.left = rcButton.right + 2;

                /* draw focus rectangle */
                if ((*lpdis).itemState & ODS_FOCUS as UINT) != 0 {
                    DrawFocusRect((*lpdis).hDC, core::ptr::addr_of_mut!((*lpdis).rcItem) as LPCRECT);
                }

                /* draw button */
                if ((*infoPtr).dwStyle & TBSTYLE_FLAT as DWORD) == 0 {
                    DrawEdge(
                        (*lpdis).hDC,
                        core::ptr::addr_of_mut!(rcButton),
                        EDGE_RAISED,
                        BF_RECT | BF_MIDDLE | BF_SOFT,
                    );
                }

                /* draw image and text */
                if ((*btnInfo).btn.fsStyle as i32 & (BTNS_SEP as i32)) == 0 {
                    let mut himl: HIMAGELIST = GETDEFIMAGELIST!(infoPtr, GETHIMLID!(infoPtr, (*btnInfo).btn.iBitmap));
                    ImageList_Draw(
                        himl,
                        (GETIBITMAP!(infoPtr, ((*btnInfo).btn.iBitmap) as u16)) as i32,
                        (*lpdis).hDC,
                        rcButton.left + 3,
                        rcButton.top + 3,
                        ILD_NORMAL,
                    );
                }
                DrawTextW(
                    (*lpdis).hDC,
                    core::ptr::addr_of_mut!((*btnInfo).text) as LPCWSTR,
                    -1,
                    core::ptr::addr_of_mut!(rcText),
                    DT_LEFT | DT_VCENTER | DT_SINGLELINE,
                );

                /* delete objects and reset colors */
                SelectObject((*lpdis).hDC, (hOldBrush).cast());
                SelectObject((*lpdis).hDC, (hOldPen).cast());
                SetBkColor((*lpdis).hDC, oldBk);
                SetTextColor((*lpdis).hDC, oldText);
                DeleteObject((hPen).cast());
                return (TRUE) as INT_PTR;
            }
            return (FALSE) as INT_PTR;
        }

        WM_MEASUREITEM => {
            if wParam == (IDC_AVAILBTN_LBOX as WPARAM) || wParam == (IDC_TOOLBARBTN_LBOX as WPARAM) {
                let mut lpmis: *mut MEASUREITEMSTRUCT = lParam as *mut MEASUREITEMSTRUCT;

                (*lpmis).itemHeight = (15 + 8) as UINT; /* default height */

                return (TRUE) as INT_PTR;
            }
            return (FALSE) as INT_PTR;
        }

        _ => {
            if uDragListMessage != 0 && uMsg == uDragListMessage {
                if wParam == (IDC_TOOLBARBTN_LBOX as WPARAM) {
                    let mut res: LRESULT = TOOLBAR_Cust_ToolbarDragListNotification(
                        custInfo as *const CUSTDLG_INFO,
                        hwnd,
                        lParam as *mut DRAGLISTINFO,
                    );
                    SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, res as LONG_PTR);
                    return (TRUE) as INT_PTR;
                } else if wParam == (IDC_AVAILBTN_LBOX as WPARAM) {
                    let mut res_0: LRESULT = TOOLBAR_Cust_AvailDragListNotification(
                        custInfo as *const CUSTDLG_INFO,
                        hwnd,
                        lParam as *mut DRAGLISTINFO,
                    );
                    SetWindowLongPtrW(hwnd, DWLP_MSGRESULT, res_0 as LONG_PTR);
                    return (TRUE) as INT_PTR;
                }
            }
            return (FALSE) as INT_PTR;
        }
    }
}

unsafe extern "C" fn TOOLBAR_AddBitmapToImageList(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut himlDef: HIMAGELIST,
    mut bitmap: *const TBITMAP_INFO,
) -> BOOL {
    let mut hbmLoad: HBITMAP = core::ptr::null_mut::<HBITMAP__>();
    let mut nCountBefore: INT = ImageList_GetImageCount(himlDef) as INT;
    let mut nCountAfter: INT = 0;
    let mut cxIcon: INT = 0;
    let mut cyIcon: INT = 0;
    let mut nAdded: INT = 0;
    let mut nIndex: INT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_AddBitmapToImageList\0" as *const u8 as *const ::core::ffi::c_char,
            2921 as ::core::ffi::c_int,
            b"adding hInst=%p nID=%d nButtons=%d\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*bitmap).hInst,
            (*bitmap).nID,
            (*bitmap).nButtons,
        );
    }
    if (*bitmap).hInst.is_null() {
        hbmLoad = CopyImage(
            ULongToHandle!((*bitmap).nID as ::core::ffi::c_long) as HANDLE,
            IMAGE_BITMAP as UINT,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as UINT,
        ) as HBITMAP;
    } else if (*bitmap).hInst == COMCTL32_hModule {
        hbmLoad = LoadImageW(
            (*bitmap).hInst,
            MAKEINTRESOURCEW!((*bitmap).nID as WORD) as LPCWSTR,
            IMAGE_BITMAP as UINT,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            LR_CREATEDIBSECTION as UINT,
        ) as HBITMAP;
    } else {
        hbmLoad = CreateMappedBitmap(
            (*bitmap).hInst,
            (*bitmap).nID as INT_PTR,
            0 as UINT,
            core::ptr::null_mut::<_COLORMAP>(),
            0 as ::core::ffi::c_int,
        );
    }
    ImageList_GetIconSize(
        himlDef,
        core::ptr::addr_of_mut!(cxIcon),
        core::ptr::addr_of_mut!(cyIcon),
    );
    if (*bitmap).hInst != COMCTL32_hModule {
        COMCTL32_EnsureBitmapSize(
            core::ptr::addr_of_mut!(hbmLoad),
            cxIcon as ::core::ffi::c_int * (*bitmap).nButtons as ::core::ffi::c_int,
            cyIcon as ::core::ffi::c_int,
            comctl32_color.clrBtnFace,
        );
    }
    nIndex = ImageList_AddMasked(himlDef, hbmLoad, comctl32_color.clrBtnFace) as INT;
    DeleteObject(hbmLoad as HGDIOBJ);
    if nIndex == -(1 as INT) {
        return FALSE;
    }
    nCountAfter = ImageList_GetImageCount(himlDef) as INT;
    nAdded = nCountAfter - nCountBefore;
    if (*bitmap).nButtons == 0 as UINT {
        ImageList_SetImageCount(
            himlDef,
            (nCountBefore + 1 as INT) as UINT,
        );
    } else if nAdded > (*bitmap).nButtons as INT {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_AddBitmapToImageList\0" as *const u8
                    as *const ::core::ffi::c_char,
                2948 as ::core::ffi::c_int,
                b"Added more images than wParam: Previous image number %i added %i while wParam %i. Images in list %i\n\0"
                    as *const u8 as *const ::core::ffi::c_char,
                nCountBefore,
                nAdded,
                (*bitmap).nButtons,
                nCountAfter,
            );
        }
    }
    (*infoPtr).nNumBitmaps += nAdded;
    return TRUE;
}

unsafe extern "C" fn TOOLBAR_CheckImageListIconSize(mut infoPtr: *mut TOOLBAR_INFO) {
    let mut himlDef: HIMAGELIST = core::ptr::null_mut::<_IMAGELIST>();
    let mut himlNew: HIMAGELIST = core::ptr::null_mut::<_IMAGELIST>();
    let mut cx: INT = 0;
    let mut cy: INT = 0;
    let mut i: INT = 0;
    himlDef = TOOLBAR_GetImageList((*infoPtr).himlDef, (*infoPtr).cimlDef, 0 as INT);
    if himlDef.is_null() || himlDef != (*infoPtr).himlInt {
        return;
    }
    if ImageList_GetIconSize(himlDef, &raw mut cx, &raw mut cy) == 0 {
        return;
    }
    if cx == (*infoPtr).nBitmapWidth && cy == (*infoPtr).nBitmapHeight {
        return;
    }
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"TOOLBAR_CheckImageListIconSize\0" as *const u8
                as *const core::ffi::c_char,
            2972 as core::ffi::c_int,
            b"Update icon size: %dx%d -> %dx%d\n\0" as *const u8
                as *const core::ffi::c_char,
            cx,
            cy,
            (*infoPtr).nBitmapWidth,
            (*infoPtr).nBitmapHeight,
        );
    }
    himlNew = ImageList_Create(
        (*infoPtr).nBitmapWidth as core::ffi::c_int,
        (*infoPtr).nBitmapHeight as core::ffi::c_int,
        (ILC_COLOR32 | ILC_MASK) as UINT,
        8 as core::ffi::c_int,
        2 as core::ffi::c_int,
    );
    i = 0 as INT;
    while i < (*infoPtr).nNumBitmapInfos {
        TOOLBAR_AddBitmapToImageList(
            infoPtr,
            himlNew,
            (*infoPtr).bitmaps.offset(i as isize) as *mut TBITMAP_INFO,
        );
        i += 1;
    }
    TOOLBAR_InsertImageList(
        &raw mut (*infoPtr).himlDef,
        &raw mut (*infoPtr).cimlDef,
        himlNew,
        0 as INT,
    );
    (*infoPtr).himlInt = himlNew;

    (*infoPtr).nNumBitmaps -= ImageList_GetImageCount(himlDef);
    ImageList_Destroy(himlDef);
}

unsafe extern "C" fn TOOLBAR_AddBitmap(
    infoPtr: *mut TOOLBAR_INFO,
    count: INT,
    lpAddBmp: *const TBADDBITMAP,
) -> LRESULT {
    let mut info: TBITMAP_INFO = TBITMAP_INFO {
        nButtons: 0,
        hInst: core::ptr::null_mut(),
        nID: 0,
    };
    let mut iSumButtons: INT;
    let mut i: INT;
    let mut himlDef: HIMAGELIST;

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"TOOLBAR_AddBitmap\0" as *const u8 as *const core::ffi::c_char,
            2996 as core::ffi::c_int,
            b"hwnd=%p count=%d lpAddBmp=%p\n\0" as *const u8 as *const core::ffi::c_char,
            (*infoPtr).hwndSelf,
            count,
            lpAddBmp,
        );
    }

    if lpAddBmp.is_null() {
        return -(1) as LRESULT;
    }

    if (*lpAddBmp).hInst == HINST_COMMCTRL {
        info.hInst = COMCTL32_hModule as HINSTANCE;
        match (*lpAddBmp).nID as u32 {
            IDB_STD_SMALL_COLOR | 2 => {
                info.nButtons = 15 as UINT;
                info.nID = IDB_STD_SMALL as UINT;
            }
            IDB_STD_LARGE_COLOR | 3 => {
                info.nButtons = 15 as UINT;
                info.nID = IDB_STD_LARGE as UINT;
            }
            IDB_VIEW_SMALL_COLOR | 6 => {
                info.nButtons = 12 as UINT;
                info.nID = IDB_VIEW_SMALL as UINT;
            }
            IDB_VIEW_LARGE_COLOR | 7 => {
                info.nButtons = 12 as UINT;
                info.nID = IDB_VIEW_LARGE as UINT;
            }
            IDB_HIST_SMALL_COLOR => {
                info.nButtons = 5 as UINT;
                info.nID = IDB_HIST_SMALL as UINT;
            }
            IDB_HIST_LARGE_COLOR => {
                info.nButtons = 5 as UINT;
                info.nID = IDB_HIST_LARGE as UINT;
            }
            _ => {
                if (*__wine_dbch___default).flags as core::ffi::c_int
                    & (1 as core::ffi::c_int) << __WINE_DBCL_WARN as core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/comctl32/toolbar.c\0"
                            as *const u8 as *const core::ffi::c_char,
                        b"TOOLBAR_AddBitmap\0" as *const u8 as *const core::ffi::c_char,
                        3034 as core::ffi::c_int,
                        b"unknown bitmap id, %ld\n\0" as *const u8 as *const core::ffi::c_char,
                        (*lpAddBmp).nID,
                    );
                }
                return -(1) as LRESULT;
            }
        }

        if (*lpAddBmp).nID & 1 != 0 {
            SendMessageW(
                (*infoPtr).hwndSelf,
                TB_SETBITMAPSIZE as UINT,
                0 as WPARAM,
                MAKELPARAM!(24, 24) as LPARAM,
            );
        } else {
            SendMessageW(
                (*infoPtr).hwndSelf,
                TB_SETBITMAPSIZE as UINT,
                0 as WPARAM,
                MAKELPARAM!(16, 16) as LPARAM,
            );
        }

        TOOLBAR_CalcToolbar(infoPtr);
    } else {
        info.nButtons = count as UINT;
        info.hInst = (*lpAddBmp).hInst;
        info.nID = (*lpAddBmp).nID as UINT;
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"TOOLBAR_AddBitmap\0" as *const u8 as *const core::ffi::c_char,
                3059 as core::ffi::c_int,
                b"adding %d bitmaps\n\0" as *const u8 as *const core::ffi::c_char,
                info.nButtons,
            );
        }
    }

    /* check if the bitmap is already loaded and compute iSumButtons */
    iSumButtons = 0 as INT;
    i = 0 as INT;
    while i < (*infoPtr).nNumBitmapInfos {
        if (*(*infoPtr).bitmaps.offset(i as isize)).hInst == info.hInst
            && (*(*infoPtr).bitmaps.offset(i as isize)).nID == info.nID
        {
            return iSumButtons as LRESULT;
        }
        iSumButtons = (iSumButtons as UINT)
            .wrapping_add((*(*infoPtr).bitmaps.offset(i as isize)).nButtons)
            as INT;
        i += 1;
    }

    if (*infoPtr).cimlDef == 0 {
        /* create new default image list */
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
            != 0
        {
            let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_3,
                __dbch_3,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"TOOLBAR_AddBitmap\0" as *const u8 as *const core::ffi::c_char,
                3074 as core::ffi::c_int,
                b"creating default image list\n\0" as *const u8 as *const core::ffi::c_char,
            );
        }
        himlDef = ImageList_Create(
            (*infoPtr).nBitmapWidth,
            (*infoPtr).nBitmapHeight,
            ILC_COLOR32 | ILC_MASK,
            (info.nButtons) as INT,
            2,
        );
        TOOLBAR_InsertImageList(
            core::ptr::addr_of_mut!((*infoPtr).himlDef),
            core::ptr::addr_of_mut!((*infoPtr).cimlDef),
            himlDef,
            0,
        );
        (*infoPtr).himlInt = himlDef;
    } else {
        himlDef = GETDEFIMAGELIST!(infoPtr, 0);
    }

    if himlDef.is_null() {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_WARN as core::ffi::c_int
            != 0
        {
            let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_4: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_4,
                __dbch_4,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"TOOLBAR_AddBitmap\0" as *const u8 as *const core::ffi::c_char,
                3086 as core::ffi::c_int,
                b"No default image list available\n\0" as *const u8 as *const core::ffi::c_char,
            );
        }
        return -(1) as LRESULT;
    }

    if TOOLBAR_AddBitmapToImageList(infoPtr, himlDef, core::ptr::addr_of_mut!(info)) == 0 {
        return -(1) as LRESULT;
    }

    /* grow the bitmap info array */
    (*infoPtr).bitmaps = ReAlloc(
        (*infoPtr).bitmaps as LPVOID,
        (((*infoPtr).nNumBitmapInfos + 1) as usize)
            .wrapping_mul(core::mem::size_of::<TBITMAP_INFO>()) as DWORD,
    ) as *mut TBITMAP_INFO;
    *(*infoPtr).bitmaps.offset((*infoPtr).nNumBitmapInfos as isize) = info;
    (*infoPtr).nNumBitmapInfos += 1;

    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>() as _, TRUE);
    return iSumButtons as LRESULT;
}

unsafe extern "C" fn TOOLBAR_AddButtonsT(infoPtr: *mut TOOLBAR_INFO, nAddButtons: INT, lpTbb: *const TBBUTTON, fUnicode: BOOL) -> LRESULT {
    return TOOLBAR_InternalInsertButtonsT(infoPtr, -1, nAddButtons as UINT, lpTbb, fUnicode) as LRESULT;
}
unsafe extern "C" fn TOOLBAR_AddStringW(infoPtr: *mut TOOLBAR_INFO, hInstance: HINSTANCE, lParam: LPARAM) -> LRESULT {
    let fFirstString: BOOL = ((*infoPtr).nNumStrings == 0) as BOOL;
    let nIndex: INT = (*infoPtr).nNumStrings;

    if IS_INTRESOURCE!(lParam as DWORD) {
        let mut szString: [WCHAR; MAX_RESOURCE_STRING_LENGTH] = [0 as WCHAR; MAX_RESOURCE_STRING_LENGTH];
        let mut delimiter: WCHAR = 0 as WCHAR;
        let mut next_delim: *mut WCHAR = core::ptr::null_mut();
        let mut hrsrc: HRSRC = core::ptr::null_mut();
        let mut p: *mut WCHAR = core::ptr::null_mut();
        let mut len: INT = 0;

        if hInstance.is_null() {
            return -(1) as LRESULT;
        }

        hrsrc = FindResourceW(
            hInstance as HMODULE,
            MAKEINTRESOURCEW!((LOWORD!(lParam as DWORD) >> 4) + 1) as LPCWSTR,
            (RT_STRING as usize) as LPCWSTR,
        );
        if hrsrc.is_null() {
            return -(1) as LRESULT;
        }

        len = LoadStringW(
            hInstance,
            lParam as UINT,
            (&raw mut szString) as LPWSTR,
            (MAX_RESOURCE_STRING_LENGTH) as INT,
        ) as INT;

        if len == 0 || len == 1 {
            return nIndex as LRESULT;
        }

        delimiter = *(&raw mut szString as *mut WCHAR);
        p = (&raw mut szString as *mut WCHAR).offset(1 as isize);

        loop {
            next_delim = wcschr(p, delimiter as wchar_t) as *mut WCHAR;
            if next_delim.is_null() {
                break;
            }
            *next_delim = 0 as WCHAR;
            if next_delim.offset(1 as isize)
                >= (&raw mut szString as *mut WCHAR).offset(len as isize)
            {
                /* this may happen if delimiter == '\0' or if the last char is a
                 * delimiter (then it is ignored like the native does) */
                break;
            }

            (*infoPtr).strings = ReAlloc(
                (*infoPtr).strings as LPVOID,
                (core::mem::size_of::<LPWSTR>() as usize).wrapping_mul(
                    ((*infoPtr).nNumStrings + 1) as usize,
                ) as DWORD,
            ) as *mut LPWSTR;
            Str_SetPtrW(
                (*infoPtr).strings.offset((*infoPtr).nNumStrings as isize) as *mut LPWSTR,
                p as LPCWSTR,
            );
            (*infoPtr).nNumStrings += 1;

            p = next_delim.offset(1 as isize);
        }
    } else {
        let mut p: LPWSTR = lParam as LPWSTR;
        let mut len: INT = 0;

        if p.is_null() {
            return -(1) as LRESULT;
        }

        while *p != 0 {
            len = lstrlenW(p as LPCWSTR) as INT;

            (*infoPtr).strings = ReAlloc(
                (*infoPtr).strings as LPVOID,
                (core::mem::size_of::<LPWSTR>() as usize).wrapping_mul(
                    ((*infoPtr).nNumStrings + 1) as usize,
                ) as DWORD,
            ) as *mut LPWSTR;
            Str_SetPtrW(
                (*infoPtr).strings.offset((*infoPtr).nNumStrings as isize) as *mut LPWSTR,
                p as LPCWSTR,
            );
            (*infoPtr).nNumStrings += 1;

            p = p.offset((len + 1) as isize);
        }
    }

    if fFirstString != 0 {
        TOOLBAR_CalcToolbar(infoPtr);
    }
    return nIndex as LRESULT;
}

unsafe extern "C" fn TOOLBAR_AddStringA(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut hInstance: HINSTANCE,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut fFirstString: BOOL = ((*infoPtr).nNumStrings == 0) as BOOL;
    let mut p: LPSTR = core::ptr::null_mut::<CHAR>();
    let mut nIndex: INT = 0;
    let mut len: INT = 0;

    if IS_INTRESOURCE!(lParam) != false {
        /* load from resources */
        return TOOLBAR_AddStringW(infoPtr, hInstance, lParam);
    }

    p = lParam as LPSTR;
    if p.is_null() {
        return -(1) as LRESULT;
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
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_AddStringA\0" as *const u8 as *const ::core::ffi::c_char,
            3211 as ::core::ffi::c_int,
            b"adding string(s) from array\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }

    nIndex = (*infoPtr).nNumStrings;
    while *p != 0 {
        len = strlen(p as *const ::core::ffi::c_char) as INT;

        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_AddStringA\0" as *const u8 as *const ::core::ffi::c_char,
                3215 as ::core::ffi::c_int,
                b"len=%d \"%s\"\n\0" as *const u8 as *const ::core::ffi::c_char,
                len,
                p,
            );
        }

        (*infoPtr).strings = ReAlloc(
            (*infoPtr).strings as LPVOID,
            (core::mem::size_of::<LPWSTR>()).wrapping_mul(
                ((*infoPtr).nNumStrings + 1) as usize,
            ) as DWORD,
        ) as *mut LPWSTR;
        Str_SetPtrAtoW(
            (*infoPtr).strings.offset((*infoPtr).nNumStrings as isize) as *mut LPWSTR,
            p as LPCSTR,
        );
        (*infoPtr).nNumStrings += 1;

        p = p.offset((len + 1) as isize);
    }

    if fFirstString != 0 {
        TOOLBAR_CalcToolbar(infoPtr);
    }
    return nIndex as LRESULT;
}

unsafe extern "C" fn TOOLBAR_AutoSize(infoPtr: *mut TOOLBAR_INFO) -> LRESULT {
    if ((*infoPtr).dwStyle & TBSTYLE_WRAPABLE) != 0 || ((*infoPtr).dwExStyle & TBSTYLE_EX_VERTICAL) != 0 {
        TOOLBAR_LayoutToolbar(infoPtr);
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut::<RECT>(), TRUE);
    }

    if ((*infoPtr).dwStyle & CCS_NORESIZE) == 0 {
        let mut window_rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        let mut parent_rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
        let mut uPosFlags: UINT = (SWP_NOZORDER | SWP_NOACTIVATE) as UINT;
        let mut parent: HWND = core::ptr::null_mut();
        let mut x: INT = 0;
        let mut y: INT = 0;
        let mut cx: INT = 0;
        let mut cy: INT = 0;

        parent = GetParent((*infoPtr).hwndSelf);

        if parent.is_null() || (*infoPtr).bDoRedraw == 0 {
            return 0;
        }

        GetClientRect(parent, core::ptr::addr_of_mut!(parent_rect));

        x = parent_rect.left;
        y = parent_rect.top;

        cy = TOP_BORDER + (*infoPtr).nRows * (*infoPtr).nButtonHeight + BOTTOM_BORDER;
        cx = parent_rect.right - parent_rect.left;

        if ((*infoPtr).dwStyle & CCS_BOTTOM) == CCS_NOMOVEY {
            GetWindowRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(window_rect));
            MapWindowPoints(
                core::ptr::null_mut(),
                parent,
                core::ptr::addr_of_mut!(window_rect) as *mut POINT,
                2,
            );
            y = window_rect.top;
        }
        if ((*infoPtr).dwStyle & CCS_BOTTOM) == CCS_BOTTOM {
            GetWindowRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(window_rect));
            y = parent_rect.bottom - (window_rect.bottom - window_rect.top);
        }

        if ((*infoPtr).dwStyle & CCS_NOPARENTALIGN) != 0 {
            uPosFlags |= SWP_NOMOVE as UINT;
        }

        if ((*infoPtr).dwStyle & CCS_NODIVIDER) == 0 {
            cy += GetSystemMetrics((SM_CYEDGE) as i32);
        }

        if ((*infoPtr).dwStyle & WS_BORDER) != 0 {
            cx += 2 * GetSystemMetrics((SM_CXBORDER) as i32);
            cy += 2 * GetSystemMetrics((SM_CYBORDER) as i32);
        }

        SetWindowPos((*infoPtr).hwndSelf, core::ptr::null_mut::<HWND__>(), x, y, cx, cy, uPosFlags);
    }

    if ((*infoPtr).dwStyle & TBSTYLE_WRAPABLE) != 0 || ((*infoPtr).dwExStyle & TBSTYLE_EX_VERTICAL) != 0 {
        TOOLBAR_LayoutToolbar(infoPtr);
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut::<RECT>(), TRUE);
    }

    return 0;
}

unsafe extern "C" fn TOOLBAR_ButtonCount(infoPtr: *const TOOLBAR_INFO) -> LRESULT {
    ((*infoPtr).nNumButtons) as _
}

unsafe extern "C" fn TOOLBAR_ButtonStructSize(mut infoPtr: *mut TOOLBAR_INFO, mut Size: DWORD) -> LRESULT {
    (*infoPtr).dwStructSize = Size;
    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_ChangeBitmap(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut Id: INT,
    mut Index: INT,
) -> LRESULT {
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut::<TBUTTON_INFO>();
    let mut nIndex: INT = 0;
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"TOOLBAR_ChangeBitmap\0" as *const u8 as *const core::ffi::c_char,
            3323 as core::ffi::c_int,
            b"button %d, iBitmap now %d\n\0" as *const u8 as *const core::ffi::c_char,
            Id,
            Index,
        );
    }
    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);
    if nIndex == -1 {
        return FALSE as LRESULT;
    }
    btnPtr = (*infoPtr).buttons.offset(nIndex as isize) as *mut TBUTTON_INFO;
    (*btnPtr).iBitmap = Index;
    InvalidateRect(
        (*infoPtr).hwndSelf,
        core::ptr::addr_of_mut!((*btnPtr).rect) as LPCRECT,
        TRUE,
    );
    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_CheckButton(infoPtr: *mut TOOLBAR_INFO, Id: INT, lParam: LPARAM) -> LRESULT {
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut();
    let mut nIndex: INT;
    let mut nOldIndex: INT = -1;
    let mut bChecked: BOOL = FALSE;

    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);

    if nIndex == -1 {
        return FALSE as LRESULT;
    }

    btnPtr = (*infoPtr).buttons.offset(nIndex as isize);

    bChecked = ((((*btnPtr).fsState as INT) & (TBSTATE_CHECKED as i32)) != 0) as BOOL;

    if LOWORD!(lParam) == 0 {
        (*btnPtr).fsState = (((*btnPtr).fsState as INT) & (!TBSTATE_CHECKED) as i32) as BYTE;
    } else {
        if (((*btnPtr).fsStyle as INT) & (BTNS_GROUP as i32)) != 0 {
            nOldIndex = TOOLBAR_GetCheckedGroupButtonIndex(infoPtr, nIndex);
            if nOldIndex == nIndex {
                return 0 as LRESULT;
            }
            if nOldIndex != -1 {
                let fresh = &mut (*(*infoPtr).buttons.offset(nOldIndex as isize)).fsState;
                *fresh = ((*fresh as INT) & (!TBSTATE_CHECKED) as i32) as BYTE;
            }
        }
        (*btnPtr).fsState = (((*btnPtr).fsState as INT) | (TBSTATE_CHECKED as i32)) as BYTE;
    }

    if bChecked != LOWORD!(lParam) as BOOL {
        if nOldIndex != -1 {
            InvalidateRect(
                (*infoPtr).hwndSelf,
                core::ptr::addr_of_mut!((*(*infoPtr).buttons.offset(nOldIndex as isize)).rect) as LPCRECT,
                TRUE,
            );
        }
        InvalidateRect(
            (*infoPtr).hwndSelf,
            core::ptr::addr_of_mut!((*btnPtr).rect) as LPCRECT,
            TRUE,
        );
    }

    /* FIXME: Send a WM_NOTIFY?? */

    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_CommandToIndex(infoPtr: *const TOOLBAR_INFO, Id: INT) -> LRESULT {
    return TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_Customize(infoPtr: *mut TOOLBAR_INFO) -> LRESULT {
    let mut custInfo: CUSTDLG_INFO = core::mem::zeroed();
    let mut ret: LRESULT;
    let mut nmhdr: NMHDR = core::mem::zeroed();

    custInfo.tbInfo = infoPtr;
    custInfo.tbHwnd = (*infoPtr).hwndSelf;

    /* send TBN_BEGINADJUST notification */
    TOOLBAR_SendNotify(core::ptr::addr_of_mut!(nmhdr), infoPtr, (TBN_BEGINADJUST) as UINT);

    ret = DialogBoxParamW(
        COMCTL32_hModule,
        MAKEINTRESOURCEW!(IDD_TBCUSTOMIZE),
        (*infoPtr).hwndSelf,
        Some(TOOLBAR_CustomizeDialogProc),
        core::ptr::addr_of_mut!(custInfo) as LPARAM,
    ) as LRESULT;

    /* send TBN_ENDADJUST notification */
    TOOLBAR_SendNotify(core::ptr::addr_of_mut!(nmhdr), infoPtr, (TBN_ENDADJUST) as UINT);

    return ret;
}

unsafe extern "C" fn TOOLBAR_DeleteButton(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut nIndex: INT,
) -> LRESULT {
    let mut nmtb: NMTOOLBARW = core::mem::zeroed();
    let mut btnPtr: *mut TBUTTON_INFO = (*infoPtr).buttons.offset(nIndex as isize) as *mut TBUTTON_INFO;

    if nIndex < 0 || nIndex >= (*infoPtr).nNumButtons {
        return FALSE as LRESULT;
    }

    memset(
        core::ptr::addr_of_mut!(nmtb) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        core::mem::size_of::<NMTOOLBARW>() as size_t,
    );
    nmtb.iItem = (*btnPtr).idCommand as ::core::ffi::c_int;
    nmtb.tbButton.iBitmap = (*btnPtr).iBitmap as ::core::ffi::c_int;
    nmtb.tbButton.idCommand = (*btnPtr).idCommand as ::core::ffi::c_int;
    nmtb.tbButton.fsState = (*btnPtr).fsState;
    nmtb.tbButton.fsStyle = (*btnPtr).fsStyle;
    nmtb.tbButton.dwData = (*btnPtr).dwData;
    nmtb.tbButton.iString = (*btnPtr).iString;
    TOOLBAR_SendNotify(
        core::ptr::addr_of_mut!(nmtb.hdr),
        infoPtr,
        TBN_DELETINGBUTTON as UINT,
    );

    TOOLBAR_TooltipDelTool(
        infoPtr,
        (*infoPtr).buttons.offset(nIndex as isize) as *mut TBUTTON_INFO,
    );

    (*infoPtr).nHotItem = -(1) as INT;
    if (*infoPtr).nNumButtons == 1 as INT {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_DeleteButton\0" as *const u8 as *const ::core::ffi::c_char,
                3439 as ::core::ffi::c_int,
                b" simple delete\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        free_string((*infoPtr).buttons);
        Free((*infoPtr).buttons as LPVOID);
        (*infoPtr).buttons = core::ptr::null_mut::<TBUTTON_INFO>();
        (*infoPtr).nNumButtons = 0 as INT;
    } else {
        let mut oldButtons: *mut TBUTTON_INFO = (*infoPtr).buttons;
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_DeleteButton\0" as *const u8 as *const ::core::ffi::c_char,
                3447 as ::core::ffi::c_int,
                b"complex delete [nIndex=%d]\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                nIndex,
            );
        }
        (*infoPtr).nNumButtons -= 1;
        (*infoPtr).buttons = Alloc(
            (::core::mem::size_of::<TBUTTON_INFO>() as usize)
                .wrapping_mul((*infoPtr).nNumButtons as usize) as DWORD,
        ) as *mut TBUTTON_INFO;
        if nIndex > 0 as INT {
            memcpy(
                (*infoPtr).buttons.offset(0 as isize) as *mut ::core::ffi::c_void,
                oldButtons.offset(0 as isize) as *const ::core::ffi::c_void,
                (nIndex as size_t).wrapping_mul(
                    core::mem::size_of::<TBUTTON_INFO>() as size_t,
                ),
            );
        }
        if nIndex < (*infoPtr).nNumButtons {
            memcpy(
                (*infoPtr).buttons.offset(nIndex as isize) as *mut ::core::ffi::c_void,
                oldButtons.offset((nIndex + 1) as isize) as *const ::core::ffi::c_void,
                (((*infoPtr).nNumButtons - nIndex) as size_t).wrapping_mul(
                    core::mem::size_of::<TBUTTON_INFO>() as size_t,
                ),
            );
        }
        free_string(oldButtons.offset(nIndex as isize));
        Free(oldButtons as LPVOID);
    }

    TOOLBAR_LayoutToolbar(infoPtr);

    InvalidateRect(
        (*infoPtr).hwndSelf,
        core::ptr::null::<RECT>(),
        TRUE as BOOL,
    );

    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_EnableButton(
    infoPtr: *mut TOOLBAR_INFO,
    Id: INT,
    lParam: LPARAM,
) -> LRESULT {
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut();
    let mut nIndex: INT;
    let mut bState: DWORD;

    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);

    if nIndex == -1 {
        return FALSE as LRESULT;
    }

    btnPtr = (*infoPtr).buttons.offset(nIndex as isize);

    bState = ((*btnPtr).fsState as DWORD) & (TBSTATE_ENABLED as DWORD);

    /* update the toolbar button state */
    if LOWORD!(lParam as DWORD) == 0 {
        (*btnPtr).fsState = ((*btnPtr).fsState as INT
            & !(TBSTATE_ENABLED as INT | TBSTATE_PRESSED as INT)) as BYTE;
    } else {
        (*btnPtr).fsState = ((*btnPtr).fsState as INT | TBSTATE_ENABLED as INT) as BYTE;
    }

    /* redraw the button only if the state of the button changed */
    if bState != ((*btnPtr).fsState as DWORD) & (TBSTATE_ENABLED as DWORD) {
        InvalidateRect(
            (*infoPtr).hwndSelf,
            core::ptr::addr_of_mut!((*btnPtr).rect) as LPCRECT,
            TRUE,
        );
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetAnchorHighlight(infoPtr: *const TOOLBAR_INFO) -> LRESULT {
    ((*infoPtr).bAnchor) as _
}

unsafe extern "C" fn TOOLBAR_GetBitmap(infoPtr: *const TOOLBAR_INFO, Id: INT) -> LRESULT {
    let mut nIndex: INT;

    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);
    if nIndex == -1 {
        return -1;
    }

    return (*(*infoPtr).buttons.offset(nIndex as isize)).iBitmap as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetBitmapFlags() -> LRESULT {
    return (if GetDeviceCaps(core::ptr::null_mut(), (LOGPIXELSX as i32)) >= 120 {
        TBBF_LARGE
    } else {
        0
    }) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetButton(infoPtr: *const TOOLBAR_INFO, nIndex: INT, lpTbb: *mut TBBUTTON) -> LRESULT {
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut::<TBUTTON_INFO>();

    if lpTbb.is_null() {
        return FALSE as LRESULT;
    }

    if (nIndex < 0 as INT) || (nIndex >= (*infoPtr).nNumButtons) {
        return FALSE as LRESULT;
    }

    btnPtr = (*infoPtr).buttons.offset(nIndex as isize) as *mut TBUTTON_INFO;
    (*lpTbb).iBitmap = (*btnPtr).iBitmap;
    (*lpTbb).idCommand = (*btnPtr).idCommand;
    (*lpTbb).fsState = (*btnPtr).fsState;
    (*lpTbb).fsStyle = (*btnPtr).fsStyle;
    (*lpTbb).bReserved[0 as usize] = 0 as BYTE;
    (*lpTbb).bReserved[1 as usize] = 0 as BYTE;
    (*lpTbb).dwData = (*btnPtr).dwData;
    (*lpTbb).iString = (*btnPtr).iString;

    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetButtonInfoT(
    infoPtr: *const TOOLBAR_INFO,
    Id: INT,
    lpTbInfo: LPTBBUTTONINFOW,
    bUnicode: BOOL,
) -> LRESULT {
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut();
    let mut nIndex: INT;
    if lpTbInfo.is_null() {
        return -(1) as LRESULT;
    }
    if (*lpTbInfo).cbSize as usize != core::mem::size_of::<TBBUTTONINFOW>() as usize {
        return -(1) as LRESULT;
    }
    nIndex = TOOLBAR_GetButtonIndex(
        infoPtr,
        Id,
        ((*lpTbInfo).dwMask & TBIF_BYINDEX) as BOOL,
    );
    if nIndex == -(1) {
        return -(1) as LRESULT;
    }
    btnPtr = (*infoPtr).buttons.offset(nIndex as isize);
    if ((*lpTbInfo).dwMask & TBIF_COMMAND) != 0 {
        (*lpTbInfo).idCommand = (*btnPtr).idCommand;
    }
    if ((*lpTbInfo).dwMask & TBIF_IMAGE) != 0 {
        (*lpTbInfo).iImage = (*btnPtr).iBitmap;
    }
    if ((*lpTbInfo).dwMask & TBIF_LPARAM) != 0 {
        (*lpTbInfo).lParam = (*btnPtr).dwData;
    }
    if ((*lpTbInfo).dwMask & TBIF_SIZE) != 0 {
        /* tests show that for separators TBIF_SIZE returns not calculated width,
           but cx property, that differs from 0 only if application have
           specifically set it */
        (*lpTbInfo).cx = (if ((*btnPtr).fsStyle as i32 & (BTNS_SEP as i32)) != 0 {
            (*btnPtr).cx as WORD
        } else {
            ((*btnPtr).rect.right - (*btnPtr).rect.left) as WORD
        }) as WORD;
    }
    if ((*lpTbInfo).dwMask & TBIF_STATE) != 0 {
        (*lpTbInfo).fsState = (*btnPtr).fsState;
    }
    if ((*lpTbInfo).dwMask & TBIF_STYLE) != 0 {
        (*lpTbInfo).fsStyle = (*btnPtr).fsStyle;
    }
    if ((*lpTbInfo).dwMask & TBIF_TEXT) != 0 {
        /* TB_GETBUTTONINFO doesn't retrieve text from the string list, so we
           can't use TOOLBAR_GetText here */
        if !IS_INTRESOURCE!((*btnPtr).iString as LPWSTR)
            && (*btnPtr).iString != -(1) as INT_PTR
        {
            let mut lpText: LPWSTR = (*btnPtr).iString as LPWSTR;
            if bUnicode != 0 {
                Str_GetPtrW(
                    lpText as LPCWSTR,
                    (*lpTbInfo).pszText,
                    (*lpTbInfo).cchText as INT,
                );
            } else {
                Str_GetPtrWtoA(
                    lpText as LPCWSTR,
                    (*lpTbInfo).pszText as LPSTR,
                    (*lpTbInfo).cchText as INT,
                );
            }
        } else if bUnicode == 0 || !(*lpTbInfo).pszText.is_null() {
            *(*lpTbInfo).pszText.offset(0 as isize) = 0 as WCHAR;
        }
    }
    return nIndex as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetButtonSize(infoPtr: *const TOOLBAR_INFO) -> LRESULT {
    return MAKELONG!((*infoPtr).nButtonWidth as WORD,
                    (*infoPtr).nButtonHeight as WORD) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetButtonText(
    infoPtr: *const TOOLBAR_INFO,
    Id: INT,
    lpStr: LPWSTR,
    isW: BOOL,
) -> LRESULT {
    let mut nIndex: INT;
    let mut lpText: LPWSTR;
    let mut ret: LRESULT = 0 as LRESULT;

    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE as BOOL);
    if nIndex == -(1 as INT) {
        return -(1 as INT) as LRESULT;
    }

    lpText = TOOLBAR_GetText(infoPtr, (*infoPtr).buttons.offset(nIndex as isize));

    if isW != 0 {
        if !lpText.is_null() {
            ret = lstrlenW(lpText as LPCWSTR) as LRESULT;
            if !lpStr.is_null() {
                lstrcpyW(lpStr, lpText as LPCWSTR);
            }
        }
    } else {
        ret = (WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            lpText as LPCWSTR,
            -(1 as ::core::ffi::c_int),
            lpStr as LPSTR,
            (if !lpStr.is_null() {
                0x7fffffff as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }),
            ::core::ptr::null::<CHAR>(),
            ::core::ptr::null_mut::<BOOL>(),
        ) - 1 as ::core::ffi::c_int) as LRESULT;
    }
    return ret;
}

unsafe extern "C" fn TOOLBAR_GetDisabledImageList(
    mut infoPtr: *const TOOLBAR_INFO,
    mut wParam: WPARAM,
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
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_GetDisabledImageList\0" as *const u8 as *const ::core::ffi::c_char,
            3654 as ::core::ffi::c_int,
            b"hwnd=%p, wParam=%ld\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).hwndSelf,
            wParam,
        );
    }
    return TOOLBAR_GetImageList((*infoPtr).himlDis, (*infoPtr).cimlDis, wParam as INT) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetExtendedStyle(infoPtr: *const TOOLBAR_INFO) -> LRESULT {
    // TRACE("\n");
    ((*infoPtr).dwExStyle) as _
}

unsafe extern "C" fn TOOLBAR_GetHotImageList(infoPtr: *const TOOLBAR_INFO, wParam: WPARAM) -> LRESULT {
    /* UNDOCUMENTED: wParam is actually the ID of the image list to return */
    return TOOLBAR_GetImageList((*infoPtr).himlHot, (*infoPtr).cimlHot, wParam as INT) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetHotItem(infoPtr: *const TOOLBAR_INFO) -> LRESULT {
    if !(((*infoPtr).dwStyle & TBSTYLE_FLAT as DWORD) != 0 || !GetWindowTheme((*infoPtr).hwndSelf).is_null()) {
        return -(1 as LRESULT);
    }

    if (*infoPtr).nHotItem < 0 as INT {
        return -(1 as LRESULT);
    }

    return (*infoPtr).nHotItem as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetDefImageList(infoPtr: *const TOOLBAR_INFO, wParam: WPARAM) -> LRESULT {
    return TOOLBAR_GetImageList((*infoPtr).himlDef, (*infoPtr).cimlDef, wParam as INT) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetInsertMark(infoPtr: *const TOOLBAR_INFO, lptbim: *mut TBINSERTMARK) -> LRESULT {
    *lptbim = (*infoPtr).tbim;

    return 0;
}

unsafe extern "C" fn TOOLBAR_GetInsertMarkColor(infoPtr: *const TOOLBAR_INFO) -> LRESULT {
    return (*infoPtr).clrInsertMark as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetItemRect(infoPtr: *const TOOLBAR_INFO, nIndex: INT, lpRect: LPRECT) -> LRESULT {
    let mut btnPtr: *mut TBUTTON_INFO;
    btnPtr = (*infoPtr).buttons.offset(nIndex as isize) as *mut TBUTTON_INFO;
    if (nIndex < 0) || (nIndex >= (*infoPtr).nNumButtons) {
        return (FALSE) as LRESULT;
    }
    if lpRect.is_null() {
        return (FALSE) as LRESULT;
    }
    if (*btnPtr).fsState as ::core::ffi::c_int & (TBSTATE_HIDDEN as i32) != 0 {
        return (FALSE) as LRESULT;
    }
    (*lpRect).left = (*btnPtr).rect.left;
    (*lpRect).right = (*btnPtr).rect.right;
    (*lpRect).bottom = (*btnPtr).rect.bottom;
    (*lpRect).top = (*btnPtr).rect.top;
    return (TRUE) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetMaxSize(
    mut infoPtr: *const TOOLBAR_INFO,
    mut lpSize: LPSIZE,
) -> LRESULT {
    if lpSize.is_null() {
        return FALSE as LRESULT;
    }
    (*lpSize).cx = (*infoPtr).rcBound.right - (*infoPtr).rcBound.left;
    (*lpSize).cy = (*infoPtr).rcBound.bottom - (*infoPtr).rcBound.top;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_GetMaxSize\0" as *const u8 as *const ::core::ffi::c_char,
            3754 as ::core::ffi::c_int,
            b"maximum size %d x %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).rcBound.right - (*infoPtr).rcBound.left,
            (*infoPtr).rcBound.bottom - (*infoPtr).rcBound.top,
        );
    }
    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetMetrics(
    mut infoPtr: *const TOOLBAR_INFO,
    mut pMetrics: *mut TBMETRICS,
) -> LRESULT {
    if pMetrics.is_null()
        || (*pMetrics).cbSize as usize != ::core::mem::size_of::<TBMETRICS>() as usize
    {
        return 0 as LRESULT;
    }
    if (*pMetrics).dwMask & TBMF_PAD as DWORD != 0 {
        (*pMetrics).cxPad = (*infoPtr).szPadding.cx as ::core::ffi::c_int;
        (*pMetrics).cyPad = (*infoPtr).szPadding.cy as ::core::ffi::c_int;
    }
    if (*pMetrics).dwMask & TBMF_BARPAD as DWORD != 0 {
        (*pMetrics).cxBarPad = (*infoPtr).szBarPadding.cx as ::core::ffi::c_int;
        (*pMetrics).cyBarPad = (*infoPtr).szBarPadding.cy as ::core::ffi::c_int;
    }
    if (*pMetrics).dwMask & TBMF_BUTTONSPACING as DWORD != 0 {
        (*pMetrics).cxButtonSpacing = (*infoPtr).szSpacing.cx as ::core::ffi::c_int;
        (*pMetrics).cyButtonSpacing = (*infoPtr).szSpacing.cy as ::core::ffi::c_int;
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetPadding(infoPtr: *const TOOLBAR_INFO) -> LRESULT {
    let lo: DWORD = ((*infoPtr).szPadding.cx as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as DWORD;
    let hi: DWORD = ((*infoPtr).szPadding.cy as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as DWORD;
    return (lo | (hi << 16 as ::core::ffi::c_int)) as LONG as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetRect(infoPtr: *const TOOLBAR_INFO, Id: INT, lpRect: LPRECT) -> LRESULT {
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut::<TBUTTON_INFO>();
    let mut nIndex: INT = 0;

    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);
    btnPtr = (*infoPtr).buttons.offset(nIndex as isize) as *mut TBUTTON_INFO;
    if (nIndex < 0) || (nIndex >= (*infoPtr).nNumButtons) {
        return FALSE as LRESULT;
    }

    if lpRect.is_null() {
        return FALSE as LRESULT;
    }

    (*lpRect).left = (*btnPtr).rect.left;
    (*lpRect).right = (*btnPtr).rect.right;
    (*lpRect).bottom = (*btnPtr).rect.bottom;
    (*lpRect).top = (*btnPtr).rect.top;

    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetRows(infoPtr: *const TOOLBAR_INFO) -> LRESULT {
    ((*infoPtr).nRows) as _
}

unsafe extern "C" fn TOOLBAR_GetState(infoPtr: *const TOOLBAR_INFO, Id: INT) -> LRESULT {
    let mut nIndex: INT;

    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);
    if nIndex == -1 {
        return -1 as LRESULT;
    }

    return (*(*infoPtr).buttons.offset(nIndex as isize)).fsState as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetStyle(infoPtr: *const TOOLBAR_INFO) -> LRESULT {
    ((*infoPtr).dwStyle) as _
}

unsafe extern "C" fn TOOLBAR_GetTextRows(infoPtr: *const TOOLBAR_INFO) -> LRESULT {
    ((*infoPtr).nMaxTextRows) as _
}

unsafe extern "C" fn TOOLBAR_GetToolTips(infoPtr: *mut TOOLBAR_INFO) -> LRESULT {
    if (((*infoPtr).dwStyle & (TBSTYLE_TOOLTIPS as DWORD)) != 0) && (*infoPtr).hwndToolTip.is_null() {
        TOOLBAR_TooltipCreateControl(infoPtr);
    }
    return (*infoPtr).hwndToolTip as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetUnicodeFormat(infoPtr: *const TOOLBAR_INFO) -> LRESULT {
    // TRACE("%s hwnd=%p\n", infoPtr->bUnicode ? "TRUE" : "FALSE", infoPtr->hwndSelf);
    ((*infoPtr).bUnicode) as _
}

unsafe extern "C" fn TOOLBAR_GetVersion(infoPtr: *const TOOLBAR_INFO) -> LRESULT {
    ((*infoPtr).iVersion) as _
}

unsafe extern "C" fn TOOLBAR_HideButton(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut Id: INT,
    mut fHide: BOOL,
) -> LRESULT {
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut::<TBUTTON_INFO>();
    let mut oldState: BYTE = 0;
    let mut nIndex: INT = 0;
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"TOOLBAR_HideButton\0" as *const u8 as *const core::ffi::c_char,
            3888 as core::ffi::c_int,
            b"\n\0" as *const u8 as *const core::ffi::c_char,
        );
    }
    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);
    if nIndex == -1 {
        return FALSE as LRESULT;
    }
    btnPtr = (*infoPtr).buttons.offset(nIndex as isize) as *mut TBUTTON_INFO;
    oldState = (*btnPtr).fsState;
    if fHide != 0 {
        (*btnPtr).fsState = ((*btnPtr).fsState as core::ffi::c_int | (TBSTATE_HIDDEN as i32)) as BYTE;
    } else {
        (*btnPtr).fsState = ((*btnPtr).fsState as core::ffi::c_int & (!TBSTATE_HIDDEN) as i32) as BYTE;
    }
    if oldState as core::ffi::c_int != (*btnPtr).fsState as core::ffi::c_int {
        TOOLBAR_LayoutToolbar(infoPtr);
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE);
    }
    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_HitTest(
    infoPtr: *const TOOLBAR_INFO,
    lpPt: *const POINT,
) -> LRESULT {
    (TOOLBAR_InternalHitTest(infoPtr, lpPt, core::ptr::null_mut())) as _
}

unsafe extern "C" fn TOOLBAR_Indeterminate(infoPtr: *const TOOLBAR_INFO, Id: INT, fIndeterminate: BOOL) -> LRESULT {
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut();
    let mut nIndex: INT = 0;
    let mut oldState: DWORD = 0;

    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);
    if nIndex == -1 {
        return FALSE as LRESULT;
    }

    btnPtr = (*infoPtr).buttons.offset(nIndex as isize) as *mut TBUTTON_INFO;
    oldState = (*btnPtr).fsState as DWORD;

    if fIndeterminate != 0 {
        (*btnPtr).fsState = ((*btnPtr).fsState as INT | (TBSTATE_INDETERMINATE as i32)) as BYTE;
    } else {
        (*btnPtr).fsState = ((*btnPtr).fsState as INT & (!TBSTATE_INDETERMINATE) as i32) as BYTE;
    }

    if oldState != (*btnPtr).fsState as DWORD {
        InvalidateRect(
            (*infoPtr).hwndSelf,
            core::ptr::addr_of_mut!((*btnPtr).rect) as LPCRECT,
            TRUE,
        );
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_InsertButtonT(
    infoPtr: *mut TOOLBAR_INFO,
    mut nIndex: INT,
    lpTbb: *const TBBUTTON,
    fUnicode: BOOL,
) -> LRESULT {
    if lpTbb.is_null() {
        return (FALSE) as LRESULT;
    }

    if nIndex == -1 {
        /* EPP: this seems to be an undocumented call (from my IE4)
         * I assume in that case that:
         * - index of insertion is at the end of existing buttons
         * I only see this happen with nIndex == -1, but it could have a special
         * meaning (like -nIndex (or ~nIndex) to get the real position of insertion).
         */
        nIndex = (*infoPtr).nNumButtons;
    } else if nIndex < 0 {
        return (FALSE) as LRESULT;
    }

    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_InsertButtonT\0" as *const u8 as *const ::core::ffi::c_char,
            3962 as ::core::ffi::c_int,
            b"inserting button index=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            nIndex,
        );
    }

    if nIndex > (*infoPtr).nNumButtons {
        nIndex = (*infoPtr).nNumButtons;
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_InsertButtonT\0" as *const u8 as *const ::core::ffi::c_char,
                3965 as ::core::ffi::c_int,
                b"adjust index=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
                nIndex,
            );
        }
    }

    return TOOLBAR_InternalInsertButtonsT(infoPtr, nIndex, 1, lpTbb, fUnicode) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_IsButtonChecked(infoPtr: *const TOOLBAR_INFO, Id: INT) -> LRESULT {
    let mut nIndex: INT = 0;
    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);
    if nIndex == -1 {
        return -1 as LRESULT;
    }

    return ((*(*infoPtr).buttons.offset(nIndex as isize)).fsState as INT
        & (TBSTATE_CHECKED as i32)) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_IsButtonEnabled(infoPtr: *const TOOLBAR_INFO, Id: INT) -> LRESULT {
    let mut nIndex: INT = 0;

    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);
    if nIndex == -1 {
        return -1 as LRESULT;
    }

    return ((*(*infoPtr).buttons.offset(nIndex as isize)).fsState as INT & (TBSTATE_ENABLED as i32)) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_IsButtonHidden(infoPtr: *const TOOLBAR_INFO, Id: INT) -> LRESULT {
    let mut nIndex: INT = 0;
    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);
    if nIndex == -1 {
        return -1 as LRESULT;
    }
    return ((*(*infoPtr).buttons.offset(nIndex as isize)).fsState as INT & (TBSTATE_HIDDEN as i32)) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_IsButtonHighlighted(infoPtr: *const TOOLBAR_INFO, Id: INT) -> LRESULT {
    let mut nIndex: INT;

    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);
    if nIndex == -1 {
        return -1 as LRESULT;
    }

    return ((*(*infoPtr).buttons.offset(nIndex as isize)).fsState as INT & (TBSTATE_MARKED as i32)) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_IsButtonIndeterminate(infoPtr: *const TOOLBAR_INFO, Id: INT) -> LRESULT {
    let mut nIndex: INT;
    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);
    if nIndex == -1 {
        return -1 as LRESULT;
    }

    return ((*(*infoPtr).buttons.offset(nIndex as isize)).fsState as INT & (TBSTATE_INDETERMINATE as i32)) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_IsButtonPressed(infoPtr: *const TOOLBAR_INFO, Id: INT) -> LRESULT {
    let mut nIndex: INT = 0;
    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);
    if nIndex == -1 {
        return -1 as LRESULT;
    }
    return ((*(*infoPtr).buttons.offset(nIndex as isize)).fsState as INT
        & (TBSTATE_PRESSED as i32)) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_LoadImages(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut wParam: WPARAM,
    mut hInstance: HINSTANCE,
) -> LRESULT {
    let mut tbab: TBADDBITMAP = tagTBADDBITMAP {
        hInst: ::core::ptr::null_mut::<HINSTANCE__>(),
        nID: 0,
    };
    tbab.hInst = hInstance;
    tbab.nID = wParam as UINT_PTR;
    return TOOLBAR_AddBitmap(infoPtr, 0, core::ptr::addr_of_mut!(tbab));
}

unsafe extern "C" fn TOOLBAR_MapAccelerator(
    infoPtr: *const TOOLBAR_INFO,
    wAccel: WCHAR,
    pIDButton: *mut UINT,
) -> LRESULT {
    let mut wszAccel: [WCHAR; 3] = ['&' as WCHAR, wAccel, 0];
    let mut i: i32;

    i = 0;
    while i < (*infoPtr).nNumButtons {
        let btnPtr: *mut TBUTTON_INFO = (*infoPtr).buttons.offset(i as isize);
        if !(((*btnPtr).fsStyle as i32 & (BTNS_NOPREFIX as i32)) != 0)
            && !(((*btnPtr).fsState as i32 & (TBSTATE_HIDDEN as i32)) != 0)
        {
            let iLen: i32 = lstrlenW(
                core::ptr::addr_of_mut!(wszAccel) as *mut WCHAR as LPCWSTR,
            );
            let mut lpszStr: LPCWSTR =
                TOOLBAR_GetText(infoPtr, btnPtr) as LPCWSTR;

            if lpszStr.is_null() {
                i += 1;
                continue;
            }

            while *lpszStr != 0 {
                if (*lpszStr.offset(0 as isize)) as i32 == '&' as i32
                    && (*lpszStr.offset(1 as isize)) as i32 == '&' as i32
                {
                    lpszStr = lpszStr.offset(2 as isize);
                    continue;
                }
                if wcsnicmp(
                    lpszStr as *const WCHAR,
                    core::ptr::addr_of_mut!(wszAccel) as *mut WCHAR,
                    iLen as usize,
                ) == 0
                {
                    *pIDButton = (*btnPtr).idCommand as UINT;
                    return (TRUE) as LRESULT;
                }
                lpszStr = lpszStr.offset(1);
            }
        }
        i += 1;
    }
    return (FALSE) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_MarkButton(infoPtr: *const TOOLBAR_INFO, Id: INT, fMark: BOOL) -> LRESULT {
    let mut nIndex: INT;
    let mut oldState: DWORD;
    let mut btnPtr: *mut TBUTTON_INFO;

    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);
    if nIndex == -1 {
        return FALSE as LRESULT;
    }

    btnPtr = (*infoPtr).buttons.offset(nIndex as isize) as *mut TBUTTON_INFO;
    oldState = (*btnPtr).fsState as DWORD;

    if fMark != 0 {
        (*btnPtr).fsState = (((*btnPtr).fsState as INT) | (TBSTATE_MARKED as i32)) as BYTE;
    } else {
        (*btnPtr).fsState = (((*btnPtr).fsState as INT) & (!TBSTATE_MARKED) as i32) as BYTE;
    }

    if oldState != (*btnPtr).fsState as DWORD {
        InvalidateRect(
            (*infoPtr).hwndSelf,
            core::ptr::addr_of_mut!((*btnPtr).rect) as LPCRECT,
            TRUE,
        );
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_MoveFixupIndex(pIndex: *mut INT, nIndex: INT, nMoveIndex: INT, bMoveUp: BOOL) {
    if bMoveUp != 0 {
        if *pIndex > nIndex && *pIndex <= nMoveIndex {
            *pIndex -= 1;
        } else if *pIndex == nIndex {
            *pIndex = nMoveIndex;
        }
    } else {
        if *pIndex >= nMoveIndex && *pIndex < nIndex {
            *pIndex += 1;
        } else if *pIndex == nIndex {
            *pIndex = nMoveIndex;
        }
    }
}

unsafe extern "C" fn TOOLBAR_MoveButton(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut Id: INT,
    mut nMoveIndex: INT,
) -> LRESULT {
    let mut nIndex: INT = 0;
    let mut nCount: INT = 0;
    let mut button: TBUTTON_INFO = TBUTTON_INFO {
        iBitmap: 0,
        idCommand: 0,
        fsState: 0,
        fsStyle: 0,
        bHot: 0,
        bDropDownPressed: 0,
        dwData: 0,
        iString: 0,
        nRow: 0,
        rect: tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        cx: 0,
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
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_MoveButton\0" as *const u8 as *const ::core::ffi::c_char,
            4161 as ::core::ffi::c_int,
            b"hwnd=%p, Id=%d, nMoveIndex=%d\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*infoPtr).hwndSelf,
            Id,
            nMoveIndex,
        );
    }

    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, TRUE);
    if (nIndex == -1) || (nMoveIndex < 0) {
        return FALSE as LRESULT;
    }

    if nMoveIndex > (*infoPtr).nNumButtons - 1 {
        nMoveIndex = (*infoPtr).nNumButtons - 1;
    }

    button = core::ptr::read((*infoPtr).buttons.offset(nIndex as isize));

    /* move button right */
    if nIndex < nMoveIndex {
        nCount = nMoveIndex - nIndex;
        memmove(
            (*infoPtr).buttons.offset(nIndex as isize) as *mut TBUTTON_INFO
                as *mut ::core::ffi::c_void,
            (*infoPtr).buttons.offset((nIndex + 1) as isize)
                as *mut TBUTTON_INFO as *const ::core::ffi::c_void,
            (nCount as size_t).wrapping_mul(
                core::mem::size_of::<TBUTTON_INFO>() as size_t,
            ),
        );
        *(*infoPtr).buttons.offset(nMoveIndex as isize) = button;

        TOOLBAR_MoveFixupIndex(
            &raw mut (*infoPtr).nButtonDown,
            nIndex,
            nMoveIndex,
            TRUE,
        );
        TOOLBAR_MoveFixupIndex(
            &raw mut (*infoPtr).nButtonDrag,
            nIndex,
            nMoveIndex,
            TRUE,
        );
        TOOLBAR_MoveFixupIndex(
            &raw mut (*infoPtr).nOldHit,
            nIndex,
            nMoveIndex,
            TRUE,
        );
        TOOLBAR_MoveFixupIndex(
            &raw mut (*infoPtr).nHotItem,
            nIndex,
            nMoveIndex,
            TRUE,
        );
    } else if nIndex > nMoveIndex {
        /* move button left */
        nCount = nIndex - nMoveIndex;
        memmove(
            (*infoPtr).buttons.offset((nMoveIndex + 1) as isize)
                as *mut TBUTTON_INFO as *mut ::core::ffi::c_void,
            (*infoPtr).buttons.offset(nMoveIndex as isize) as *mut TBUTTON_INFO
                as *const ::core::ffi::c_void,
            (nCount as size_t).wrapping_mul(
                core::mem::size_of::<TBUTTON_INFO>() as size_t,
            ),
        );
        *(*infoPtr).buttons.offset(nMoveIndex as isize) = button;

        TOOLBAR_MoveFixupIndex(
            &raw mut (*infoPtr).nButtonDown,
            nIndex,
            nMoveIndex,
            FALSE,
        );
        TOOLBAR_MoveFixupIndex(
            &raw mut (*infoPtr).nButtonDrag,
            nIndex,
            nMoveIndex,
            FALSE,
        );
        TOOLBAR_MoveFixupIndex(
            &raw mut (*infoPtr).nOldHit,
            nIndex,
            nMoveIndex,
            FALSE,
        );
        TOOLBAR_MoveFixupIndex(
            &raw mut (*infoPtr).nHotItem,
            nIndex,
            nMoveIndex,
            FALSE,
        );
    }

    TOOLBAR_LayoutToolbar(infoPtr);
    TOOLBAR_AutoSize(infoPtr);
    InvalidateRect((*infoPtr).hwndSelf, ::core::ptr::null::<RECT>(), TRUE);

    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_PressButton(infoPtr: *const TOOLBAR_INFO, Id: INT, fPress: BOOL) -> LRESULT {
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut();
    let mut nIndex: INT;
    let mut oldState: DWORD;

    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);
    if nIndex == -1 {
        return FALSE as LRESULT;
    }

    btnPtr = (*infoPtr).buttons.offset(nIndex as isize);
    oldState = (*btnPtr).fsState as DWORD;

    if fPress != 0 {
        (*btnPtr).fsState = ((*btnPtr).fsState as INT | (TBSTATE_PRESSED as i32)) as BYTE;
    } else {
        (*btnPtr).fsState = ((*btnPtr).fsState as INT & (!TBSTATE_PRESSED) as i32) as BYTE;
    }

    if oldState != (*btnPtr).fsState as DWORD {
        InvalidateRect(
            (*infoPtr).hwndSelf,
            core::ptr::addr_of_mut!((*btnPtr).rect) as LPCRECT,
            TRUE,
        );
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_ReplaceBitmap(infoPtr: *mut TOOLBAR_INFO, lpReplace: *const TBREPLACEBITMAP) -> LRESULT {
    let mut hBitmap: HBITMAP = core::ptr::null_mut::<HBITMAP__>();
    let mut i: ::core::ffi::c_int = 0;
    let mut nOldButtons: ::core::ffi::c_int = 0;
    let mut pos: ::core::ffi::c_int = 0;
    let mut nOldBitmaps: ::core::ffi::c_int = 0;
    let mut nNewBitmaps: ::core::ffi::c_int = 0;
    let mut himlDef: HIMAGELIST = core::ptr::null_mut();
    if (*lpReplace).hInstOld == HINST_COMMCTRL {
        return FALSE as LRESULT;
    } else if (*lpReplace).hInstOld != core::ptr::null_mut() && (*lpReplace).hInstOld != (*lpReplace).hInstNew {
        FIXME!("resources not in the current module not implemented\n");
    }
    i = 0;
    while i < (*infoPtr).nNumBitmapInfos {
        let mut tbi: *mut TBITMAP_INFO = (*infoPtr).bitmaps.offset(i as isize) as *mut TBITMAP_INFO;
        if (*tbi).hInst == (*lpReplace).hInstOld && (*tbi).nID as UINT_PTR == (*lpReplace).nIDOld {
            nOldButtons = (*tbi).nButtons as ::core::ffi::c_int;
            (*tbi).nButtons = (*lpReplace).nButtons as UINT;
            (*tbi).hInst = (*lpReplace).hInstNew;
            (*tbi).nID = (*lpReplace).nIDNew as UINT;
            break;
        } else {
            pos = (pos as UINT).wrapping_add((*tbi).nButtons) as ::core::ffi::c_int;
            i += 1;
        }
    }
    if nOldButtons == 0 {
        return FALSE as LRESULT;
    }
    if !(*lpReplace).hInstNew.is_null() {
        hBitmap = LoadBitmapW(
            (*lpReplace).hInstNew,
            (*lpReplace).nIDNew as LPWSTR as LPCWSTR,
        );
    } else {
        hBitmap = CopyImage(
            (*lpReplace).nIDNew as HBITMAP as HANDLE,
            IMAGE_BITMAP as UINT,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as UINT,
        ) as HBITMAP;
    }
    himlDef = GETDEFIMAGELIST!(infoPtr, 0);
    nOldBitmaps = ImageList_GetImageCount(himlDef);
    i = pos + nOldBitmaps - 1;
    while i >= pos {
        ImageList_Remove(himlDef, i);
        i -= 1;
    }
    if !hBitmap.is_null() {
        ImageList_AddMasked(himlDef, hBitmap, comctl32_color.clrBtnFace);
        nNewBitmaps = ImageList_GetImageCount(himlDef);
        DeleteObject(hBitmap as HGDIOBJ);
    }
    (*infoPtr).nNumBitmaps =
        ((*infoPtr).nNumBitmaps as ::core::ffi::c_int - nOldBitmaps + nNewBitmaps) as INT;
    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), TRUE);
    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_Save(infoPtr: *mut TOOLBAR_INFO, params: *const TBSAVEPARAMSW) -> BOOL {
    let mut save: NMTBSAVE = core::mem::zeroed();
    let mut ret: INT;
    let mut i: INT;
    let mut alloced: BOOL = FALSE;
    let mut key: HKEY = core::ptr::null_mut();

    memset(
        core::ptr::addr_of_mut!(save) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<NMTBSAVE>(),
    );
    save.cbData = (((*infoPtr).nNumButtons as usize) * core::mem::size_of::<DWORD>()) as UINT;
    save.iItem = -1;
    save.cButtons = ((*infoPtr).nNumButtons as UINT) as i32;
    save.tbButton.idCommand = -1;
    TOOLBAR_SendNotify(core::ptr::addr_of_mut!(save.hdr), infoPtr, (TBN_SAVE) as UINT);

    if save.pData.is_null() {
        save.pData = Alloc(save.cbData) as *mut DWORD;
        if save.pData.is_null() {
            return FALSE;
        }
        alloced = TRUE;
    }
    if save.pCurrent.is_null() {
        save.pCurrent = save.pData;
    }

    i = 0;
    while i < (*infoPtr).nNumButtons {
        save.iItem = i;
        save.tbButton.iBitmap = (*(*infoPtr).buttons.add(i as usize)).iBitmap;
        save.tbButton.idCommand = (*(*infoPtr).buttons.add(i as usize)).idCommand;
        save.tbButton.fsState = (*(*infoPtr).buttons.add(i as usize)).fsState;
        save.tbButton.fsStyle = (*(*infoPtr).buttons.add(i as usize)).fsStyle;
        memset(
            core::ptr::addr_of_mut!(save.tbButton.bReserved) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<[BYTE; 6]>(),
        );
        save.tbButton.dwData = (*(*infoPtr).buttons.add(i as usize)).dwData;
        save.tbButton.iString = (*(*infoPtr).buttons.add(i as usize)).iString;

        let fresh = save.pCurrent;
        save.pCurrent = save.pCurrent.add(1);
        *fresh = save.tbButton.idCommand as DWORD;

        TOOLBAR_SendNotify(core::ptr::addr_of_mut!(save.hdr), infoPtr, (TBN_SAVE) as UINT);
        i += 1;
    }

    ret = RegCreateKeyW(
        (*params).hkr,
        (*params).pszSubKey,
        core::ptr::addr_of_mut!(key),
    ) as INT;
    if ret == (ERROR_SUCCESS as i32) {
        ret = RegSetValueExW(
            key,
            (*params).pszValueName,
            0,
            REG_BINARY,
            save.pData as *mut BYTE,
            save.cbData,
        ) as INT;
        RegCloseKey(key);
    }

    if alloced != 0 {
        Free(save.pData as LPVOID);
    }
    return (ret == 0) as BOOL;
}

unsafe extern "C" fn TOOLBAR_DeleteAllButtons(infoPtr: *mut TOOLBAR_INFO) {
    let mut i: INT;
    i = 0;
    while i < (*infoPtr).nNumButtons {
        free_string((*infoPtr).buttons.offset(i as isize));
        TOOLBAR_TooltipDelTool(infoPtr, (*infoPtr).buttons.offset(i as isize) as *mut TBUTTON_INFO);
        i += 1;
    }

    Free((*infoPtr).buttons as LPVOID);
    (*infoPtr).buttons = core::ptr::null_mut::<TBUTTON_INFO>();
    (*infoPtr).nNumButtons = 0;
}

unsafe extern "C" fn TOOLBAR_Restore(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut lpSave: *const TBSAVEPARAMSW,
) -> BOOL {
    let mut res: LONG = 0;
    let mut hkey: HKEY = ::core::ptr::null_mut::<HKEY__>();
    let mut ret: BOOL = FALSE;
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD = 0 as DWORD;
    let mut nmtbr: NMTBRESTORE = tagNMTBRESTORE {
        hdr: tagNMHDR {
            hwndFrom: ::core::ptr::null_mut::<HWND__>(),
            idFrom: 0,
            code: 0,
        },
        pData: ::core::ptr::null_mut::<DWORD>(),
        pCurrent: ::core::ptr::null_mut::<DWORD>(),
        cbData: 0,
        iItem: 0,
        cButtons: 0,
        cbBytesPerRecord: 0,
        tbButton: _TBBUTTON {
            iBitmap: 0,
            idCommand: 0,
            fsState: 0,
            fsStyle: 0,
            bReserved: Default::default(),
            dwData: 0,
            iString: 0,
        },
    };
    let mut hdr: NMHDR = tagNMHDR {
        hwndFrom: ::core::ptr::null_mut::<HWND__>(),
        idFrom: 0,
        code: 0,
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
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_Restore\0" as *const u8 as *const ::core::ffi::c_char,
            4393 as ::core::ffi::c_int,
            b"restore from %s %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w((*lpSave).pszSubKey as *const WCHAR),
            debugstr_w((*lpSave).pszValueName as *const WCHAR),
        );
    }

    memset(
        &raw mut nmtbr as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<NMTBRESTORE>() as size_t,
    );

    res = RegOpenKeyExW(
        (*lpSave).hkr,
        (*lpSave).pszSubKey,
        0 as DWORD,
        KEY_QUERY_VALUE as REGSAM,
        &raw mut hkey,
    ) as LONG;
    if res == 0 {
        res = RegQueryValueExW(
            hkey,
            (*lpSave).pszValueName,
            ::core::ptr::null_mut::<DWORD>(),
            &raw mut dwType,
            ::core::ptr::null_mut::<BYTE>(),
            &raw mut dwSize,
        ) as LONG;
    }
    if res == 0 && dwType != REG_BINARY as DWORD {
        res = ERROR_FILE_NOT_FOUND as LONG;
    }
    if res == 0 {
        nmtbr.pData = Alloc(dwSize) as *mut DWORD;
        nmtbr.cbData = dwSize as UINT;
        if nmtbr.pData.is_null() {
            res = ERROR_OUTOFMEMORY as LONG;
        }
    }
    if res == 0 {
        res = RegQueryValueExW(
            hkey,
            (*lpSave).pszValueName,
            ::core::ptr::null_mut::<DWORD>(),
            &raw mut dwType,
            nmtbr.pData as LPBYTE,
            &raw mut dwSize,
        ) as LONG;
    }
    if res == 0 {
        nmtbr.pCurrent = nmtbr.pData;
        nmtbr.iItem = -(1 as ::core::ffi::c_int);
        nmtbr.cbBytesPerRecord = ::core::mem::size_of::<DWORD>() as ::core::ffi::c_int;
        nmtbr.cButtons =
            nmtbr.cbData.wrapping_div(nmtbr.cbBytesPerRecord as UINT) as ::core::ffi::c_int;

        if TOOLBAR_SendNotify(&raw mut nmtbr.hdr, infoPtr, TBN_RESTORE as UINT) == 0 {
            let mut i: INT = 0;
            let mut count: INT = nmtbr.cButtons as INT;

            /* remove all existing buttons as this function is designed to
             * restore the toolbar to a previously saved state */
            TOOLBAR_DeleteAllButtons(infoPtr);

            i = 0 as INT;
            while i < count {
                nmtbr.iItem = i as ::core::ffi::c_int;
                nmtbr.tbButton.iBitmap = -(1 as ::core::ffi::c_int);
                nmtbr.tbButton.fsState = 0 as BYTE;
                nmtbr.tbButton.fsStyle = 0 as BYTE;
                nmtbr.tbButton.dwData = 0 as DWORD_PTR;
                nmtbr.tbButton.iString = 0 as INT_PTR;

                if (*nmtbr.pCurrent & (0x80000000 as DWORD)) != 0 {
                    /* separator */
                    nmtbr.tbButton.iBitmap = SEPARATOR_WIDTH as INT;
                    nmtbr.tbButton.idCommand = 0 as ::core::ffi::c_int;
                    nmtbr.tbButton.fsStyle = BTNS_SEP as BYTE;
                    if *nmtbr.pCurrent
                        != ((-(1 as ::core::ffi::c_int)) as DWORD)
                    {
                        nmtbr.tbButton.fsState = TBSTATE_HIDDEN as BYTE;
                    }
                } else {
                    nmtbr.tbButton.idCommand = *nmtbr.pCurrent as ::core::ffi::c_int;
                }

                nmtbr.pCurrent = nmtbr.pCurrent.offset(1);

                TOOLBAR_SendNotify(&raw mut nmtbr.hdr, infoPtr, TBN_RESTORE as UINT);

                /* All returned ptrs and -1 are ignored */
                if !IS_INTRESOURCE!(nmtbr.tbButton.iString) {
                    nmtbr.tbButton.iString = 0 as INT_PTR;
                }

                TOOLBAR_InsertButtonT(
                    infoPtr,
                    -(1 as INT),
                    &raw mut nmtbr.tbButton,
                    TRUE as BOOL,
                );
                i += 1;
            }

            TOOLBAR_SendNotify(&raw mut hdr, infoPtr, TBN_BEGINADJUST as UINT);
            i = 0 as INT;
            'adjust: loop {
                let mut tb: NMTOOLBARW = tagNMTOOLBARW {
                    hdr: tagNMHDR {
                        hwndFrom: ::core::ptr::null_mut::<HWND__>(),
                        idFrom: 0,
                        code: 0,
                    },
                    iItem: 0,
                    tbButton: _TBBUTTON {
                        iBitmap: 0,
                        idCommand: 0,
                        fsState: 0,
                        fsStyle: 0,
                        bReserved: Default::default(),
                        dwData: 0,
                        iString: 0,
                    },
                    cchText: 0,
                    pszText: ::core::ptr::null_mut::<WCHAR>(),
                    rcButton: tagRECT {
                        left: 0,
                        top: 0,
                        right: 0,
                        bottom: 0,
                    },
                };
                let mut bi: TBBUTTONINFOW = TBBUTTONINFOW {
                    cbSize: 0,
                    dwMask: 0,
                    idCommand: 0,
                    iImage: 0,
                    fsState: 0,
                    fsStyle: 0,
                    cx: 0,
                    lParam: 0,
                    pszText: ::core::ptr::null_mut::<WCHAR>(),
                    cchText: 0,
                };
                let mut buf: [WCHAR; 128] = [0; 128];
                let mut code: UINT = if (*infoPtr).bUnicode != 0 {
                    TBN_GETBUTTONINFOW as UINT
                } else {
                    TBN_GETBUTTONINFOA as UINT
                };
                let mut idx: INT = 0;

                memset(
                    &raw mut tb as *mut ::core::ffi::c_void,
                    0 as ::core::ffi::c_int,
                    ::core::mem::size_of::<NMTOOLBARW>() as size_t,
                );
                tb.iItem = i as ::core::ffi::c_int;
                tb.cchText = ARRAY_SIZE!(buf) as ::core::ffi::c_int;
                tb.pszText = &raw mut buf as *mut WCHAR as LPWSTR;

                /* Use the same struct for both A and W versions since the layout is the same. */
                if TOOLBAR_SendNotify(&raw mut tb.hdr, infoPtr, code) == 0 {
                    break 'adjust;
                }

                'body: {
                    idx = TOOLBAR_GetButtonIndex(
                        infoPtr,
                        tb.tbButton.idCommand as INT,
                        FALSE as BOOL,
                    );
                    if idx == -(1 as INT) {
                        break 'body;
                    }

                    /* tb.pszText is ignored - the string comes from tb.tbButton.iString, which may
                       be an index or a ptr.  Either way it is simply copied.  There is no api to change
                       the string index, so we set it manually.  The other properties can be set with SetButtonInfo. */
                    free_string((*infoPtr).buttons.offset(idx as isize));
                    (*(*infoPtr).buttons.offset(idx as isize)).iString = tb.tbButton.iString;

                    memset(
                        &raw mut bi as *mut ::core::ffi::c_void,
                        0 as ::core::ffi::c_int,
                        ::core::mem::size_of::<TBBUTTONINFOW>() as size_t,
                    );
                    bi.cbSize = ::core::mem::size_of::<TBBUTTONINFOW>() as UINT;
                    bi.dwMask = (TBIF_IMAGE
                        | TBIF_STATE
                        | TBIF_STYLE
                        | TBIF_LPARAM) as DWORD;
                    bi.iImage = tb.tbButton.iBitmap;
                    bi.fsState = tb.tbButton.fsState;
                    bi.fsStyle = tb.tbButton.fsStyle;
                    bi.lParam = tb.tbButton.dwData;

                    TOOLBAR_SetButtonInfo(
                        infoPtr,
                        tb.tbButton.idCommand as INT,
                        &raw mut bi,
                        TRUE as BOOL,
                    );
                }
                i += 1;
            }
            TOOLBAR_SendNotify(&raw mut hdr, infoPtr, TBN_ENDADJUST as UINT);

            /* remove all uninitialised buttons
             * note: loop backwards to avoid having to fixup i on a
             * delete */
            i = ((*infoPtr).nNumButtons as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int) as INT;
            while i >= 0 as INT {
                if (*(*infoPtr).buttons.offset(i as isize)).iBitmap
                    == -(1 as ::core::ffi::c_int)
                {
                    TOOLBAR_DeleteButton(infoPtr, i);
                }
                i -= 1;
            }

            /* only indicate success if at least one button survived */
            if (*infoPtr).nNumButtons > 0 as ::core::ffi::c_int {
                ret = TRUE as BOOL;
            }
        }
    }
    Free(nmtbr.pData as LPVOID);
    RegCloseKey(hkey);

    return ret;
}

unsafe extern "C" fn TOOLBAR_SaveRestoreW(infoPtr: *mut TOOLBAR_INFO, wParam: WPARAM, lpSave: *const TBSAVEPARAMSW) -> LRESULT {
    if lpSave.is_null() {
        return 0;
    }

    if wParam != 0 {
        return TOOLBAR_Save(infoPtr, lpSave) as LRESULT;
    } else {
        return TOOLBAR_Restore(infoPtr, lpSave) as LRESULT;
    }
}

unsafe extern "C" fn TOOLBAR_SaveRestoreA(mut infoPtr: *mut TOOLBAR_INFO, mut wParam: WPARAM, mut lpSave: *const TBSAVEPARAMSA) -> LRESULT {
    let mut pszValueName: LPWSTR = core::ptr::null_mut();
    let mut pszSubKey: LPWSTR = core::ptr::null_mut();
    let mut SaveW: TBSAVEPARAMSW = core::mem::zeroed();
    let mut result: LRESULT = 0;
    let mut len: core::ffi::c_int;

    if lpSave.is_null() {
        return 0;
    }

    'exit: {
        len = MultiByteToWideChar(
            CP_ACP,
            0,
            (*lpSave).pszSubKey,
            -(1),
            core::ptr::null_mut(),
            0,
        );
        pszSubKey = Alloc(
            ((len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as DWORD,
        ) as LPWSTR;
        if pszSubKey.is_null() {
            break 'exit;
        }
        MultiByteToWideChar(
            CP_ACP,
            0,
            (*lpSave).pszSubKey,
            -(1),
            pszSubKey,
            len,
        );

        len = MultiByteToWideChar(
            CP_ACP,
            0,
            (*lpSave).pszValueName,
            -(1),
            core::ptr::null_mut(),
            0,
        );
        pszValueName = Alloc(
            ((len as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as DWORD,
        ) as LPWSTR;
        if pszValueName.is_null() {
            break 'exit;
        }
        MultiByteToWideChar(
            CP_ACP,
            0,
            (*lpSave).pszValueName,
            -(1),
            pszValueName,
            len,
        );

        SaveW.pszValueName = pszValueName as LPCWSTR;
        SaveW.pszSubKey = pszSubKey as LPCWSTR;
        SaveW.hkr = (*lpSave).hkr;
        result = TOOLBAR_SaveRestoreW(infoPtr, wParam, &mut SaveW as *mut TBSAVEPARAMSW);
    }

    Free(pszValueName as LPVOID);
    Free(pszSubKey as LPVOID);

    return result;
}

unsafe extern "C" fn TOOLBAR_SetAnchorHighlight(infoPtr: *mut TOOLBAR_INFO, bAnchor: BOOL) -> LRESULT {
    let mut bOldAnchor: BOOL = (*infoPtr).bAnchor;

    (*infoPtr).bAnchor = bAnchor;

    /* Native does not remove the hot effect from an already hot button */

    return bOldAnchor as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetBitmapSize(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut himlDef: HIMAGELIST = GETDEFIMAGELIST!(infoPtr, 0);
    let mut width: i16 = LOWORD!(lParam as DWORD) as i16;
    let mut height: i16 = HIWORD!(lParam as DWORD) as i16;

    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const i8,
            b"TOOLBAR_SetBitmapSize\0" as *const u8 as *const i8,
            4584i32,
            b"hwnd=%p, wParam=%ld, size %d x %d\n\0" as *const u8 as *const i8,
            (*infoPtr).hwndSelf,
            wParam,
            width as i32,
            height as i32,
        );
    }

    if wParam != 0 {
        if (*__wine_dbch___default).flags as i32
            & (1i32) << __WINE_DBCL_FIXME as i32
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const i8,
                b"TOOLBAR_SetBitmapSize\0" as *const u8 as *const i8,
                4587i32,
                b"wParam is %ld. Perhaps image list index?\n\0" as *const u8 as *const i8,
                wParam,
            );
        }
    }

    /* 0 width or height is changed to 1 */
    if width as i32 == 0 {
        width = 1 as i16;
    }
    if height as i32 == 0 {
        height = 1 as i16;
    }

    if (*infoPtr).nNumButtons > 0 {
        if (*__wine_dbch___default).flags as i32
            & (1i32) << __WINE_DBCL_TRACE as i32
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const i8,
                b"TOOLBAR_SetBitmapSize\0" as *const u8 as *const i8,
                4598i32,
                b"%d buttons, undoc change to bitmap size : %d-%d -> %d-%d\n\0"
                    as *const u8 as *const i8,
                (*infoPtr).nNumButtons,
                (*infoPtr).nBitmapWidth,
                (*infoPtr).nBitmapHeight,
                width as i32,
                height as i32,
            );
        }
    }

    if (width as i32) < -1 || (height as i32) < -1 {
        /* Windows destroys the imagelist and seems to actually use negative
         * values to compute button sizes */
        if (*__wine_dbch___default).flags as i32
            & (1i32) << __WINE_DBCL_FIXME as i32
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const i8,
                b"TOOLBAR_SetBitmapSize\0" as *const u8 as *const i8,
                4604i32,
                b"Negative bitmap sizes not supported (%d, %d)\n\0" as *const u8 as *const i8,
                width as i32,
                height as i32,
            );
        }
        return FALSE as LRESULT;
    }

    /* width or height of -1 means no change */
    if width as i32 != -1 {
        (*infoPtr).nBitmapWidth = width as INT;
    }
    if height as i32 != -1 {
        (*infoPtr).nBitmapHeight = height as INT;
    }

    if (himlDef == (*infoPtr).himlInt)
        && (ImageList_GetImageCount((*infoPtr).himlInt) == 0)
    {
        ImageList_SetIconSize(
            (*infoPtr).himlInt,
            (*infoPtr).nBitmapWidth,
            (*infoPtr).nBitmapHeight,
        );
    }

    TOOLBAR_CalcToolbar(infoPtr);
    InvalidateRect(
        (*infoPtr).hwndSelf,
        core::ptr::null_mut(),
        FALSE,
    );
    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetButtonInfo(
    infoPtr: *mut TOOLBAR_INFO,
    Id: INT,
    lptbbi: *const TBBUTTONINFOW,
    isW: BOOL,
) -> LRESULT {
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut();
    let mut nIndex: INT = 0;
    let mut oldBtnRect: RECT = core::mem::zeroed();

    if lptbbi.is_null() {
        return FALSE as LRESULT;
    }
    if ((*lptbbi).cbSize as usize) < core::mem::size_of::<TBBUTTONINFOW>() {
        return FALSE as LRESULT;
    }

    nIndex = TOOLBAR_GetButtonIndex(
        infoPtr,
        Id,
        ((*lptbbi).dwMask & (TBIF_BYINDEX as DWORD)) as BOOL,
    );
    if nIndex == -(1 as INT) {
        return FALSE as LRESULT;
    }

    btnPtr = (*infoPtr).buttons.offset(nIndex as isize);
    if (*lptbbi).dwMask & (TBIF_COMMAND as DWORD) != 0 {
        (*btnPtr).idCommand = (*lptbbi).idCommand as INT;
    }
    if (*lptbbi).dwMask & (TBIF_IMAGE as DWORD) != 0 {
        (*btnPtr).iBitmap = (*lptbbi).iImage as INT;
    }
    if (*lptbbi).dwMask & (TBIF_LPARAM as DWORD) != 0 {
        (*btnPtr).dwData = (*lptbbi).lParam;
    }
    if (*lptbbi).dwMask & (TBIF_SIZE as DWORD) != 0 {
        (*btnPtr).cx = (*lptbbi).cx as INT;
    }
    if (*lptbbi).dwMask & (TBIF_STATE as DWORD) != 0 {
        (*btnPtr).fsState = (*lptbbi).fsState;
    }
    if (*lptbbi).dwMask & (TBIF_STYLE as DWORD) != 0 {
        (*btnPtr).fsStyle = (*lptbbi).fsStyle;
    }

    if (*lptbbi).dwMask & (TBIF_TEXT as DWORD) != 0 {
        set_stringT(btnPtr, (*lptbbi).pszText, isW);
    }

    /* save the button rect to see if we need to redraw the whole toolbar */
    oldBtnRect = (*btnPtr).rect;
    TOOLBAR_LayoutToolbar(infoPtr);

    if EqualRect(
        core::ptr::addr_of_mut!(oldBtnRect),
        core::ptr::addr_of_mut!((*btnPtr).rect),
    ) == 0
    {
        InvalidateRect(
            (*infoPtr).hwndSelf,
            core::ptr::null(),
            TRUE,
        );
    } else {
        InvalidateRect(
            (*infoPtr).hwndSelf,
            core::ptr::addr_of_mut!((*btnPtr).rect),
            TRUE,
        );
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetButtonSize(infoPtr: *mut TOOLBAR_INFO, lParam: LPARAM) -> LRESULT {
    let mut cx: INT = LOWORD!(lParam as DWORD) as i16 as INT;
    let mut cy: INT = HIWORD!(lParam as DWORD) as i16 as INT;
    let mut top: core::ffi::c_int = default_top_margin(infoPtr);

    if (cx < 0) || (cy < 0) {
        return (FALSE) as LRESULT;
    }

    if cx == 0 {
        cx = 24;
    }
    if cy == 0 {
        cy = 22;
    }

    cx = max(
        cx,
        (*infoPtr).szPadding.cx
            + (*infoPtr).nBitmapWidth
            + (*infoPtr).themeMargins.cxLeftWidth
            + (*infoPtr).themeMargins.cxRightWidth,
    );
    cy = max(
        cy,
        (*infoPtr).szPadding.cy
            + (*infoPtr).nBitmapHeight
            + (*infoPtr).themeMargins.cyTopHeight
            + (*infoPtr).themeMargins.cyBottomHeight,
    );

    if cx != (*infoPtr).nButtonWidth
        || cy != (*infoPtr).nButtonHeight
        || top != (*infoPtr).iTopMargin
    {
        (*infoPtr).nButtonWidth = cx;
        (*infoPtr).nButtonHeight = cy;
        (*infoPtr).iTopMargin = top;

        TOOLBAR_LayoutToolbar(infoPtr);
        InvalidateRect(
            (*infoPtr).hwndSelf,
            core::ptr::null_mut(),
            TRUE,
        );
    }
    return (TRUE) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetButtonWidth(infoPtr: *mut TOOLBAR_INFO, lParam: LPARAM) -> LRESULT {
    /* if setting to current values, ignore */
    if ((*infoPtr).cxMin == (LOWORD!(lParam) as i16) as i32)
        && ((*infoPtr).cxMax == (HIWORD!(lParam) as i16) as i32)
    {
        return (TRUE) as LRESULT;
    }

    /* save new values */
    (*infoPtr).cxMin = (LOWORD!(lParam) as i16) as i32;
    (*infoPtr).cxMax = (HIWORD!(lParam) as i16) as i32;

    /* otherwise we need to recalc the toolbar and in some cases
       recalc the bounding rectangle (does DrawText w/ DT_CALCRECT
       which doesn't actually draw - GA). */

    TOOLBAR_CalcToolbar(infoPtr);

    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null(), TRUE);

    return (TRUE) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetCmdId(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut nIndex: INT,
    mut nId: INT,
) -> LRESULT {
    if nIndex < 0 as ::core::ffi::c_int || nIndex >= (*infoPtr).nNumButtons {
        return FALSE as LRESULT;
    }
    (*(*infoPtr).buttons.offset(nIndex as isize)).idCommand = nId;
    if !(*infoPtr).hwndToolTip.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_SetCmdId\0" as *const u8 as *const ::core::ffi::c_char,
                4763 as ::core::ffi::c_int,
                b"change tool tip\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    }
    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetDisabledImageList(
    infoPtr: *mut TOOLBAR_INFO,
    wParam: WPARAM,
    himl: HIMAGELIST,
) -> LRESULT {
    let mut himlTemp: HIMAGELIST = core::ptr::null_mut();
    let mut id: INT = 0;

    if (*infoPtr).iVersion >= 5 {
        id = wParam as INT;
    }

    himlTemp = TOOLBAR_InsertImageList(
        core::ptr::addr_of_mut!((*infoPtr).himlDis),
        core::ptr::addr_of_mut!((*infoPtr).cimlDis),
        himl,
        id,
    );

    /* FIXME: redraw ? */

    return himlTemp as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetDrawTextFlags(infoPtr: *mut TOOLBAR_INFO, mask: DWORD, flags: DWORD) -> LRESULT {
    let mut old_flags: DWORD;

    old_flags = (*infoPtr).dwDTFlags;
    (*infoPtr).dwDTFlags = (old_flags & !mask) | (flags & mask);

    return old_flags as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetExtendedStyle(
    infoPtr: *mut TOOLBAR_INFO,
    mask: DWORD,
    style: DWORD,
) -> LRESULT {
    let old_style: DWORD = (*infoPtr).dwExStyle;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_SetExtendedStyle\0" as *const u8 as *const ::core::ffi::c_char,
            4813 as ::core::ffi::c_int,
            b"mask=0x%08x, style=0x%08x\n\0" as *const u8 as *const ::core::ffi::c_char,
            mask,
            style,
        );
    }
    if mask != 0 {
        (*infoPtr).dwExStyle = (old_style & !mask) | (style & mask);
    } else {
        (*infoPtr).dwExStyle = style;
    }
    if (*infoPtr).dwExStyle & !(TBSTYLE_EX_ALL as DWORD) != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_SetExtendedStyle\0" as *const u8 as *const ::core::ffi::c_char,
                4822 as ::core::ffi::c_int,
                b"Unknown Toolbar Extended Style 0x%08x. Please report.\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (*infoPtr).dwExStyle & !(TBSTYLE_EX_ALL as DWORD),
            );
        }
    }
    if (old_style ^ (*infoPtr).dwExStyle) & (TBSTYLE_EX_MIXEDBUTTONS as DWORD) != 0 {
        TOOLBAR_CalcToolbar(infoPtr);
    } else {
        TOOLBAR_LayoutToolbar(infoPtr);
    }
    TOOLBAR_AutoSize(infoPtr);
    InvalidateRect(
        (*infoPtr).hwndSelf,
        ::core::ptr::null::<RECT>(),
        TRUE,
    );
    return old_style as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetHotImageList(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut wParam: WPARAM,
    mut himl: HIMAGELIST,
) -> LRESULT {
    let mut himlTemp: HIMAGELIST = core::ptr::null_mut();
    let mut id: INT = 0;
    if (*infoPtr).iVersion >= 5 {
        id = wParam as INT;
    }
    himlTemp = TOOLBAR_InsertImageList(
        core::ptr::addr_of_mut!((*infoPtr).himlHot),
        core::ptr::addr_of_mut!((*infoPtr).cimlHot),
        himl,
        id,
    );
    return himlTemp as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetHotItemEx(infoPtr: *mut TOOLBAR_INFO, nHit: INT, dwReason: DWORD) {
    if (*infoPtr).nHotItem != nHit {
        let mut nmhotitem: NMTBHOTITEM = core::mem::zeroed();
        let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut();
        let mut oldBtnPtr: *mut TBUTTON_INFO = core::ptr::null_mut();

        nmhotitem.dwFlags = dwReason;
        if (*infoPtr).nHotItem >= (0 as INT) {
            oldBtnPtr = (*infoPtr).buttons.offset((*infoPtr).nHotItem as isize) as *mut TBUTTON_INFO;
            nmhotitem.idOld = (*oldBtnPtr).idCommand as INT;
        } else {
            nmhotitem.dwFlags |= HICF_ENTERING as DWORD;
            nmhotitem.idOld = 0 as INT;
        }

        if nHit >= (0 as INT) {
            btnPtr = (*infoPtr).buttons.offset(nHit as isize) as *mut TBUTTON_INFO;
            nmhotitem.idNew = (*btnPtr).idCommand as INT;
        } else {
            nmhotitem.dwFlags |= HICF_LEAVING as DWORD;
            nmhotitem.idNew = 0 as INT;
        }

        /* now change the hot and invalidate the old and new buttons - if the
         * parent agrees */
        if TOOLBAR_SendNotify(
            core::ptr::addr_of_mut!(nmhotitem.hdr),
            infoPtr,
            TBN_HOTITEMCHANGE as UINT,
        ) == 0
        {
            if !oldBtnPtr.is_null() {
                (*oldBtnPtr).bHot = FALSE as BOOL;
                InvalidateRect(
                    (*infoPtr).hwndSelf,
                    core::ptr::addr_of_mut!((*oldBtnPtr).rect) as *const RECT,
                    TRUE as BOOL,
                );
            }
            /* setting disabled buttons as hot fails even if the notify contains the button id */
            if !btnPtr.is_null() && (((*btnPtr).fsState as INT) & (TBSTATE_ENABLED as INT)) != 0 {
                (*btnPtr).bHot = TRUE as BOOL;
                InvalidateRect(
                    (*infoPtr).hwndSelf,
                    core::ptr::addr_of_mut!((*btnPtr).rect) as *const RECT,
                    TRUE as BOOL,
                );
                (*infoPtr).nHotItem = nHit;
            } else {
                (*infoPtr).nHotItem = -(1 as INT);
            }
        }
    }
}

unsafe extern "C" fn TOOLBAR_SetHotItem(
    infoPtr: *mut TOOLBAR_INFO,
    mut nHotItem: INT,
) -> LRESULT {
    let mut nOldHotItem: INT = (*infoPtr).nHotItem;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_SetHotItem\0" as *const u8 as *const ::core::ffi::c_char,
            4918 as ::core::ffi::c_int,
            b"hwnd = %p, nHotItem = %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).hwndSelf,
            nHotItem,
        );
    }
    if nHotItem >= (*infoPtr).nNumButtons {
        return (*infoPtr).nHotItem as LRESULT;
    }
    if nHotItem < 0 {
        nHotItem = -(1) as INT;
    }
    TOOLBAR_SetHotItemEx(infoPtr, nHotItem, HICF_OTHER as DWORD);
    if nOldHotItem < 0 {
        return -(1) as LRESULT;
    }
    return nOldHotItem as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetImageList(infoPtr: *mut TOOLBAR_INFO, wParam: WPARAM, himl: HIMAGELIST) -> LRESULT {
    let mut himlTemp: HIMAGELIST = core::ptr::null_mut();
    let mut oldButtonWidth: INT = (*infoPtr).nButtonWidth;
    let mut oldBitmapWidth: INT = (*infoPtr).nBitmapWidth;
    let mut oldBitmapHeight: INT = (*infoPtr).nBitmapHeight;
    let mut i: INT = 0;
    let mut id: INT = 0;

    if (*infoPtr).iVersion >= 5 {
        id = wParam as INT;
    }

    himlTemp = TOOLBAR_InsertImageList(
        core::ptr::addr_of_mut!((*infoPtr).himlDef),
        core::ptr::addr_of_mut!((*infoPtr).cimlDef),
        himl,
        id,
    );

    (*infoPtr).nNumBitmaps = 0;
    i = 0;
    while i < (*infoPtr).cimlDef {
        (*infoPtr).nNumBitmaps += ImageList_GetImageCount((**(*infoPtr).himlDef.offset(i as isize)).himl);
        i += 1;
    }

    if ImageList_GetIconSize(
        himl,
        core::ptr::addr_of_mut!((*infoPtr).nBitmapWidth),
        core::ptr::addr_of_mut!((*infoPtr).nBitmapHeight),
    ) == 0
    {
        (*infoPtr).nBitmapWidth = 1;
        (*infoPtr).nBitmapHeight = 1;
    }
    if (oldBitmapWidth != (*infoPtr).nBitmapWidth) || (oldBitmapHeight != (*infoPtr).nBitmapHeight) {
        TOOLBAR_CalcToolbar(infoPtr);
        if (*infoPtr).nButtonWidth < oldButtonWidth {
            TOOLBAR_SetButtonSize(
                infoPtr,
                MAKELONG!(oldButtonWidth, (*infoPtr).nButtonHeight) as LPARAM,
            );
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
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_SetImageList\0" as *const u8 as *const ::core::ffi::c_char,
            4972 as ::core::ffi::c_int,
            b"hwnd %p, new himl=%p, id = %d, count=%d, bitmap w=%d, h=%d\n\0"
                as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).hwndSelf,
            (*infoPtr).himlDef,
            id,
            (*infoPtr).nNumBitmaps,
            (*infoPtr).nBitmapWidth,
            (*infoPtr).nBitmapHeight,
        );
    }

    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null(), TRUE);

    return himlTemp as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetIndent(infoPtr: *mut TOOLBAR_INFO, nIndent: INT) -> LRESULT {
    (*infoPtr).nIndent = nIndent;

    /* process only on indent changing */
    if (*infoPtr).nIndent != nIndent {
        (*infoPtr).nIndent = nIndent;
        TOOLBAR_CalcToolbar(infoPtr);
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), FALSE);
    }

    return (TRUE) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetInsertMark(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut lptbim: *const TBINSERTMARK,
) -> LRESULT {
    if ((*lptbim).dwFlags & !TBIMHT_AFTER) != 0 {
        return 0;
    }

    if ((*lptbim).iButton == -1)
        || (((*lptbim).iButton < (*infoPtr).nNumButtons)
            && ((*lptbim).iButton >= 0))
    {
        (*infoPtr).tbim = *lptbim;
        /* FIXME: don't need to update entire toolbar */
        InvalidateRect(
            (*infoPtr).hwndSelf,
            core::ptr::null::<RECT>(),
            TRUE,
        );
    } else if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_SetInsertMark\0" as *const u8 as *const ::core::ffi::c_char,
            5019 as ::core::ffi::c_int,
            b"Invalid button index %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*lptbim).iButton,
        );
    }

    return 0;
}

unsafe extern "C" fn TOOLBAR_SetInsertMarkColor(infoPtr: *mut TOOLBAR_INFO, clr: COLORREF) -> LRESULT {
    (*infoPtr).clrInsertMark = clr;

    /* FIXME: don't need to update entire toolbar */
    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE);

    return 0;
}

unsafe extern "C" fn TOOLBAR_SetMaxTextRows(mut infoPtr: *mut TOOLBAR_INFO, mut nMaxRows: INT) -> LRESULT {
    (*infoPtr).nMaxTextRows = nMaxRows;
    TOOLBAR_CalcToolbar(infoPtr);
    return (TRUE) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetMetrics(mut infoPtr: *mut TOOLBAR_INFO, mut pMetrics: *mut TBMETRICS) -> LRESULT {
    let mut changed: BOOL = FALSE;

    if pMetrics.is_null()
        || (*pMetrics).cbSize as usize != ::core::mem::size_of::<TBMETRICS>() as usize
    {
        return 0 as LRESULT;
    }

    if ((*pMetrics).dwMask & TBMF_PAD as DWORD) != 0 {
        (*infoPtr).szPadding.cx = (*pMetrics).cxPad as LONG;
        (*infoPtr).szPadding.cy = (*pMetrics).cyPad as LONG;
        changed = TRUE as BOOL;
    }

    if ((*pMetrics).dwMask & TBMF_BARPAD as DWORD) != 0 {
        (*infoPtr).szBarPadding.cx = (*pMetrics).cxBarPad as LONG;
        (*infoPtr).szBarPadding.cy = (*pMetrics).cyBarPad as LONG;
        changed = TRUE as BOOL;
    }

    if ((*pMetrics).dwMask & TBMF_BUTTONSPACING as DWORD) != 0 {
        (*infoPtr).szSpacing.cx = (*pMetrics).cxButtonSpacing as LONG;
        (*infoPtr).szSpacing.cy = (*pMetrics).cyButtonSpacing as LONG;
        changed = TRUE as BOOL;
    }

    if changed != 0 {
        TOOLBAR_CalcToolbar(infoPtr);
    }

    // TB_SETMETRICS's return value is ignored (upstream 9b120aa796e).
    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetPadding(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut oldPad: DWORD = 0;
    oldPad = MAKELONG!(
        (*infoPtr).szPadding.cx as WORD,
        (*infoPtr).szPadding.cy as WORD,
    ) as DWORD;
    (*infoPtr).szPadding.cx = min(
        LOWORD!(lParam as DWORD) as ::core::ffi::c_int,
        GetSystemMetrics((SM_CXEDGE) as i32),
    ) as LONG;
    (*infoPtr).szPadding.cy = min(
        HIWORD!(lParam as DWORD) as ::core::ffi::c_int,
        GetSystemMetrics((SM_CYEDGE) as i32),
    ) as LONG;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_SetPadding\0" as *const u8 as *const ::core::ffi::c_char,
            5105 as ::core::ffi::c_int,
            b"cx=%d, cy=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).szPadding.cx,
            (*infoPtr).szPadding.cy,
        );
    }
    return oldPad as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetParent(
    infoPtr: *mut TOOLBAR_INFO,
    hParent: HWND,
) -> LRESULT {
    let mut hwndOldNotify: HWND = core::ptr::null_mut::<HWND__>();
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"TOOLBAR_SetParent\0" as *const u8 as *const core::ffi::c_char,
            5115 as core::ffi::c_int,
            b"\n\0" as *const u8 as *const core::ffi::c_char,
        );
    }
    hwndOldNotify = (*infoPtr).hwndNotify;
    (*infoPtr).hwndNotify = hParent;
    return hwndOldNotify as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetRows(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut wParam: WPARAM,
    mut lprc: LPRECT,
) -> LRESULT {
    let mut rows: ::core::ffi::c_int = LOWORD!(wParam) as ::core::ffi::c_int;
    let mut bLarger: BOOL = HIWORD!(wParam) as BOOL;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_SetRows\0" as *const u8 as *const ::core::ffi::c_char,
            5130 as ::core::ffi::c_int,
            b"\n\0" as *const u8 as *const ::core::ffi::c_char,
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
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_SetRows\0" as *const u8 as *const ::core::ffi::c_char,
            5132 as ::core::ffi::c_int,
            b"Setting rows to %d (%d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            rows,
            bLarger,
        );
    }
    if (*infoPtr).nRows != rows {
        let mut btnPtr: *mut TBUTTON_INFO = (*infoPtr).buttons;
        let mut curColumn: ::core::ffi::c_int = 0;
        let mut curRow: ::core::ffi::c_int = 0;
        let mut hidden: ::core::ffi::c_int = 0;
        let mut seps: ::core::ffi::c_int = 0;
        let mut idealWrap: ::core::ffi::c_int = 0;
        let mut i: ::core::ffi::c_int = 0;
        let mut wrap: BOOL = 0;
        i = 0;
        while i < (*infoPtr).nNumButtons {
            let ref mut fresh27 = (*btnPtr.offset(i as isize)).fsState;
            *fresh27 = (*fresh27 as ::core::ffi::c_int & (!TBSTATE_WRAP) as i32) as BYTE;
            if (*btnPtr.offset(i as isize)).fsState as ::core::ffi::c_int
                & (TBSTATE_HIDDEN as i32)
                != 0
            {
                hidden += 1;
            } else if (*btnPtr.offset(i as isize)).fsStyle as ::core::ffi::c_int
                & (BTNS_SEP as i32)
                != 0
            {
                seps += 1;
            }
            i += 1;
        }
        if seps != 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/toolbar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TOOLBAR_SetRows\0" as *const u8 as *const ::core::ffi::c_char,
                    5169 as ::core::ffi::c_int,
                    b"Separators unhandled\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
        }
        idealWrap = ((*infoPtr).nNumButtons as ::core::ffi::c_int - hidden
            + (rows - 1))
            / (if rows != 0 {
                rows
            } else {
                1
            });
        if bLarger != 0 && idealWrap > 1 {
            let mut resRows: ::core::ffi::c_int = ((*infoPtr).nNumButtons as ::core::ffi::c_int
                + (idealWrap - 1))
                / idealWrap;
            let mut moreRows: ::core::ffi::c_int = ((*infoPtr).nNumButtons as ::core::ffi::c_int
                + (idealWrap - 2))
                / (idealWrap - 1);
            if resRows < rows && moreRows > rows {
                idealWrap -= 1;
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_2,
                        __dbch_2,
                        b"dll/win32/comctl32/toolbar.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"TOOLBAR_SetRows\0" as *const u8 as *const ::core::ffi::c_char,
                        5184 as ::core::ffi::c_int,
                        b"Changing idealWrap due to bLarger (now %d)\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        idealWrap,
                    );
                }
            }
        }
        curColumn = 0;
        curRow = curColumn;
        wrap = FALSE as BOOL;
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_3,
                __dbch_3,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_SetRows\0" as *const u8 as *const ::core::ffi::c_char,
                5191 as ::core::ffi::c_int,
                b"Trying to wrap at %d (%d,%d,%d)\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                idealWrap,
                (*infoPtr).nNumButtons,
                hidden,
                rows,
            );
        }
        i = 0;
        while i < (*infoPtr).nNumButtons {
            if !((*btnPtr.offset(i as isize)).fsState as ::core::ffi::c_int
                & (TBSTATE_HIDDEN as i32)
                != 0)
            {
                if wrap == 0 {
                    curColumn += 1;
                } else {
                    wrap = FALSE as BOOL;
                    curColumn = 1;
                    curRow += 1;
                }
                if curColumn > idealWrap - 1 {
                    wrap = TRUE as BOOL;
                    let ref mut fresh28 = (*btnPtr.offset(i as isize)).fsState;
                    *fresh28 = (*fresh28 as ::core::ffi::c_int | (TBSTATE_WRAP as i32)) as BYTE;
                }
            }
            i += 1;
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_4: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_4,
                __dbch_4,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_SetRows\0" as *const u8 as *const ::core::ffi::c_char,
                5213 as ::core::ffi::c_int,
                b"Result - %d rows\n\0" as *const u8 as *const ::core::ffi::c_char,
                curRow + 1 as ::core::ffi::c_int,
            );
        }
        TOOLBAR_CalcToolbar(infoPtr);
        if (*infoPtr).dwStyle as ::core::ffi::c_ulong
            & CCS_NORESIZE as ::core::ffi::c_ulong
            != 0
        {
            SetWindowPos(
                (*infoPtr).hwndSelf,
                ::core::ptr::null_mut::<HWND__>(),
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (*infoPtr).rcBound.right as ::core::ffi::c_int
                    - (*infoPtr).rcBound.left as ::core::ffi::c_int,
                (*infoPtr).rcBound.bottom as ::core::ffi::c_int
                    - (*infoPtr).rcBound.top as ::core::ffi::c_int,
                (SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE) as UINT,
            );
        }
        InvalidateRect(
            (*infoPtr).hwndSelf,
            ::core::ptr::null::<RECT>(),
            TRUE,
        );
    }
    if !lprc.is_null() {
        (*lprc).left = (*infoPtr).rcBound.left;
        (*lprc).right = (*infoPtr).rcBound.right;
        (*lprc).top = (*infoPtr).rcBound.top;
        (*lprc).bottom = (*infoPtr).rcBound.bottom;
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetState(infoPtr: *mut TOOLBAR_INFO, Id: INT, lParam: LPARAM) -> LRESULT {
    let mut btnPtr: *mut TBUTTON_INFO;
    let mut nIndex: INT;

    nIndex = TOOLBAR_GetButtonIndex(infoPtr, Id, FALSE);
    if nIndex == -1 {
        return FALSE as LRESULT;
    }

    btnPtr = (*infoPtr).buttons.offset(nIndex as isize);

    /* if hidden state has changed the invalidate entire window and recalc */
    if ((*btnPtr).fsState as INT & (TBSTATE_HIDDEN as i32)) != (LOWORD!(lParam) as INT & (TBSTATE_HIDDEN as i32)) {
        (*btnPtr).fsState = LOWORD!(lParam) as BYTE;
        TOOLBAR_CalcToolbar(infoPtr);
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null(), TRUE);
        return TRUE as LRESULT;
    }

    /* process state changing if current state doesn't match new state */
    if (*btnPtr).fsState as INT != LOWORD!(lParam) as INT {
        (*btnPtr).fsState = LOWORD!(lParam) as BYTE;
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!((*btnPtr).rect), TRUE);
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn unwrap(info: *mut TOOLBAR_INFO) {
    let mut i: INT;
    i = 0;
    while i < (*info).nNumButtons {
        (*(*info).buttons.offset(i as isize)).fsState &= (!(TBSTATE_WRAP)) as BYTE;
        i += 1;
    }
}

unsafe extern "C" fn TOOLBAR_SetStyle(infoPtr: *mut TOOLBAR_INFO, style: DWORD) -> LRESULT {
    let dwOldStyle: DWORD = (*infoPtr).dwStyle;

    if style & (TBSTYLE_LIST as DWORD) != 0 {
        (*infoPtr).dwDTFlags = ((DT_LEFT as DWORD) | (DT_VCENTER as DWORD) | (DT_SINGLELINE as DWORD) | (DT_END_ELLIPSIS as DWORD));
    } else {
        (*infoPtr).dwDTFlags = ((DT_CENTER as DWORD) | (DT_END_ELLIPSIS as DWORD));
    }

    (*infoPtr).dwStyle = style;
    TOOLBAR_CheckStyle(infoPtr);

    if ((dwOldStyle ^ style) & (TBSTYLE_WRAPABLE as DWORD)) != 0 {
        if dwOldStyle & (TBSTYLE_WRAPABLE as DWORD) != 0 {
            unwrap(infoPtr);
        }
        TOOLBAR_CalcToolbar(infoPtr);
    } else if ((dwOldStyle ^ style) & (CCS_VERT as DWORD)) != 0 {
        TOOLBAR_LayoutToolbar(infoPtr);
    }

    /* only resize if one of the CCS_* styles was changed */
    if ((dwOldStyle ^ style) & (COMMON_STYLES as DWORD)) != 0 {
        TOOLBAR_AutoSize(infoPtr);
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE as BOOL);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetToolTips(infoPtr: *mut TOOLBAR_INFO, hwndTooltip: HWND) -> LRESULT {
    (*infoPtr).hwndToolTip = hwndTooltip;
    return 0;
}

unsafe extern "C" fn TOOLBAR_SetUnicodeFormat(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut wParam: WPARAM,
) -> LRESULT {
    let mut bTemp: BOOL = 0;
    bTemp = (*infoPtr).bUnicode;
    (*infoPtr).bUnicode = wParam as BOOL;
    return bTemp as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetColorScheme(infoPtr: *const TOOLBAR_INFO, lParam: LPCOLORSCHEME) -> LRESULT {
    (*lParam).clrBtnHighlight = if (*infoPtr).clrBtnHighlight == CLR_DEFAULT {
        comctl32_color.clrBtnHighlight
    } else {
        (*infoPtr).clrBtnHighlight
    };
    (*lParam).clrBtnShadow = if (*infoPtr).clrBtnShadow == CLR_DEFAULT {
        comctl32_color.clrBtnShadow
    } else {
        (*infoPtr).clrBtnShadow
    };
    return 1;
}

unsafe extern "C" fn TOOLBAR_SetColorScheme(infoPtr: *mut TOOLBAR_INFO, lParam: *const COLORSCHEME) -> LRESULT {
    (*infoPtr).clrBtnHighlight = (*lParam).clrBtnHighlight;
    (*infoPtr).clrBtnShadow = (*lParam).clrBtnShadow;
    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE);
    return 0;
}

unsafe extern "C" fn TOOLBAR_SetVersion(infoPtr: *mut TOOLBAR_INFO, iVersion: INT) -> LRESULT {
    let mut iOldVersion: INT = (*infoPtr).iVersion;

    /* The v6 control doesn't support changing its version */
    if iOldVersion == 6 {
        return iOldVersion as LRESULT;
    }

    /* And a control that is not v6 can't be set to be a v6 one */
    if iVersion >= 6 {
        return -(1) as LRESULT;
    }

    (*infoPtr).iVersion = iVersion;

    if (*infoPtr).iVersion >= 5 {
        TOOLBAR_SetUnicodeFormat(infoPtr, TRUE as WPARAM);
    }

    return iOldVersion as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetStringA(
    infoPtr: *const TOOLBAR_INFO,
    wParam: WPARAM,
    str: LPSTR,
) -> LRESULT {
    let iString: WORD = HIWORD!(wParam);
    let buffersize: WORD = LOWORD!(wParam);
    let mut ret: LRESULT = -(1) as LRESULT;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_GetStringA\0" as *const u8 as *const ::core::ffi::c_char,
            5399 as ::core::ffi::c_int,
            b"hwnd=%p, iString=%d, buffersize=%d, string=%p\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*infoPtr).hwndSelf,
            iString as ::core::ffi::c_int,
            buffersize as ::core::ffi::c_int,
            str,
        );
    }
    if (iString as ::core::ffi::c_int) < (*infoPtr).nNumStrings {
        ret = WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            *(*infoPtr).strings.offset(iString as isize) as LPCWSTR,
            -(1) as ::core::ffi::c_int,
            str,
            buffersize as ::core::ffi::c_int,
            ::core::ptr::null::<CHAR>(),
            ::core::ptr::null_mut::<BOOL>(),
        ) as LRESULT;
        ret -= 1;
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_GetStringA\0" as *const u8 as *const ::core::ffi::c_char,
                5406 as ::core::ffi::c_int,
                b"returning %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_a(str as *const ::core::ffi::c_char),
            );
        }
    } else if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_WARN;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_GetStringA\0" as *const u8 as *const ::core::ffi::c_char,
            5409 as ::core::ffi::c_int,
            b"String index %d out of range (largest is %d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            iString as ::core::ffi::c_int,
            (*infoPtr).nNumStrings as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        );
    }
    return ret;
}

unsafe extern "C" fn TOOLBAR_GetStringW(
    infoPtr: *const TOOLBAR_INFO,
    wParam: WPARAM,
    str: LPWSTR,
) -> LRESULT {
    let mut iString: WORD = HIWORD!(wParam);
    let mut len: WORD = (LOWORD!(wParam) as usize)
        .wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
        .wrapping_sub(1 as usize) as WORD;
    let mut ret: LRESULT = -(1 as LRESULT);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_GetStringW\0" as *const u8 as *const ::core::ffi::c_char,
            5422 as ::core::ffi::c_int,
            b"hwnd=%p, iString=%d, buffersize=%d, string=%p\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*infoPtr).hwndSelf,
            iString as ::core::ffi::c_int,
            LOWORD!(wParam) as ::core::ffi::c_int,
            str,
        );
    }
    if (iString as ::core::ffi::c_int) < (*infoPtr).nNumStrings {
        let lim: ::core::ffi::c_int = lstrlenW(
            *(*infoPtr).strings.offset(iString as isize) as LPCWSTR,
        );
        len = min(len as ::core::ffi::c_int, lim) as WORD;
        ret = ((len as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as LRESULT;
        if !str.is_null() {
            let _ = memcpy(
                str as *mut ::core::ffi::c_void,
                *(*infoPtr).strings.offset(iString as isize) as *const ::core::ffi::c_void,
                ret as size_t,
            );
            *str.offset(len as isize) = '\0' as i32 as WCHAR;
        }
        ret = len as LRESULT;
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_GetStringW\0" as *const u8 as *const ::core::ffi::c_char,
                5435 as ::core::ffi::c_int,
                b"returning %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w(str as *const WCHAR),
            );
        }
    } else if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_WARN;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_GetStringW\0" as *const u8 as *const ::core::ffi::c_char,
            5438 as ::core::ffi::c_int,
            b"String index %d out of range (largest is %d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            iString as ::core::ffi::c_int,
            (*infoPtr).nNumStrings as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
        );
    }
    return ret;
}

unsafe extern "C" fn TOOLBAR_SetBoundingSize(
    mut hwnd: HWND,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut pSize: *mut SIZE = lParam as *mut SIZE;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_SetBoundingSize\0" as *const u8 as *const ::core::ffi::c_char,
            5446 as ::core::ffi::c_int,
            b"hwnd=%p, wParam=0x%08lx, size.cx=%d, size.cy=%d stub\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hwnd,
            wParam,
            (*pSize).cx,
            (*pSize).cy,
        );
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetHotItem2(
    infoPtr: *mut TOOLBAR_INFO,
    mut nHotItem: INT,
    lParam: LPARAM,
) -> LRESULT {
    let nOldHotItem: INT = (*infoPtr).nHotItem;

    if nHotItem < 0 || nHotItem > (*infoPtr).nNumButtons {
        nHotItem = -1;
    }

    /* NOTE: an application can still remove the hot item even if anchor
     * highlighting is enabled */

    TOOLBAR_SetHotItemEx(infoPtr, nHotItem, lParam as DWORD);

    return if nOldHotItem < 0 {
        (-1) as LRESULT
    } else {
        nOldHotItem as LRESULT
    };
}

unsafe extern "C" fn TOOLBAR_SetListGap(infoPtr: *mut TOOLBAR_INFO, iListGap: INT) -> LRESULT {
    (*infoPtr).iListGap = iListGap;

    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE);

    return 0;
}

unsafe extern "C" fn TOOLBAR_GetImageListCount(infoPtr: *const TOOLBAR_INFO) -> LRESULT {
    return max((*infoPtr).cimlDef, max((*infoPtr).cimlHot, (*infoPtr).cimlDis)) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetIdealSize(
    infoPtr: *const TOOLBAR_INFO,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let lpsize: LPSIZE = lParam as LPSIZE;

    if lpsize.is_null() {
        return FALSE as LRESULT;
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
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_GetIdealSize\0" as *const u8 as *const ::core::ffi::c_char,
            5511 as ::core::ffi::c_int,
            b"wParam %ld, lParam 0x%08lx -> 0x%08x 0x%08x\n\0" as *const u8
                as *const ::core::ffi::c_char,
            wParam,
            lParam,
            (*lpsize).cx,
            (*lpsize).cy,
        );
    }

    match wParam {
        0 => {
            if (*lpsize).cx == -(1) {
                (*lpsize).cx = (*infoPtr).rcBound.right - (*infoPtr).rcBound.left;
            } else if HIWORD!((*lpsize).cx as DWORD) != 0 {
                let mut rc: RECT = RECT {
                    left: 0,
                    top: 0,
                    right: 0,
                    bottom: 0,
                };
                let hwndParent: HWND = GetParent((*infoPtr).hwndSelf);

                GetWindowRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rc));
                MapWindowPoints(
                    core::ptr::null_mut(),
                    hwndParent,
                    core::ptr::addr_of_mut!(rc) as LPPOINT,
                    2,
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
                        b"dll/win32/comctl32/toolbar.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"TOOLBAR_GetIdealSize\0" as *const u8
                            as *const ::core::ffi::c_char,
                        5524 as ::core::ffi::c_int,
                        b"mapped to (%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
                        wine_dbgstr_rect(core::ptr::addr_of_mut!(rc)),
                    );
                }

                (*lpsize).cx = max(
                    rc.right - rc.left,
                    (*infoPtr).rcBound.right - (*infoPtr).rcBound.left,
                );
            } else {
                (*lpsize).cx = (*infoPtr).rcBound.right - (*infoPtr).rcBound.left;
            }
        }
        1 => {
            (*lpsize).cy = (*infoPtr).rcBound.bottom - (*infoPtr).rcBound.top;
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/toolbar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TOOLBAR_GetIdealSize\0" as *const u8 as *const ::core::ffi::c_char,
                    5536 as ::core::ffi::c_int,
                    b"Unknown wParam %ld\n\0" as *const u8 as *const ::core::ffi::c_char,
                    wParam,
                );
            }
            return 0 as LRESULT;
        }
    }

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_2,
            __dbch_2,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_GetIdealSize\0" as *const u8 as *const ::core::ffi::c_char,
            5540 as ::core::ffi::c_int,
            b"set to -> 0x%08x 0x%08x\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*lpsize).cx,
            (*lpsize).cy,
        );
    }

    return 1 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_Unkwn464(hwnd: HWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let _ = (wParam, lParam);
    InvalidateRect(hwnd, core::ptr::null_mut(), TRUE);
    return 1;
}

unsafe extern "C" fn TOOLBAR_Create(mut hwnd: HWND, mut lpcs: *const CREATESTRUCTW) -> LRESULT {
    let mut infoPtr: *mut TOOLBAR_INFO =
        GetWindowLongPtrW(hwnd, 0 as ::core::ffi::c_int) as *mut TOOLBAR_INFO;
    let mut logFont: LOGFONTW = core::mem::zeroed();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_Create\0" as *const u8 as *const ::core::ffi::c_char,
            5559 as ::core::ffi::c_int,
            b"hwnd = %p, style=0x%08x\n\0" as *const u8 as *const ::core::ffi::c_char,
            hwnd,
            (*lpcs).style,
        );
    }
    (*infoPtr).dwStyle = GetWindowLongW(hwnd, GWL_STYLE) as DWORD;
    GetClientRect(hwnd, core::ptr::addr_of_mut!((*infoPtr).client_rect));
    (*infoPtr).bUnicode = ((!(*infoPtr).hwndNotify.is_null())
        && (NFR_UNICODE as LRESULT
            == SendMessageW(
                hwnd,
                WM_NOTIFYFORMAT as UINT,
                hwnd as WPARAM,
                NF_REQUERY as LPARAM,
            ))) as ::core::ffi::c_int as BOOL;
    (*infoPtr).hwndToolTip = core::ptr::null_mut::<HWND__>();
    SystemParametersInfoW(
        SPI_GETICONTITLELOGFONT as UINT,
        0 as UINT,
        core::ptr::addr_of_mut!(logFont) as PVOID,
        0 as UINT,
    );
    (*infoPtr).hDefaultFont = CreateFontIndirectW(core::ptr::addr_of_mut!(logFont));
    (*infoPtr).hFont = (*infoPtr).hDefaultFont;
    {
        let theme: HTHEME = OpenThemeData(hwnd, core::ptr::addr_of!(themeClass) as LPCWSTR);
        if !theme.is_null() {
            GetThemeMargins(
                theme,
                core::ptr::null_mut::<HDC__>(),
                TP_BUTTON as ::core::ffi::c_int,
                TS_NORMAL as ::core::ffi::c_int,
                (TMT_CONTENTMARGINS as i32),
                core::ptr::null_mut::<RECT>(),
                core::ptr::addr_of_mut!((*infoPtr).themeMargins),
            );
        }
    }
    TOOLBAR_CheckStyle(infoPtr);
    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_Destroy(infoPtr: *mut TOOLBAR_INFO) -> LRESULT {
    let mut i: INT = 0;
    /* delete tooltip control */
    if !(*infoPtr).hwndToolTip.is_null() {
        DestroyWindow((*infoPtr).hwndToolTip);
    }
    /* delete temporary buffer for tooltip text */
    Free((*infoPtr).pszTooltipText as LPVOID);
    Free((*infoPtr).bitmaps as LPVOID);
    /* delete button data */
    i = 0;
    while i < (*infoPtr).nNumButtons {
        free_string((*infoPtr).buttons.offset(i as isize));
        i += 1;
    }
    Free((*infoPtr).buttons as LPVOID);
    /* delete strings */
    if !(*infoPtr).strings.is_null() {
        i = 0;
        while i < (*infoPtr).nNumStrings {
            Free(*(*infoPtr).strings.offset(i as isize) as LPVOID);
            i += 1;
        }
        Free((*infoPtr).strings as LPVOID);
    }
    /* destroy internal image list */
    if !(*infoPtr).himlInt.is_null() {
        ImageList_Destroy((*infoPtr).himlInt);
    }
    TOOLBAR_DeleteImageList(&raw mut (*infoPtr).himlDef, &raw mut (*infoPtr).cimlDef);
    TOOLBAR_DeleteImageList(&raw mut (*infoPtr).himlDis, &raw mut (*infoPtr).cimlDis);
    TOOLBAR_DeleteImageList(&raw mut (*infoPtr).himlHot, &raw mut (*infoPtr).cimlHot);
    /* delete default font */
    DeleteObject((*infoPtr).hDefaultFont as HGDIOBJ);
    CloseThemeData(GetWindowTheme((*infoPtr).hwndSelf));
    /* free toolbar info data */
    SetWindowLongPtrW((*infoPtr).hwndSelf, 0, 0);
    Free(infoPtr as LPVOID);
    return 0;
}

unsafe extern "C" fn TOOLBAR_EraseBackground(
    infoPtr: *mut TOOLBAR_INFO,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut tbcd: NMTBCUSTOMDRAW = core::mem::zeroed();
    let mut ret: INT = FALSE;
    let mut ntfret: DWORD;
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    let mut dwEraseCustDraw: DWORD = 0;

    /* the app has told us not to redraw the toolbar */
    if (*infoPtr).bDoRedraw == 0 {
        return FALSE as LRESULT;
    }

    if (*infoPtr).dwStyle & TBSTYLE_CUSTOMERASE as DWORD != 0 {
        ZeroMemory!(
            core::ptr::addr_of_mut!(tbcd) as *mut core::ffi::c_void,
            core::mem::size_of::<NMTBCUSTOMDRAW>() as usize,
        );
        tbcd.nmcd.dwDrawStage = CDDS_PREERASE as DWORD;
        tbcd.nmcd.hdc = wParam as HDC;
        ntfret = TOOLBAR_SendNotify(
            core::ptr::addr_of_mut!(tbcd.nmcd.hdr),
            infoPtr,
            NM_CUSTOMDRAW as UINT,
        ) as DWORD;
        dwEraseCustDraw = ntfret & 0xffff as DWORD;

        /* FIXME: in general the return flags *can* be or'ed together */
        match dwEraseCustDraw {
            CDRF_DODEFAULT => {}
            CDRF_SKIPDEFAULT => {
                return TRUE as LRESULT;
            }
            _ => {
                if (*__wine_dbch___default).flags as core::ffi::c_int
                    & (1 as core::ffi::c_int) << __WINE_DBCL_FIXME as core::ffi::c_int
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/comctl32/toolbar.c\0"
                            as *const u8 as *const core::ffi::c_char,
                        b"TOOLBAR_EraseBackground\0" as *const u8 as *const core::ffi::c_char,
                        5662 as core::ffi::c_int,
                        b"[%p] response %d not handled to NM_CUSTOMDRAW (CDDS_PREERASE)\n\0"
                            as *const u8 as *const core::ffi::c_char,
                        (*infoPtr).hwndSelf,
                        ntfret,
                    );
                }
            }
        }
    }

    /* If the toolbar is "transparent" then pass the WM_ERASEBKGND up
     * to my parent for processing.
     */
    if !theme.is_null() || (*infoPtr).dwStyle & TBSTYLE_TRANSPARENT as DWORD != 0 {
        let mut pt: POINT = POINT { x: 0, y: 0 };
        let mut ptorig: POINT = POINT { x: 0, y: 0 };
        let hdc: HDC = wParam as HDC;
        let mut parent: HWND = core::ptr::null_mut();

        pt.x = 0 as LONG;
        pt.y = 0 as LONG;
        parent = GetParent((*infoPtr).hwndSelf);
        MapWindowPoints(
            (*infoPtr).hwndSelf,
            parent,
            core::ptr::addr_of_mut!(pt),
            1 as UINT,
        );
        OffsetWindowOrgEx(hdc, pt.x, pt.y, core::ptr::addr_of_mut!(ptorig));
        ret = SendMessageW(
            parent,
            WM_ERASEBKGND as UINT,
            wParam,
            lParam,
        ) as INT;
        SetWindowOrgEx(
            hdc,
            ptorig.x,
            ptorig.y,
            core::ptr::null_mut(),
        );
    }
    if ret == 0 {
        ret = DefWindowProcW(
            (*infoPtr).hwndSelf,
            WM_ERASEBKGND as UINT,
            wParam,
            lParam,
        ) as INT;
    }

    if dwEraseCustDraw & CDRF_NOTIFYPOSTERASE as DWORD != 0 {
        ZeroMemory!(
            core::ptr::addr_of_mut!(tbcd) as *mut core::ffi::c_void,
            core::mem::size_of::<NMTBCUSTOMDRAW>() as usize,
        );
        tbcd.nmcd.dwDrawStage = CDDS_POSTERASE as DWORD;
        tbcd.nmcd.hdc = wParam as HDC;
        ntfret = TOOLBAR_SendNotify(
            core::ptr::addr_of_mut!(tbcd.nmcd.hdr),
            infoPtr,
            NM_CUSTOMDRAW as UINT,
        ) as DWORD;
        dwEraseCustDraw = ntfret & 0xffff as DWORD;
        match dwEraseCustDraw {
            CDRF_DODEFAULT => {}
            CDRF_SKIPDEFAULT => {
                return TRUE as LRESULT;
            }
            _ => {
                if (*__wine_dbch___default).flags as core::ffi::c_int
                    & (1 as core::ffi::c_int) << __WINE_DBCL_FIXME as core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_FIXME;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/comctl32/toolbar.c\0"
                            as *const u8 as *const core::ffi::c_char,
                        b"TOOLBAR_EraseBackground\0" as *const u8 as *const core::ffi::c_char,
                        5699 as core::ffi::c_int,
                        b"[%p] response %d not handled to NM_CUSTOMDRAW (CDDS_POSTERASE)\n\0"
                            as *const u8 as *const core::ffi::c_char,
                        (*infoPtr).hwndSelf,
                        ntfret,
                    );
                }
            }
        }
    }
    return ret as LRESULT;
}

unsafe extern "C" fn TOOLBAR_GetFont(infoPtr: *const TOOLBAR_INFO) -> LRESULT {
    return (*infoPtr).hFont as LRESULT;
}

unsafe extern "C" fn TOOLBAR_SetRelativeHotItem(infoPtr: *mut TOOLBAR_INFO, iDirection: INT, dwReason: DWORD) {
    let mut i: INT = 0;
    let mut nNewHotItem: INT = (*infoPtr).nHotItem;

    while i < (*infoPtr).nNumButtons {
        /* did we wrap? */
        if (nNewHotItem + iDirection < 0)
            || (nNewHotItem + iDirection >= (*infoPtr).nNumButtons)
        {
            let mut nmtbwhi: NMTBWRAPHOTITEM = core::mem::zeroed();
            nmtbwhi.idNew = (*(*infoPtr).buttons.offset(nNewHotItem as isize)).idCommand;
            nmtbwhi.iDirection = iDirection;
            nmtbwhi.dwReason = dwReason;

            if TOOLBAR_SendNotify(
                core::ptr::addr_of_mut!(nmtbwhi.hdr),
                infoPtr,
                (TBN_WRAPHOTITEM) as UINT,
            ) != 0
            {
                return;
            }
        }

        nNewHotItem += iDirection;
        nNewHotItem = (nNewHotItem + (*infoPtr).nNumButtons) % (*infoPtr).nNumButtons;

        if (((*(*infoPtr).buttons.offset(nNewHotItem as isize)).fsState as INT)
            & (TBSTATE_ENABLED as INT))
            != 0
            && (((*(*infoPtr).buttons.offset(nNewHotItem as isize)).fsStyle as INT)
                & (BTNS_SEP as INT))
                == 0
        {
            TOOLBAR_SetHotItemEx(infoPtr, nNewHotItem, dwReason);
            break;
        }

        i += 1;
    }
}

unsafe extern "C" fn TOOLBAR_KeyDown(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut nmkey: NMKEY = NMKEY {
        hdr: NMHDR {
            hwndFrom: core::ptr::null_mut(),
            idFrom: 0,
            code: 0,
        },
        nVKey: 0,
        uFlags: 0,
    };
    nmkey.nVKey = wParam as UINT;
    nmkey.uFlags = HIWORD!(lParam as DWORD) as UINT;

    if TOOLBAR_SendNotify(core::ptr::addr_of_mut!(nmkey.hdr), infoPtr, NM_KEYDOWN) != 0 {
        return DefWindowProcW((*infoPtr).hwndSelf, WM_KEYDOWN as UINT, wParam, lParam);
    }

    match wParam as UINT {
        VK_LEFT | VK_UP => {
            TOOLBAR_SetRelativeHotItem(infoPtr, -(1 as INT), HICF_ARROWKEYS as DWORD);
        }
        VK_RIGHT | VK_DOWN => {
            TOOLBAR_SetRelativeHotItem(infoPtr, 1 as INT, HICF_ARROWKEYS as DWORD);
        }
        VK_SPACE | VK_RETURN => {
            if (*infoPtr).nHotItem >= 0 as INT
                && ((*(*infoPtr).buttons.offset((*infoPtr).nHotItem as isize)).fsState as ::core::ffi::c_int
                    & TBSTATE_ENABLED as ::core::ffi::c_int)
                    != 0
            {
                SendMessageW(
                    (*infoPtr).hwndNotify,
                    WM_COMMAND as UINT,
                    MAKEWPARAM!(
                        (*(*infoPtr).buttons.offset((*infoPtr).nHotItem as isize)).idCommand as WORD,
                        BN_CLICKED as WORD,
                    ) as WPARAM,
                    (*infoPtr).hwndSelf as LPARAM,
                );
            }
        }
        _ => {}
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_LButtonDblClk(
    infoPtr: *mut TOOLBAR_INFO,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut button: BOOL = 0;

    pt.x = LOWORD!(lParam) as i16 as LONG;
    pt.y = HIWORD!(lParam) as i16 as LONG;
    TOOLBAR_InternalHitTest(infoPtr, core::ptr::addr_of_mut!(pt), core::ptr::addr_of_mut!(button));

    if button != 0 {
        TOOLBAR_LButtonDown(infoPtr, wParam, lParam);
    } else if ((*infoPtr).dwStyle & CCS_ADJUSTABLE) != 0 {
        TOOLBAR_Customize(infoPtr);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_LButtonDown(
    infoPtr: *mut TOOLBAR_INFO,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut();
    let mut pt: POINT = core::mem::zeroed();
    let mut nHit: INT = 0;
    let mut nmtb: NMTOOLBARA = core::mem::zeroed();
    let mut nmmouse: NMMOUSE = core::mem::zeroed();
    let mut bDragKeyPressed: BOOL = 0;
    let mut button: BOOL = 0;

    if ((*infoPtr).dwStyle & TBSTYLE_ALTDRAG) != 0 {
        bDragKeyPressed = ((GetKeyState(VK_MENU) as INT) < 0) as BOOL;
    } else {
        bDragKeyPressed = (wParam & (MK_SHIFT as WPARAM)) as BOOL;
    }

    if !(*infoPtr).hwndToolTip.is_null() {
        TOOLBAR_RelayEvent(
            (*infoPtr).hwndToolTip,
            (*infoPtr).hwndSelf,
            WM_LBUTTONDOWN as UINT,
            wParam,
            lParam,
        );
    }

    pt.x = (LOWORD!(lParam as DWORD) as i16) as LONG;
    pt.y = (HIWORD!(lParam as DWORD) as i16) as LONG;
    nHit = TOOLBAR_InternalHitTest(infoPtr, core::ptr::addr_of_mut!(pt), core::ptr::addr_of_mut!(button));

    if button != 0 {
        btnPtr = (*infoPtr).buttons.offset(nHit as isize);

        if bDragKeyPressed != 0 && (((*infoPtr).dwStyle & CCS_ADJUSTABLE) != 0) {
            (*infoPtr).nButtonDrag = nHit;
            SetCapture((*infoPtr).hwndSelf);

            /* If drag cursor has not been loaded, load it.
             * Note: it doesn't need to be freed */
            if hCursorDrag.is_null() {
                hCursorDrag = LoadCursorW(
                    COMCTL32_hModule,
                    MAKEINTRESOURCEW!(IDC_MOVEBUTTON),
                );
            }
            SetCursor(hCursorDrag);
        } else {
            let mut arrowRect: RECT = core::mem::zeroed();
            (*infoPtr).nOldHit = nHit;

            arrowRect = (*btnPtr).rect;
            arrowRect.left = max(
                (*btnPtr).rect.left,
                (*btnPtr).rect.right - DDARROW_WIDTH,
            );

            /* for EX_DRAWDDARROWS style,  click must be in the drop-down arrow rect */
            if ((((*btnPtr).fsState as INT) & (TBSTATE_ENABLED as i32)) != 0)
                && ((((*btnPtr).fsStyle as INT) & (BTNS_WHOLEDROPDOWN as i32)) != 0
                    || ((((*btnPtr).fsStyle as INT) & (BTNS_DROPDOWN as i32)) != 0)
                        && ((TOOLBAR_HasDropDownArrows((*infoPtr).dwExStyle) != 0
                            && PtInRect(
                                core::ptr::addr_of_mut!(arrowRect) as LPCRECT,
                                pt,
                            ) != 0)
                            || (TOOLBAR_HasDropDownArrows((*infoPtr).dwExStyle) == 0)))
            {
                let mut res: LRESULT = 0;

                /* draw in pressed state */
                if (((*btnPtr).fsStyle as INT) & (BTNS_WHOLEDROPDOWN as i32)) != 0 {
                    (*btnPtr).fsState =
                        (((*btnPtr).fsState as INT) | (TBSTATE_PRESSED as i32)) as BYTE;
                } else {
                    (*btnPtr).bDropDownPressed = TRUE as BOOL;
                }
                RedrawWindow(
                    (*infoPtr).hwndSelf,
                    core::ptr::addr_of_mut!((*btnPtr).rect) as LPCRECT,
                    core::ptr::null_mut(),
                    (RDW_ERASE | RDW_INVALIDATE | RDW_UPDATENOW) as UINT,
                );

                memset(
                    core::ptr::addr_of_mut!(nmtb) as *mut core::ffi::c_void,
                    0 as i32,
                    core::mem::size_of::<NMTOOLBARA>(),
                );
                nmtb.iItem = (*btnPtr).idCommand as INT;
                nmtb.rcButton = (*btnPtr).rect;
                res = TOOLBAR_SendNotify(
                    core::ptr::addr_of_mut!(nmtb) as *mut NMHDR,
                    infoPtr,
                    TBN_DROPDOWN as UINT,
                ) as LRESULT;

                if res != (TBDDRET_TREATPRESSED as LRESULT) {
                    let mut msg: MSG = core::mem::zeroed();

                    /* redraw button in unpressed state */
                    if (((*btnPtr).fsStyle as INT) & (BTNS_WHOLEDROPDOWN as i32)) != 0 {
                        (*btnPtr).fsState = ((((*btnPtr).fsState as INT)
                            & !(TBSTATE_PRESSED as INT))) as BYTE;
                    } else {
                        (*btnPtr).bDropDownPressed = FALSE as BOOL;
                    }
                    InvalidateRect(
                        (*infoPtr).hwndSelf,
                        core::ptr::addr_of_mut!((*btnPtr).rect) as LPCRECT,
                        TRUE as BOOL,
                    );

                    /* find and set hot item */
                    GetCursorPos(core::ptr::addr_of_mut!(pt));
                    ScreenToClient((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(pt));
                    nHit = TOOLBAR_InternalHitTest(
                        infoPtr,
                        core::ptr::addr_of_mut!(pt),
                        core::ptr::addr_of_mut!(button),
                    );
                    if (*infoPtr).bAnchor == 0 || button != 0 {
                        TOOLBAR_SetHotItemEx(
                            infoPtr,
                            nHit,
                            (HICF_MOUSE | HICF_LMOUSE) as DWORD,
                        );
                    }

                    /* remove any left mouse button down or double-click messages
                     * so that we can get a toggle effect on the button */
                    while PeekMessageW(
                        core::ptr::addr_of_mut!(msg),
                        (*infoPtr).hwndSelf,
                        WM_LBUTTONDOWN as UINT,
                        WM_LBUTTONDOWN as UINT,
                        PM_REMOVE as UINT,
                    ) != 0
                        || PeekMessageW(
                            core::ptr::addr_of_mut!(msg),
                            (*infoPtr).hwndSelf,
                            WM_LBUTTONDBLCLK as UINT,
                            WM_LBUTTONDBLCLK as UINT,
                            PM_REMOVE as UINT,
                        ) != 0
                    {}

                    return 0 as LRESULT;
                }
                /* otherwise drop through and process as pushed */
            }
            (*infoPtr).bCaptured = TRUE as BOOL;
            (*infoPtr).nButtonDown = nHit;
            (*infoPtr).bDragOutSent = FALSE as BOOL;

            (*btnPtr).fsState = (((*btnPtr).fsState as INT) | (TBSTATE_PRESSED as i32)) as BYTE;

            TOOLBAR_SetHotItemEx(
                infoPtr,
                nHit,
                (HICF_MOUSE | HICF_LMOUSE) as DWORD,
            );

            if (((*btnPtr).fsState as INT) & (TBSTATE_ENABLED as i32)) != 0 {
                InvalidateRect(
                    (*infoPtr).hwndSelf,
                    core::ptr::addr_of_mut!((*btnPtr).rect) as LPCRECT,
                    TRUE as BOOL,
                );
            }
            UpdateWindow((*infoPtr).hwndSelf);
            SetCapture((*infoPtr).hwndSelf);
        }

        memset(
            core::ptr::addr_of_mut!(nmtb) as *mut core::ffi::c_void,
            0 as i32,
            core::mem::size_of::<NMTOOLBARA>(),
        );
        nmtb.iItem = (*btnPtr).idCommand as INT;
        TOOLBAR_SendNotify(
            core::ptr::addr_of_mut!(nmtb) as *mut NMHDR,
            infoPtr,
            TBN_BEGINDRAG as UINT,
        );
    }

    nmmouse.dwHitInfo = nHit as LPARAM;

    /* !!! Undocumented - sends NM_LDOWN with the NMMOUSE structure. */
    if button == 0 {
        nmmouse.dwItemSpec = (-(1 as isize)) as DWORD_PTR;
    } else {
        nmmouse.dwItemSpec = (*(*infoPtr).buttons.offset(nmmouse.dwHitInfo as isize)).idCommand
            as DWORD_PTR;
        nmmouse.dwItemData = (*(*infoPtr).buttons.offset(nmmouse.dwHitInfo as isize)).dwData;
    }

    ClientToScreen((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(pt));
    nmmouse.pt = pt;

    if TOOLBAR_SendNotify(
        core::ptr::addr_of_mut!(nmmouse.hdr) as *mut NMHDR,
        infoPtr,
        NM_LDOWN as UINT,
    ) == 0
    {
        return DefWindowProcW(
            (*infoPtr).hwndSelf,
            WM_LBUTTONDOWN as UINT,
            wParam,
            lParam,
        );
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_LButtonUp(
    infoPtr: *mut TOOLBAR_INFO,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut();
    let mut pt: POINT = core::mem::zeroed();
    let mut nHit: INT = 0;
    let mut nOldIndex: INT = -(1 as INT);
    let mut hdr: NMHDR = core::mem::zeroed();
    let mut nmmouse: NMMOUSE = core::mem::zeroed();
    let mut nmtb: NMTOOLBARA = core::mem::zeroed();
    let mut button: BOOL = 0;

    if !(*infoPtr).hwndToolTip.is_null() {
        TOOLBAR_RelayEvent(
            (*infoPtr).hwndToolTip,
            (*infoPtr).hwndSelf,
            WM_LBUTTONUP as UINT,
            wParam,
            lParam,
        );
    }

    pt.x = LOWORD!(lParam) as i16 as LONG;
    pt.y = HIWORD!(lParam) as i16 as LONG;
    nHit = TOOLBAR_InternalHitTest(
        infoPtr,
        core::ptr::addr_of_mut!(pt),
        core::ptr::addr_of_mut!(button),
    );

    if (*infoPtr).bAnchor == 0 || button != 0 {
        TOOLBAR_SetHotItemEx(
            infoPtr,
            if button != 0 { nHit } else { TOOLBAR_NOWHERE as INT },
            (HICF_MOUSE | HICF_LMOUSE) as DWORD,
        );
    }

    if (*infoPtr).nButtonDrag >= (0 as INT) {
        let mut rcClient: RECT = core::mem::zeroed();
        let mut hdr_0: NMHDR = core::mem::zeroed();

        btnPtr = (*infoPtr).buttons.offset((*infoPtr).nButtonDrag as isize) as *mut TBUTTON_INFO;
        ReleaseCapture();
        /* reset cursor */
        SetCursor(LoadCursorW(
            core::ptr::null_mut(),
            IDC_ARROW as LPCWSTR,
        ));

        GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcClient));
        if PtInRect(
            core::ptr::addr_of_mut!(rcClient) as LPCRECT,
            pt,
        ) != 0
        {
            let mut nButton: INT = -(1 as INT);
            if nHit >= (0 as INT) {
                nButton = nHit;
            } else if nHit < -(1 as INT) {
                nButton = -nHit;
            } else if (nHit == -(1 as INT))
                && PtInRect(
                    core::ptr::addr_of_mut!((*(*infoPtr).buttons.offset((-nHit) as isize)).rect)
                        as LPCRECT,
                    pt,
                ) != 0
            {
                nButton = -nHit;
            }

            if nButton == (*infoPtr).nButtonDrag {
                /* if the button is moved slightly left and we have a
                 * separator there then remove it */
                if pt.x
                    < ((*btnPtr).rect.left as INT
                        + (((*btnPtr).rect.right as INT - (*btnPtr).rect.left as INT)
                            / (2 as INT)))
                {
                    if (nButton > (0 as INT))
                        && (((*(*infoPtr).buttons.offset(
                            (nButton - (1 as INT)) as isize,
                        ))
                        .fsStyle as INT)
                            & (BTNS_SEP as INT))
                            != 0
                    {
                        TOOLBAR_DeleteButton(infoPtr, nButton - (1 as INT));
                    }
                } else {
                    /* else insert a separator before the dragged button */
                    let mut tbb: TBBUTTON = core::mem::zeroed();
                    memset(
                        core::ptr::addr_of_mut!(tbb) as *mut core::ffi::c_void,
                        0 as core::ffi::c_int,
                        core::mem::size_of::<TBBUTTON>() as size_t,
                    );
                    tbb.fsStyle = BTNS_SEP as BYTE;
                    tbb.iString = -(1 as INT) as INT_PTR;
                    TOOLBAR_InsertButtonT(
                        infoPtr,
                        nButton,
                        core::ptr::addr_of_mut!(tbb),
                        TRUE as BOOL,
                    );
                }
            } else {
                if nButton == -(1 as INT) {
                    if ((*infoPtr).nNumButtons > (0 as INT))
                        && (pt.x
                            < (*(*infoPtr).buttons.offset(0 as isize))
                                .rect
                                .left as LONG)
                    {
                        TOOLBAR_MoveButton(
                            infoPtr,
                            (*infoPtr).nButtonDrag,
                            0 as INT,
                        );
                    } else {
                        TOOLBAR_MoveButton(infoPtr, (*infoPtr).nButtonDrag, (*infoPtr).nNumButtons);
                    }
                } else {
                    TOOLBAR_MoveButton(infoPtr, (*infoPtr).nButtonDrag, nButton);
                }
            }
        } else {
            if (*__wine_dbch___default).flags as INT
                & ((1 as INT) << (__WINE_DBCL_TRACE as INT))
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/toolbar.c\0"
                        .as_ptr() as *const core::ffi::c_char,
                    b"TOOLBAR_LButtonUp\0".as_ptr() as *const core::ffi::c_char,
                    6020 as core::ffi::c_int,
                    b"button %d dragged out of toolbar\n\0".as_ptr() as *const core::ffi::c_char,
                    (*infoPtr).nButtonDrag,
                );
            }
            TOOLBAR_DeleteButton(infoPtr, (*infoPtr).nButtonDrag);
        }

        /* button under cursor changed so need to re-set hot item */
        TOOLBAR_SetHotItemEx(
            infoPtr,
            if button != 0 { nHit } else { TOOLBAR_NOWHERE as INT },
            (HICF_MOUSE | HICF_LMOUSE) as DWORD,
        );
        (*infoPtr).nButtonDrag = -(1 as INT);

        TOOLBAR_SendNotify(
            core::ptr::addr_of_mut!(hdr_0),
            infoPtr,
            TBN_TOOLBARCHANGE as UINT,
        );
    } else if (*infoPtr).nButtonDown >= (0 as INT) {
        let mut was_clicked: BOOL = (nHit == (*infoPtr).nButtonDown) as INT;

        btnPtr = (*infoPtr).buttons.offset((*infoPtr).nButtonDown as isize) as *mut TBUTTON_INFO;
        (*btnPtr).fsState =
            (((*btnPtr).fsState as INT) & !(TBSTATE_PRESSED as INT)) as BYTE;

        if (((*btnPtr).fsStyle as INT) & (BTNS_CHECK as INT)) != 0 {
            if (((*btnPtr).fsStyle as INT) & (BTNS_GROUP as INT)) != 0 {
                nOldIndex = TOOLBAR_GetCheckedGroupButtonIndex(infoPtr, nHit);
                if (nOldIndex != nHit) && (nOldIndex != -(1 as INT)) {
                    (*(*infoPtr).buttons.offset(nOldIndex as isize)).fsState =
                        (((*(*infoPtr).buttons.offset(nOldIndex as isize)).fsState as INT)
                            & !(TBSTATE_CHECKED as INT)) as BYTE;
                }
                (*btnPtr).fsState =
                    (((*btnPtr).fsState as INT) | (TBSTATE_CHECKED as INT)) as BYTE;
            } else {
                if (((*btnPtr).fsState as INT) & (TBSTATE_CHECKED as INT)) != 0 {
                    (*btnPtr).fsState =
                        (((*btnPtr).fsState as INT) & !(TBSTATE_CHECKED as INT)) as BYTE;
                } else {
                    (*btnPtr).fsState =
                        (((*btnPtr).fsState as INT) | (TBSTATE_CHECKED as INT)) as BYTE;
                }
            }
        }

        if nOldIndex != -(1 as INT) {
            InvalidateRect(
                (*infoPtr).hwndSelf,
                core::ptr::addr_of_mut!((*(*infoPtr).buttons.offset(nOldIndex as isize)).rect)
                    as LPCRECT,
                TRUE as BOOL,
            );
        }

        /*
         * now we can ReleaseCapture, which triggers CAPTURECHANGED msg,
         * that resets bCaptured and btn TBSTATE_PRESSED flags,
         * and obliterates nButtonDown and nOldHit (see TOOLBAR_CaptureChanged)
         */
        if ((*infoPtr).bCaptured != 0) && ((*infoPtr).nButtonDown >= (0 as INT)) {
            ReleaseCapture();
        }
        (*infoPtr).nButtonDown = -(1 as INT);

        /* Issue NM_RELEASEDCAPTURE to parent to let him know it is released */
        TOOLBAR_SendNotify(
            core::ptr::addr_of_mut!(hdr),
            infoPtr,
            NM_RELEASEDCAPTURE as UINT,
        );

        memset(
            core::ptr::addr_of_mut!(nmtb) as *mut core::ffi::c_void,
            0 as core::ffi::c_int,
            core::mem::size_of::<NMTOOLBARA>() as size_t,
        );
        nmtb.iItem = (*btnPtr).idCommand as INT;
        TOOLBAR_SendNotify(
            core::ptr::addr_of_mut!(nmtb) as *mut NMHDR,
            infoPtr,
            TBN_ENDDRAG as UINT,
        );

        if (was_clicked != 0) && (((*btnPtr).fsState as INT) & (TBSTATE_ENABLED as INT)) != 0 {
            SendMessageW(
                (*infoPtr).hwndNotify,
                WM_COMMAND as UINT,
                MAKEWPARAM!(
                    (*(*infoPtr).buttons.offset(nHit as isize)).idCommand,
                    BN_CLICKED as WORD,
                ),
                (*infoPtr).hwndSelf as LPARAM,
            );

            /* In case we have just been destroyed... */
            if IsWindow((*infoPtr).hwndSelf) == 0 {
                return 0 as LRESULT;
            }
        }
    }

    /* !!! Undocumented - toolbar at 4.71 level and above sends
    * NM_CLICK with the NMMOUSE structure. */
    nmmouse.dwHitInfo = nHit as LPARAM;

    if button == 0 {
        nmmouse.dwItemSpec = -(1 as INT) as DWORD_PTR;
    } else {
        nmmouse.dwItemSpec =
            (*(*infoPtr).buttons.offset(nmmouse.dwHitInfo as isize)).idCommand as DWORD_PTR;
        nmmouse.dwItemData = (*(*infoPtr).buttons.offset(nmmouse.dwHitInfo as isize)).dwData;
    }

    ClientToScreen((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(pt));
    nmmouse.pt = pt;

    if TOOLBAR_SendNotify(
        core::ptr::addr_of_mut!(nmmouse) as LPNMHDR,
        infoPtr,
        NM_CLICK as UINT,
    ) == 0
    {
        return DefWindowProcW(
            (*infoPtr).hwndSelf,
            WM_LBUTTONUP as UINT,
            wParam,
            lParam,
        );
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_RButtonUp(
    mut infoPtr: *mut TOOLBAR_INFO,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut nHit: INT = 0;
    let mut nmmouse: NMMOUSE = core::mem::zeroed();
    let mut pt: POINT = tagPOINT { x: 0, y: 0 };
    let mut button: BOOL = 0;

    pt.x = LOWORD!(lParam as DWORD_PTR) as i16 as LONG;
    pt.y = HIWORD!(lParam as DWORD_PTR) as i16 as LONG;

    nHit = TOOLBAR_InternalHitTest(infoPtr, core::ptr::addr_of_mut!(pt), core::ptr::addr_of_mut!(button));
    nmmouse.dwHitInfo = nHit as LPARAM;

    if button == 0 {
        nmmouse.dwItemSpec = -(1 as ::core::ffi::c_int) as DWORD_PTR;
    } else {
        nmmouse.dwItemSpec =
            (*(*infoPtr).buttons.offset(nmmouse.dwHitInfo as isize)).idCommand as DWORD_PTR;
        nmmouse.dwItemData =
            (*(*infoPtr).buttons.offset(nmmouse.dwHitInfo as isize)).dwData;
    }

    ClientToScreen((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(pt));
    nmmouse.pt = pt;

    if TOOLBAR_SendNotify(
        core::ptr::addr_of_mut!(nmmouse) as LPNMHDR,
        infoPtr,
        NM_RCLICK as UINT,
    ) == 0
    {
        return DefWindowProcW(
            (*infoPtr).hwndSelf,
            WM_RBUTTONUP as UINT,
            wParam,
            lParam,
        );
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_RButtonDblClk(
    infoPtr: *mut TOOLBAR_INFO,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut nmhdr: NMHDR = core::mem::zeroed();

    if TOOLBAR_SendNotify(core::ptr::addr_of_mut!(nmhdr), infoPtr, NM_RDBLCLK) == 0 {
        return DefWindowProcW(
            (*infoPtr).hwndSelf,
            (WM_RBUTTONDBLCLK as UINT),
            wParam,
            lParam,
        );
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_CaptureChanged(infoPtr: *mut TOOLBAR_INFO) -> LRESULT {
    let mut btnPtr: *mut TBUTTON_INFO;
    (*infoPtr).bCaptured = FALSE;
    if (*infoPtr).nButtonDown >= 0 {
        btnPtr = (*infoPtr).buttons.offset((*infoPtr).nButtonDown as isize);
        (*btnPtr).fsState = ((*btnPtr).fsState as ::core::ffi::c_int & (!TBSTATE_PRESSED) as i32) as BYTE;
        (*infoPtr).nOldHit = -1;
        if (*btnPtr).fsState as ::core::ffi::c_int & (TBSTATE_ENABLED as i32) != 0 {
            InvalidateRect(
                (*infoPtr).hwndSelf,
                core::ptr::addr_of_mut!((*btnPtr).rect),
                TRUE,
            );
        }
    }
    return 0;
}

unsafe extern "C" fn TOOLBAR_MouseLeave(mut infoPtr: *mut TOOLBAR_INFO) -> LRESULT {
    /* don't remove hot effects when in anchor highlighting mode or when a
     * drop-down button is pressed */
    if (*infoPtr).nHotItem >= 0 && (*infoPtr).bAnchor == 0 {
        let mut hotBtnPtr: *mut TBUTTON_INFO =
            (*infoPtr).buttons.offset((*infoPtr).nHotItem as isize) as *mut TBUTTON_INFO;
        if (*hotBtnPtr).bDropDownPressed == 0 {
            TOOLBAR_SetHotItemEx(infoPtr, TOOLBAR_NOWHERE, HICF_MOUSE);
        }
    }

    if (*infoPtr).nOldHit < 0 {
        return TRUE as LRESULT;
    }

    /* If the last button we were over is depressed then make it not */
    /* depressed and redraw it */
    if (*infoPtr).nOldHit == (*infoPtr).nButtonDown {
        let mut btnPtr: *mut TBUTTON_INFO = ::core::ptr::null_mut::<TBUTTON_INFO>();
        let mut rc1: RECT = core::mem::zeroed();

        btnPtr = (*infoPtr).buttons.offset((*infoPtr).nButtonDown as isize) as *mut TBUTTON_INFO;

        (*btnPtr).fsState = ((*btnPtr).fsState as ::core::ffi::c_int & (!TBSTATE_PRESSED) as i32) as BYTE;

        rc1 = (*btnPtr).rect;
        InflateRect(core::ptr::addr_of_mut!(rc1), 1, 1);
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rc1), TRUE);
    }

    if (*infoPtr).bCaptured != 0 && (*infoPtr).bDragOutSent == 0 {
        let mut nmt: NMTOOLBARW = core::mem::zeroed();
        core::ptr::write_bytes(core::ptr::addr_of_mut!(nmt), 0, 1);
        nmt.iItem = (*(*infoPtr).buttons.offset((*infoPtr).nButtonDown as isize)).idCommand
            as ::core::ffi::c_int;
        TOOLBAR_SendNotify(core::ptr::addr_of_mut!(nmt.hdr), infoPtr, (TBN_DRAGOUT) as UINT);
        (*infoPtr).bDragOutSent = TRUE;
    }

    (*infoPtr).nOldHit = -1; /* reset the old hit index as we've left the toolbar */

    return TRUE as LRESULT;
}

unsafe extern "C" fn TOOLBAR_MouseMove(
    infoPtr: *mut TOOLBAR_INFO,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut pt: POINT = core::mem::zeroed();
    let mut trackinfo: TRACKMOUSEEVENT = core::mem::zeroed();
    let mut nHit: INT = 0;
    let mut btnPtr: *mut TBUTTON_INFO = core::ptr::null_mut();
    let mut button: BOOL = 0;

    if ((*infoPtr).dwStyle & ((TBSTYLE_TOOLTIPS) as DWORD)) != 0 && (*infoPtr).hwndToolTip.is_null() {
        TOOLBAR_TooltipCreateControl(infoPtr);
    }

    if ((*infoPtr).dwStyle & ((TBSTYLE_FLAT) as DWORD)) != 0
        || !GetWindowTheme((*infoPtr).hwndSelf).is_null()
    {
        /* fill in the TRACKMOUSEEVENT struct */
        trackinfo.cbSize = (::core::mem::size_of::<TRACKMOUSEEVENT>()) as DWORD;
        trackinfo.dwFlags = (TME_QUERY) as DWORD;

        /* call _TrackMouseEvent to see if we are currently tracking for this hwnd */
        _TrackMouseEvent(&raw mut trackinfo);

        /* Make sure tracking is enabled so we receive a WM_MOUSELEAVE message */
        if trackinfo.hwndTrack != (*infoPtr).hwndSelf
            || (trackinfo.dwFlags & ((TME_LEAVE) as DWORD)) == 0
        {
            trackinfo.dwFlags = (TME_LEAVE) as DWORD; /* notify upon leaving */
            trackinfo.hwndTrack = (*infoPtr).hwndSelf;

            /* call TRACKMOUSEEVENT so we receive a WM_MOUSELEAVE message */
            /* and can properly deactivate the hot toolbar button */
            _TrackMouseEvent(&raw mut trackinfo);
        }
    }

    if !(*infoPtr).hwndToolTip.is_null() {
        TOOLBAR_RelayEvent(
            (*infoPtr).hwndToolTip,
            (*infoPtr).hwndSelf,
            (WM_MOUSEMOVE) as UINT,
            wParam,
            lParam,
        );
    }

    pt.x = (LOWORD!(lParam as DWORD) as ::core::ffi::c_short) as LONG;
    pt.y = (HIWORD!(lParam as DWORD) as ::core::ffi::c_short) as LONG;

    nHit = TOOLBAR_InternalHitTest(infoPtr, &raw mut pt, &raw mut button);

    if ((*infoPtr).dwStyle & ((TBSTYLE_FLAT) as DWORD)) != 0
        || !GetWindowTheme((*infoPtr).hwndSelf).is_null()
    {
        if (*infoPtr).bAnchor == 0 || button != 0 {
            TOOLBAR_SetHotItemEx(
                infoPtr,
                if button != 0 { nHit } else { (TOOLBAR_NOWHERE) as INT },
                (HICF_MOUSE) as DWORD,
            );
        }
    }

    if (*infoPtr).nOldHit != nHit {
        if (*infoPtr).bCaptured != 0 {
            if (*infoPtr).bDragOutSent == 0 {
                let mut nmt: NMTOOLBARW = core::mem::zeroed();
                ZeroMemory!(
                    &raw mut nmt as *mut core::ffi::c_void,
                    core::mem::size_of::<NMTOOLBARW>(),
                );
                nmt.iItem = (*(*infoPtr).buttons.offset((*infoPtr).nButtonDown as isize)).idCommand
                    as INT;
                TOOLBAR_SendNotify(&raw mut nmt.hdr, infoPtr, (TBN_DRAGOUT) as UINT);
                (*infoPtr).bDragOutSent = (TRUE) as BOOL;
            }

            btnPtr = (*infoPtr).buttons.offset((*infoPtr).nButtonDown as isize) as *mut TBUTTON_INFO;
            if (*infoPtr).nOldHit == (*infoPtr).nButtonDown {
                (*btnPtr).fsState =
                    ((*btnPtr).fsState as ::core::ffi::c_int & !((TBSTATE_PRESSED) as ::core::ffi::c_int)) as BYTE;
                InvalidateRect(
                    (*infoPtr).hwndSelf,
                    &raw mut (*btnPtr).rect as LPCRECT,
                    (TRUE) as BOOL,
                );
            } else if nHit == (*infoPtr).nButtonDown {
                (*btnPtr).fsState =
                    ((*btnPtr).fsState as ::core::ffi::c_int | ((TBSTATE_PRESSED) as ::core::ffi::c_int)) as BYTE;
                InvalidateRect(
                    (*infoPtr).hwndSelf,
                    &raw mut (*btnPtr).rect as LPCRECT,
                    (TRUE) as BOOL,
                );
            }
            (*infoPtr).nOldHit = nHit;
        }
    }

    return (0) as LRESULT;
}

unsafe extern "C" fn TOOLBAR_NCActivate(hwnd: HWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    return DefWindowProcW(hwnd, WM_NCACTIVATE as UINT, wParam, lParam);
}

unsafe extern "C" fn TOOLBAR_NCCalcSize(hwnd: HWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if (GetWindowLongW(hwnd, GWL_STYLE) as i32 & CCS_NODIVIDER as i32) == 0 {
        (*(lParam as LPRECT)).top += GetSystemMetrics((SM_CYEDGE) as i32);
    }

    return DefWindowProcW(hwnd, WM_NCCALCSIZE as UINT, wParam, lParam);
}

unsafe extern "C" fn TOOLBAR_NCCreate(
    hwnd: HWND,
    wParam: WPARAM,
    lpcs: *const CREATESTRUCTW,
    iVersion: ::core::ffi::c_int,
) -> LRESULT {
    let mut infoPtr: *mut TOOLBAR_INFO = core::ptr::null_mut::<TOOLBAR_INFO>();
    let mut styleadd: DWORD = 0 as DWORD;

    /* allocate memory for info structure */
    infoPtr = Alloc(core::mem::size_of::<TOOLBAR_INFO>() as DWORD) as *mut TOOLBAR_INFO;
    SetWindowLongPtrW(hwnd, 0 as ::core::ffi::c_int, infoPtr as LONG_PTR);

    /* paranoid!! */
    (*infoPtr).dwStructSize = core::mem::size_of::<TBBUTTON>() as DWORD;
    (*infoPtr).nRows = 1 as INT;

    /* initialize info structure */
    (*infoPtr).nButtonWidth = 23 as INT;
    (*infoPtr).nButtonHeight = 22 as INT;
    (*infoPtr).nBitmapHeight = 16 as INT;
    (*infoPtr).nBitmapWidth = 16 as INT;

    (*infoPtr).nMaxTextRows = 1 as INT;
    (*infoPtr).cxMin = -(1 as ::core::ffi::c_int) as INT;
    (*infoPtr).cxMax = -(1 as ::core::ffi::c_int) as INT;
    (*infoPtr).nNumBitmaps = 0 as INT;
    (*infoPtr).nNumStrings = 0 as INT;

    (*infoPtr).bCaptured = FALSE as BOOL;
    (*infoPtr).nButtonDown = -(1 as ::core::ffi::c_int) as INT;
    (*infoPtr).nButtonDrag = -(1 as ::core::ffi::c_int) as INT;
    (*infoPtr).nOldHit = -(1 as ::core::ffi::c_int) as INT;
    (*infoPtr).nHotItem = -(1 as ::core::ffi::c_int) as INT;
    (*infoPtr).hwndNotify = (*lpcs).hwndParent;
    (*infoPtr).dwDTFlags = (if ((*lpcs).style as DWORD & (TBSTYLE_LIST as DWORD)) != 0 {
        (DT_LEFT as DWORD) | (DT_VCENTER as DWORD) | (DT_SINGLELINE as DWORD) | (DT_END_ELLIPSIS as DWORD)
    } else {
        (DT_CENTER as DWORD) | (DT_END_ELLIPSIS as DWORD)
    }) as DWORD;
    (*infoPtr).bAnchor = FALSE as BOOL; /* no anchor highlighting */
    (*infoPtr).bDragOutSent = FALSE as BOOL;
    (*infoPtr).iVersion = iVersion as INT;
    (*infoPtr).hwndSelf = hwnd;
    (*infoPtr).bDoRedraw = TRUE as BOOL;
    (*infoPtr).clrBtnHighlight = CLR_DEFAULT as COLORREF;
    (*infoPtr).clrBtnShadow = CLR_DEFAULT as COLORREF;
    (*infoPtr).szPadding.cx = DEFPAD_CX as LONG;
    (*infoPtr).szPadding.cy = DEFPAD_CY as LONG;
    (*infoPtr).szSpacing.cx = 0 as LONG;
    (*infoPtr).szSpacing.cy = 0 as LONG;
    memset(
        core::ptr::addr_of_mut!((*infoPtr).themeMargins) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        core::mem::size_of::<MARGINS>() as size_t,
    );
    (*infoPtr).iListGap = DEFLISTGAP as INT;
    (*infoPtr).iTopMargin = default_top_margin(infoPtr) as INT;
    (*infoPtr).dwStyle = (*lpcs).style as DWORD;
    (*infoPtr).tbim.iButton = -(1 as ::core::ffi::c_int);

    /* fix instance handle, if the toolbar was created by CreateToolbarEx() */
    if GetWindowLongPtrW(hwnd, GWLP_HINSTANCE as ::core::ffi::c_int) == 0 {
        let mut hInst: HINSTANCE = GetWindowLongPtrW(
            GetParent(hwnd),
            GWLP_HINSTANCE as ::core::ffi::c_int,
        ) as HINSTANCE;
        SetWindowLongPtrW(hwnd, GWLP_HINSTANCE as ::core::ffi::c_int, hInst as LONG_PTR);
    }

    /* I think the code below is a bug, but it is the way that the native
     * controls seem to work. The effect is that if the user of TBSTYLE_FLAT
     * forgets to specify TBSTYLE_TRANSPARENT but does specify either
     * CCS_TOP or CCS_BOTTOM (_NOMOVEY and _TOP), then the control
     * does *not* set TBSTYLE_TRANSPARENT even though it should!!!!
     * Somehow, the only cases of this seem to be MFC programs.
     *
     * Note also that the addition of _TRANSPARENT occurs *only* here. It
     * does not occur in the WM_STYLECHANGING routine.
     *    (Guy Albertelli   9/2001)
     *
     */
    if (((*infoPtr).dwStyle & (TBSTYLE_FLAT as DWORD) != 0
        || !GetWindowTheme((*infoPtr).hwndSelf).is_null())
        && ((*lpcs).style as DWORD & (TBSTYLE_TRANSPARENT as DWORD)) == 0)
    {
        styleadd |= TBSTYLE_TRANSPARENT as DWORD;
    }
    if ((*lpcs).style as DWORD & ((CCS_TOP as DWORD) | (CCS_NOMOVEY as DWORD))) == 0 {
        styleadd |= CCS_TOP as DWORD; /* default to top */
        SetWindowLongW(hwnd, GWL_STYLE as ::core::ffi::c_int, ((*lpcs).style as DWORD | styleadd) as LONG);
    }

    return DefWindowProcW(hwnd, WM_NCCREATE as UINT, wParam, lpcs as LPARAM);
}

unsafe extern "C" fn TOOLBAR_NCPaint(hwnd: HWND, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut dwStyle: DWORD = GetWindowLongW(hwnd, GWL_STYLE) as DWORD;
    let mut rcWindow: RECT = core::mem::zeroed();
    let mut hdc: HDC;

    if (dwStyle & WS_MINIMIZE as DWORD) != 0 {
        return 0;
    }

    DefWindowProcW(hwnd, WM_NCPAINT as UINT, wParam, lParam);

    hdc = GetDCEx(
        hwnd,
        core::ptr::null_mut(),
        (DCX_USESTYLE | DCX_WINDOW) as DWORD,
    );
    if hdc.is_null() {
        return 0;
    }

    if (dwStyle & CCS_NODIVIDER as DWORD) == 0 {
        GetWindowRect(hwnd, core::ptr::addr_of_mut!(rcWindow));
        OffsetRect(
            core::ptr::addr_of_mut!(rcWindow),
            -(rcWindow.left),
            -(rcWindow.top),
        );
        if (dwStyle & WS_BORDER as DWORD) != 0 {
            InflateRect(
                core::ptr::addr_of_mut!(rcWindow),
                -(1),
                -(1),
            );
        }
        DrawEdge(
            hdc,
            core::ptr::addr_of_mut!(rcWindow),
            EDGE_ETCHED as UINT,
            BF_TOP as UINT,
        );
    }

    ReleaseDC(hwnd, hdc);

    return 0;
}

unsafe extern "C" fn TOOLBAR_TTGetDispInfo(
    infoPtr: *mut TOOLBAR_INFO,
    lpnmtdi: *mut NMTTDISPINFOW,
) -> LRESULT {
    let mut index: INT = TOOLBAR_GetButtonIndex(infoPtr, (*lpnmtdi).hdr.idFrom as INT, FALSE);
    let mut nmtdi: NMTTDISPINFOA = core::mem::zeroed();
    let mut len: u32 = 0;
    let mut ret: LRESULT = 0;

    Free((*infoPtr).pszTooltipText as LPVOID);
    (*infoPtr).pszTooltipText = core::ptr::null_mut();

    if index < 0 {
        return 0 as LRESULT;
    }

    if (*infoPtr).bUnicode != 0 {
        let mut wszBuffer: [WCHAR; (INFOTIPSIZE + 1) as _] =
            [0; (INFOTIPSIZE + 1) as _];
        let mut tbgit: NMTBGETINFOTIPW = core::mem::zeroed();

        wszBuffer[0 as usize] = 0;
        wszBuffer[INFOTIPSIZE as usize] = 0;

        tbgit.pszText = wszBuffer.as_mut_ptr();
        tbgit.cchTextMax = INFOTIPSIZE as INT;
        tbgit.iItem = (*lpnmtdi).hdr.idFrom as INT;
        tbgit.lParam = (*(*infoPtr).buttons.offset(index as isize)).dwData as LPARAM;

        TOOLBAR_SendNotify(core::ptr::addr_of_mut!(tbgit.hdr), infoPtr, (TBN_GETINFOTIPW) as UINT);

        len = if !tbgit.pszText.is_null() {
            lstrlenW(tbgit.pszText as LPCWSTR) as u32
        } else {
            0 as u32
        };
        if len > ARRAY_SIZE!((*lpnmtdi).szText) as u32 - 1 {
            (*infoPtr).pszTooltipText = Alloc(
                ((len + 1) as usize * core::mem::size_of::<WCHAR>()) as DWORD,
            ) as LPWSTR;
            if !(*infoPtr).pszTooltipText.is_null() {
                memcpy(
                    (*infoPtr).pszTooltipText as *mut core::ffi::c_void,
                    tbgit.pszText as *const core::ffi::c_void,
                    ((len + 1) as usize * core::mem::size_of::<WCHAR>()) as usize,
                );
                (*lpnmtdi).lpszText = (*infoPtr).pszTooltipText;
                return 0 as LRESULT;
            }
        } else if len > 0 as u32 {
            memcpy(
                (*lpnmtdi).lpszText as *mut core::ffi::c_void,
                tbgit.pszText as *const core::ffi::c_void,
                ((len + 1) as usize * core::mem::size_of::<WCHAR>()) as usize,
            );
            return 0 as LRESULT;
        }
    } else {
        let mut szBuffer: [CHAR; (INFOTIPSIZE + 1) as _] =
            [0; (INFOTIPSIZE + 1) as _];
        let mut tbgit_0: NMTBGETINFOTIPA = core::mem::zeroed();

        szBuffer[0 as usize] = 0;
        szBuffer[INFOTIPSIZE as usize] = 0;

        tbgit_0.pszText = szBuffer.as_mut_ptr();
        tbgit_0.cchTextMax = INFOTIPSIZE as INT;
        tbgit_0.iItem = (*lpnmtdi).hdr.idFrom as INT;
        tbgit_0.lParam = (*(*infoPtr).buttons.offset(index as isize)).dwData as LPARAM;

        TOOLBAR_SendNotify(core::ptr::addr_of_mut!(tbgit_0.hdr), infoPtr, (TBN_GETINFOTIPA) as UINT);

        len = MultiByteToWideChar(
            CP_ACP,
            0,
            tbgit_0.pszText as LPCSTR,
            -(1 as INT),
            core::ptr::null_mut(),
            0,
        ) as u32;
        if len > ARRAY_SIZE!((*lpnmtdi).szText) as u32 {
            (*infoPtr).pszTooltipText = Alloc(
                (len as usize * core::mem::size_of::<WCHAR>()) as DWORD,
            ) as LPWSTR;
            if !(*infoPtr).pszTooltipText.is_null() {
                MultiByteToWideChar(
                    CP_ACP,
                    0,
                    tbgit_0.pszText as LPCSTR,
                    -(1 as INT),
                    (*infoPtr).pszTooltipText,
                    len as INT,
                );
                (*lpnmtdi).lpszText = (*infoPtr).pszTooltipText;
                return 0 as LRESULT;
            }
        } else if !tbgit_0.pszText.is_null()
            && *tbgit_0.pszText.offset(0 as isize) != 0
        {
            MultiByteToWideChar(
                CP_ACP,
                0,
                tbgit_0.pszText as LPCSTR,
                -(1 as INT),
                (*lpnmtdi).lpszText,
                ARRAY_SIZE!((*lpnmtdi).szText) as INT,
            );
            return 0 as LRESULT;
        }
    }

    /* if button has text, but it is not shown then automatically
     * use that text as tooltip */
    if (((*infoPtr).dwExStyle & TBSTYLE_EX_MIXEDBUTTONS as DWORD) != 0)
        && ((((*(*infoPtr).buttons.offset(index as isize)).fsStyle) as u32
            & BTNS_SHOWTEXT as u32)
            == 0)
    {
        let pszText: LPWSTR = TOOLBAR_GetText(
            infoPtr,
            (*infoPtr).buttons.offset(index as isize),
        );
        len = if !pszText.is_null() {
            lstrlenW(pszText as LPCWSTR) as u32
        } else {
            0 as u32
        };

        if len > ARRAY_SIZE!((*lpnmtdi).szText) as u32 - 1 {
            (*infoPtr).pszTooltipText = Alloc(
                ((len + 1) as usize * core::mem::size_of::<WCHAR>()) as DWORD,
            ) as LPWSTR;
            if !(*infoPtr).pszTooltipText.is_null() {
                memcpy(
                    (*infoPtr).pszTooltipText as *mut core::ffi::c_void,
                    pszText as *const core::ffi::c_void,
                    ((len + 1) as usize * core::mem::size_of::<WCHAR>()) as usize,
                );
                (*lpnmtdi).lpszText = (*infoPtr).pszTooltipText;
                return 0 as LRESULT;
            }
        } else if len > 0 as u32 {
            memcpy(
                (*lpnmtdi).lpszText as *mut core::ffi::c_void,
                pszText as *const core::ffi::c_void,
                ((len + 1) as usize * core::mem::size_of::<WCHAR>()) as usize,
            );
            return 0 as LRESULT;
        }
    }

    /* Last resort, forward TTN_GETDISPINFO to the app:

       - NFR_UNICODE gets TTN_GETDISPINFOW, and TTN_GETDISPINFOA if -W returned no text;
       - NFR_ANSI gets only TTN_GETDISPINFOA.
    */
    if (*infoPtr).bUnicode != 0 {
        ret = SendMessageW(
            (*infoPtr).hwndNotify,
            WM_NOTIFY as UINT,
            (*lpnmtdi).hdr.idFrom as WPARAM,
            lpnmtdi as LPARAM,
        );

        if IS_INTRESOURCE!((*lpnmtdi).lpszText as LPCWSTR) != false {
            return ret;
        }

        if !(*lpnmtdi).lpszText.is_null() && *(*lpnmtdi).lpszText != 0 {
            return ret;
        }
    }

    nmtdi.hdr.hwndFrom = (*lpnmtdi).hdr.hwndFrom;
    nmtdi.hdr.idFrom = (*lpnmtdi).hdr.idFrom;
    nmtdi.hdr.code = TTN_GETDISPINFOA as UINT;
    nmtdi.lpszText = nmtdi.szText.as_mut_ptr();
    nmtdi.szText[0 as usize] = 0 as CHAR;
    nmtdi.hinst = (*lpnmtdi).hinst;
    nmtdi.uFlags = (*lpnmtdi).uFlags;
    nmtdi.lParam = (*lpnmtdi).lParam;

    ret = SendMessageW(
        (*infoPtr).hwndNotify,
        WM_NOTIFY as UINT,
        nmtdi.hdr.idFrom as WPARAM,
        core::ptr::addr_of_mut!(nmtdi) as LPARAM,
    );

    (*lpnmtdi).hinst = nmtdi.hinst;
    (*lpnmtdi).uFlags = nmtdi.uFlags;
    (*lpnmtdi).lParam = nmtdi.lParam;

    if IS_INTRESOURCE!(nmtdi.lpszText as LPCSTR) != false {
        (*lpnmtdi).lpszText = nmtdi.lpszText as *mut WCHAR as LPWSTR;
        return ret;
    }

    if nmtdi.lpszText.is_null() || *nmtdi.lpszText == 0 {
        return ret;
    }

    len = MultiByteToWideChar(
        CP_ACP,
        0,
        nmtdi.lpszText as LPCSTR,
        -(1 as INT),
        core::ptr::null_mut(),
        0,
    ) as u32;
    if len > ARRAY_SIZE!((*lpnmtdi).szText) as u32 {
        (*infoPtr).pszTooltipText = Alloc(
            (len as usize * core::mem::size_of::<WCHAR>()) as DWORD,
        ) as LPWSTR;
        if !(*infoPtr).pszTooltipText.is_null() {
            MultiByteToWideChar(
                CP_ACP,
                0,
                nmtdi.lpszText as LPCSTR,
                -(1 as INT),
                (*infoPtr).pszTooltipText,
                len as INT,
            );
            (*lpnmtdi).lpszText = (*infoPtr).pszTooltipText;
            return 0 as LRESULT;
        }
    } else {
        MultiByteToWideChar(
            CP_ACP,
            0,
            nmtdi.lpszText as LPCSTR,
            -(1 as INT),
            (*lpnmtdi).lpszText,
            ARRAY_SIZE!(nmtdi.szText) as INT,
        );
        return 0 as LRESULT;
    }

    return ret;
}

unsafe extern "C" fn TOOLBAR_Notify(mut infoPtr: *mut TOOLBAR_INFO, mut lpnmh: LPNMHDR) -> LRESULT {
    match ((*lpnmh).code) as i32 {
        PGN_CALCSIZE => {
            let mut lppgc: LPNMPGCALCSIZE = lpnmh as LPNMPGCALCSIZE;

            if (*lppgc).dwFlag == PGF_CALCWIDTH {
                (*lppgc).iWidth = ((*infoPtr).rcBound.right - (*infoPtr).rcBound.left) as ::core::ffi::c_int;
                if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/comctl32/toolbar.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"TOOLBAR_Notify\0" as *const u8 as *const ::core::ffi::c_char,
                        6627 as ::core::ffi::c_int,
                        b"processed PGN_CALCSIZE, returning horz size = %d\n\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        (*lppgc).iWidth,
                    );
                }
            } else {
                (*lppgc).iHeight = ((*infoPtr).rcBound.bottom - (*infoPtr).rcBound.top) as ::core::ffi::c_int;
                if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                    & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/comctl32/toolbar.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"TOOLBAR_Notify\0" as *const u8 as *const ::core::ffi::c_char,
                        6632 as ::core::ffi::c_int,
                        b"processed PGN_CALCSIZE, returning vert size = %d\n\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        (*lppgc).iHeight,
                    );
                }
            }
            return 0;
        }

        PGN_SCROLL => {
            let mut lppgs: LPNMPGSCROLL = lpnmh as LPNMPGSCROLL;

            (*lppgs).iScroll = (if ((*lppgs).iDir & ((PGF_SCROLLLEFT | PGF_SCROLLRIGHT) as i32)) != 0 {
                (*infoPtr).nButtonWidth
            } else {
                (*infoPtr).nButtonHeight
            }) as ::core::ffi::c_int;

            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/toolbar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TOOLBAR_Notify\0" as *const u8 as *const ::core::ffi::c_char,
                    6644 as ::core::ffi::c_int,
                    b"processed PGN_SCROLL, returning scroll=%d, dir=%d\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    (*lppgs).iScroll,
                    (*lppgs).iDir,
                );
            }

            return 0;
        }

        TTN_GETDISPINFOW => {
            return TOOLBAR_TTGetDispInfo(infoPtr, lpnmh as LPNMTTDISPINFOW);
        }

        TTN_GETDISPINFOA => {
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int))
                != 0
            {
                let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_2: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl_2,
                    __dbch_2,
                    b"dll/win32/comctl32/toolbar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TOOLBAR_Notify\0" as *const u8 as *const ::core::ffi::c_char,
                    6652 as ::core::ffi::c_int,
                    b"TTN_GETDISPINFOA - should not be received; please report\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return 0;
        }

        _ => {
            return 0;
        }
    }
}

unsafe extern "C" fn TOOLBAR_NotifyFormat(
    infoPtr: *const TOOLBAR_INFO,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut format: LRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_NotifyFormat\0" as *const u8 as *const ::core::ffi::c_char,
            6666 as ::core::ffi::c_int,
            b"wParam = 0x%lx, lParam = 0x%08lx\n\0" as *const u8
                as *const ::core::ffi::c_char,
            wParam,
            lParam,
        );
    }
    if lParam == (NF_QUERY as LPARAM) {
        return (NFR_UNICODE as LRESULT);
    }
    if lParam == (NF_REQUERY as LPARAM) {
        format = SendMessageW(
            (*infoPtr).hwndNotify,
            (WM_NOTIFYFORMAT as UINT),
            (*infoPtr).hwndSelf as WPARAM,
            (NF_QUERY as LPARAM),
        );
        if format != (NFR_ANSI as LRESULT) && format != (NFR_UNICODE as LRESULT) {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/toolbar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"TOOLBAR_NotifyFormat\0" as *const u8 as *const ::core::ffi::c_char,
                    6676 as ::core::ffi::c_int,
                    b"wrong response to WM_NOTIFYFORMAT (%ld), assuming ANSI\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    format,
                );
            }
            format = (NFR_ANSI as LRESULT);
        }
        return format;
    }
    return (0 as LRESULT);
}

unsafe extern "C" fn TOOLBAR_Paint(infoPtr: *mut TOOLBAR_INFO, wParam: WPARAM) -> LRESULT {
    let mut hdc: HDC = core::ptr::null_mut::<HDC__>();
    let mut ps: PAINTSTRUCT = core::mem::zeroed::<PAINTSTRUCT>();
    ps.rcPaint = (*infoPtr).rcBound;
    hdc = if wParam == 0 {
        BeginPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps))
    } else {
        wParam as HDC
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
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_Paint\0" as *const u8 as *const ::core::ffi::c_char,
            6696 as ::core::ffi::c_int,
            b"psrect=(%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_rect(core::ptr::addr_of_mut!(ps.rcPaint)),
        );
    }
    TOOLBAR_Refresh(infoPtr, hdc, core::ptr::addr_of_mut!(ps));
    if wParam == 0 {
        EndPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
    }
    return 0;
}

unsafe extern "C" fn TOOLBAR_SetFocus(infoPtr: *mut TOOLBAR_INFO) -> LRESULT {
    if (*infoPtr).nNumButtons > 0 {
        TOOLBAR_SetHotItemEx(infoPtr, 0, HICF_OTHER);
    }

    return 0;
}

unsafe extern "C" fn TOOLBAR_SetFont(infoPtr: *mut TOOLBAR_INFO, hFont: HFONT, Redraw: WORD) -> LRESULT {
    if hFont == core::ptr::null_mut() {
        (*infoPtr).hFont = (*infoPtr).hDefaultFont;
    } else {
        (*infoPtr).hFont = hFont;
    }

    TOOLBAR_CalcToolbar(infoPtr);

    if Redraw != 0 {
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE);
    }
    return 1;
}

unsafe extern "C" fn TOOLBAR_SetRedraw(mut infoPtr: *mut TOOLBAR_INFO, mut wParam: WPARAM) -> LRESULT {
    let mut oldredraw: BOOL = (*infoPtr).bDoRedraw;
    (*infoPtr).bDoRedraw = wParam as BOOL;
    if wParam != 0 {
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null(), TRUE);
    }
    return if oldredraw != 0 {
        1
    } else {
        0
    };
}

unsafe extern "C" fn TOOLBAR_Size(mut infoPtr: *mut TOOLBAR_INFO) -> LRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"TOOLBAR_Size\0" as *const u8 as *const ::core::ffi::c_char,
            6765 as ::core::ffi::c_int,
            b"sizing toolbar\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if ((*infoPtr).dwExStyle & (TBSTYLE_EX_HIDECLIPPEDBUTTONS as DWORD)) != 0 {
        let mut delta_width: RECT = tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let mut delta_height: RECT = tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let mut client: RECT = tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let mut dummy: RECT = tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let mut min_x: DWORD = 0;
        let mut max_x: DWORD = 0;
        let mut min_y: DWORD = 0;
        let mut max_y: DWORD = 0;
        let mut btnPtr: *mut TBUTTON_INFO = ::core::ptr::null_mut::<TBUTTON_INFO>();
        let mut i: INT = 0;
        GetClientRect((*infoPtr).hwndSelf, &raw mut client);
        if client.right > (*infoPtr).client_rect.right {
            min_x = (*infoPtr).client_rect.right as DWORD;
            max_x = client.right as DWORD;
        } else {
            max_x = (*infoPtr).client_rect.right as DWORD;
            min_x = client.right as DWORD;
        }
        if client.bottom > (*infoPtr).client_rect.bottom {
            min_y = (*infoPtr).client_rect.bottom as DWORD;
            max_y = client.bottom as DWORD;
        } else {
            max_y = (*infoPtr).client_rect.bottom as DWORD;
            min_y = client.bottom as DWORD;
        }
        SetRect(
            &raw mut delta_width,
            min_x as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            max_x as ::core::ffi::c_int,
            min_y as ::core::ffi::c_int,
        );
        SetRect(
            &raw mut delta_height,
            0 as ::core::ffi::c_int,
            min_y as ::core::ffi::c_int,
            max_x as ::core::ffi::c_int,
            max_y as ::core::ffi::c_int,
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
                b"dll/win32/comctl32/toolbar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"TOOLBAR_Size\0" as *const u8 as *const ::core::ffi::c_char,
                6799 as ::core::ffi::c_int,
                b"delta_width %s delta_height %s\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                wine_dbgstr_rect(&raw mut delta_width),
                wine_dbgstr_rect(&raw mut delta_height),
            );
        }
        btnPtr = (*infoPtr).buttons;
        i = 0 as INT;
        while i < (*infoPtr).nNumButtons {
            if IntersectRect(
                &raw mut dummy,
                &raw mut delta_width,
                &raw mut (*btnPtr).rect,
            ) != 0
                || IntersectRect(
                    &raw mut dummy,
                    &raw mut delta_height,
                    &raw mut (*btnPtr).rect,
                ) != 0
            {
                InvalidateRect(
                    (*infoPtr).hwndSelf,
                    &raw mut (*btnPtr).rect,
                    TRUE as BOOL,
                );
            }
            i += 1;
            btnPtr = btnPtr.offset(1);
        }
    }
    GetClientRect((*infoPtr).hwndSelf, &raw mut (*infoPtr).client_rect);
    TOOLBAR_AutoSize(infoPtr);
    return 0 as LRESULT;
}

unsafe extern "C" fn TOOLBAR_StyleChanged(infoPtr: *mut TOOLBAR_INFO, nType: INT, lpStyle: *const STYLESTRUCT) -> LRESULT {
    if nType == GWL_STYLE {
        return TOOLBAR_SetStyle(infoPtr, (*lpStyle).styleNew);
    }

    return 0;
}

unsafe extern "C" fn TOOLBAR_SysColorChange() -> LRESULT {
    COMCTL32_RefreshSysColors();

    return 0;
}

unsafe extern "C" fn theme_changed(infoPtr: *mut TOOLBAR_INFO) -> LRESULT {
    let mut theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    CloseThemeData(theme);
    OpenThemeData((*infoPtr).hwndSelf, core::ptr::addr_of!(themeClass) as LPCWSTR);
    theme = GetWindowTheme((*infoPtr).hwndSelf);
    if !theme.is_null() {
        GetThemeMargins(
            theme,
            core::ptr::null_mut(),
            (TP_BUTTON) as i32,
            (TS_NORMAL) as i32,
            (TMT_CONTENTMARGINS as i32),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*infoPtr).themeMargins),
        );
    } else {
        memset(
            core::ptr::addr_of_mut!((*infoPtr).themeMargins) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<MARGINS>(),
        );
    }

    return 0;
}

unsafe extern "system" fn ToolbarWindowProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: *mut TOOLBAR_INFO =
        GetWindowLongPtrW(hwnd, 0) as *mut TOOLBAR_INFO;
    if ((*__wine_dbch___default).flags as INT & ((1 as INT) << (__WINE_DBCL_TRACE as INT))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/toolbar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ToolbarWindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            6861 as INT,
            b"hwnd=%p msg=%x wparam=%lx lparam=%lx\n\0" as *const u8
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
        TB_ADDBITMAP => {
            return TOOLBAR_AddBitmap(infoPtr, wParam as INT, lParam as *mut TBADDBITMAP);
        }
        TB_ADDBUTTONSA | TB_ADDBUTTONSW => {
            return TOOLBAR_AddButtonsT(
                infoPtr,
                wParam as INT,
                lParam as LPTBBUTTON as *const TBBUTTON,
                (uMsg == TB_ADDBUTTONSW) as INT,
            );
        }
        TB_ADDSTRINGA => {
            return TOOLBAR_AddStringA(infoPtr, wParam as HINSTANCE, lParam);
        }
        TB_ADDSTRINGW => {
            return TOOLBAR_AddStringW(infoPtr, wParam as HINSTANCE, lParam);
        }
        TB_AUTOSIZE => {
            return TOOLBAR_AutoSize(infoPtr);
        }
        TB_BUTTONCOUNT => {
            return TOOLBAR_ButtonCount(infoPtr);
        }
        TB_BUTTONSTRUCTSIZE => {
            return TOOLBAR_ButtonStructSize(infoPtr, wParam as DWORD);
        }
        TB_CHANGEBITMAP => {
            return TOOLBAR_ChangeBitmap(infoPtr, wParam as INT, LOWORD!(lParam as DWORD) as INT);
        }
        TB_CHECKBUTTON => {
            return TOOLBAR_CheckButton(infoPtr, wParam as INT, lParam);
        }
        TB_COMMANDTOINDEX => {
            return TOOLBAR_CommandToIndex(infoPtr, wParam as INT);
        }
        TB_CUSTOMIZE => {
            return TOOLBAR_Customize(infoPtr);
        }
        TB_DELETEBUTTON => {
            return TOOLBAR_DeleteButton(infoPtr, wParam as INT);
        }
        TB_ENABLEBUTTON => {
            return TOOLBAR_EnableButton(infoPtr, wParam as INT, lParam);
        }
        TB_GETANCHORHIGHLIGHT => {
            return TOOLBAR_GetAnchorHighlight(infoPtr);
        }
        TB_GETBITMAP => {
            return TOOLBAR_GetBitmap(infoPtr, wParam as INT);
        }
        TB_GETBITMAPFLAGS => {
            return TOOLBAR_GetBitmapFlags();
        }
        TB_GETBUTTON => {
            return TOOLBAR_GetButton(infoPtr, wParam as INT, lParam as *mut TBBUTTON);
        }
        TB_GETBUTTONINFOA | TB_GETBUTTONINFOW => {
            return TOOLBAR_GetButtonInfoT(
                infoPtr,
                wParam as INT,
                lParam as LPTBBUTTONINFOW,
                (uMsg == TB_GETBUTTONINFOW) as INT,
            );
        }
        TB_GETBUTTONSIZE => {
            return TOOLBAR_GetButtonSize(infoPtr);
        }
        TB_GETBUTTONTEXTA | TB_GETBUTTONTEXTW => {
            return TOOLBAR_GetButtonText(
                infoPtr,
                wParam as INT,
                lParam as LPWSTR,
                (uMsg == TB_GETBUTTONTEXTW) as INT,
            );
        }
        TB_GETDISABLEDIMAGELIST => {
            return TOOLBAR_GetDisabledImageList(infoPtr, wParam);
        }
        TB_GETEXTENDEDSTYLE => {
            return TOOLBAR_GetExtendedStyle(infoPtr);
        }
        TB_GETHOTIMAGELIST => {
            return TOOLBAR_GetHotImageList(infoPtr, wParam);
        }
        TB_GETHOTITEM => {
            return TOOLBAR_GetHotItem(infoPtr);
        }
        TB_GETIMAGELIST => {
            return TOOLBAR_GetDefImageList(infoPtr, wParam);
        }
        TB_GETINSERTMARK => {
            return TOOLBAR_GetInsertMark(infoPtr, lParam as *mut TBINSERTMARK);
        }
        TB_GETINSERTMARKCOLOR => {
            return TOOLBAR_GetInsertMarkColor(infoPtr);
        }
        TB_GETITEMRECT => {
            return TOOLBAR_GetItemRect(infoPtr, wParam as INT, lParam as LPRECT);
        }
        TB_GETMAXSIZE => {
            return TOOLBAR_GetMaxSize(infoPtr, lParam as LPSIZE);
        }
        TB_GETMETRICS => {
            return TOOLBAR_GetMetrics(infoPtr, lParam as *mut TBMETRICS);
        }
        TB_GETPADDING => {
            return TOOLBAR_GetPadding(infoPtr);
        }
        TB_GETRECT => {
            return TOOLBAR_GetRect(infoPtr, wParam as INT, lParam as LPRECT);
        }
        TB_GETROWS => {
            return TOOLBAR_GetRows(infoPtr);
        }
        TB_GETSTATE => {
            return TOOLBAR_GetState(infoPtr, wParam as INT);
        }
        TB_GETSTRINGA => {
            return TOOLBAR_GetStringA(infoPtr, wParam, lParam as LPSTR);
        }
        TB_GETSTRINGW => {
            return TOOLBAR_GetStringW(infoPtr, wParam, lParam as LPWSTR);
        }
        TB_GETSTYLE => {
            return TOOLBAR_GetStyle(infoPtr);
        }
        TB_GETTEXTROWS => {
            return TOOLBAR_GetTextRows(infoPtr);
        }
        TB_GETTOOLTIPS => {
            return TOOLBAR_GetToolTips(infoPtr);
        }
        TB_GETUNICODEFORMAT => {
            return TOOLBAR_GetUnicodeFormat(infoPtr);
        }
        TB_HIDEBUTTON => {
            return TOOLBAR_HideButton(infoPtr, wParam as INT, LOWORD!(lParam as DWORD) as BOOL);
        }
        TB_HITTEST => {
            return TOOLBAR_HitTest(infoPtr, lParam as LPPOINT as *const POINT);
        }
        TB_INDETERMINATE => {
            return TOOLBAR_Indeterminate(infoPtr, wParam as INT, LOWORD!(lParam as DWORD) as BOOL);
        }
        TB_INSERTBUTTONA | TB_INSERTBUTTONW => {
            return TOOLBAR_InsertButtonT(
                infoPtr,
                wParam as INT,
                lParam as *mut TBBUTTON,
                (uMsg == TB_INSERTBUTTONW) as INT,
            );
        }
        TB_ISBUTTONCHECKED => {
            return TOOLBAR_IsButtonChecked(infoPtr, wParam as INT);
        }
        TB_ISBUTTONENABLED => {
            return TOOLBAR_IsButtonEnabled(infoPtr, wParam as INT);
        }
        TB_ISBUTTONHIDDEN => {
            return TOOLBAR_IsButtonHidden(infoPtr, wParam as INT);
        }
        TB_ISBUTTONHIGHLIGHTED => {
            return TOOLBAR_IsButtonHighlighted(infoPtr, wParam as INT);
        }
        TB_ISBUTTONINDETERMINATE => {
            return TOOLBAR_IsButtonIndeterminate(infoPtr, wParam as INT);
        }
        TB_ISBUTTONPRESSED => {
            return TOOLBAR_IsButtonPressed(infoPtr, wParam as INT);
        }
        TB_LOADIMAGES => {
            return TOOLBAR_LoadImages(infoPtr, wParam, lParam as HINSTANCE);
        }
        TB_MAPACCELERATORA | TB_MAPACCELERATORW => {
            return TOOLBAR_MapAccelerator(infoPtr, wParam as WCHAR, lParam as *mut UINT);
        }
        TB_MARKBUTTON => {
            return TOOLBAR_MarkButton(infoPtr, wParam as INT, LOWORD!(lParam as DWORD) as BOOL);
        }
        TB_MOVEBUTTON => {
            return TOOLBAR_MoveButton(infoPtr, wParam as INT, lParam as INT);
        }
        TB_PRESSBUTTON => {
            return TOOLBAR_PressButton(infoPtr, wParam as INT, LOWORD!(lParam as DWORD) as BOOL);
        }
        TB_REPLACEBITMAP => {
            return TOOLBAR_ReplaceBitmap(
                infoPtr,
                lParam as LPTBREPLACEBITMAP as *const TBREPLACEBITMAP,
            );
        }
        TB_SAVERESTOREA => {
            return TOOLBAR_SaveRestoreA(
                infoPtr,
                wParam,
                lParam as LPTBSAVEPARAMSA as *const TBSAVEPARAMSA,
            );
        }
        TB_SAVERESTOREW => {
            return TOOLBAR_SaveRestoreW(
                infoPtr,
                wParam,
                lParam as LPTBSAVEPARAMSW as *const TBSAVEPARAMSW,
            );
        }
        TB_SETANCHORHIGHLIGHT => {
            return TOOLBAR_SetAnchorHighlight(infoPtr, wParam as BOOL);
        }
        TB_SETBITMAPSIZE => {
            return TOOLBAR_SetBitmapSize(infoPtr, wParam, lParam);
        }
        TB_SETBUTTONINFOA | TB_SETBUTTONINFOW => {
            return TOOLBAR_SetButtonInfo(
                infoPtr,
                wParam as INT,
                lParam as LPTBBUTTONINFOW as *const TBBUTTONINFOW,
                (uMsg == TB_SETBUTTONINFOW) as INT,
            );
        }
        TB_SETBUTTONSIZE => {
            return TOOLBAR_SetButtonSize(infoPtr, lParam);
        }
        TB_SETBUTTONWIDTH => {
            return TOOLBAR_SetButtonWidth(infoPtr, lParam);
        }
        TB_SETCMDID => {
            return TOOLBAR_SetCmdId(infoPtr, wParam as INT, lParam as INT);
        }
        TB_SETDISABLEDIMAGELIST => {
            return TOOLBAR_SetDisabledImageList(infoPtr, wParam, lParam as HIMAGELIST);
        }
        TB_SETDRAWTEXTFLAGS => {
            return TOOLBAR_SetDrawTextFlags(infoPtr, wParam as DWORD, lParam as DWORD);
        }
        TB_SETEXTENDEDSTYLE => {
            return TOOLBAR_SetExtendedStyle(infoPtr, wParam as DWORD, lParam as DWORD);
        }
        TB_SETHOTIMAGELIST => {
            return TOOLBAR_SetHotImageList(infoPtr, wParam, lParam as HIMAGELIST);
        }
        TB_SETHOTITEM => {
            return TOOLBAR_SetHotItem(infoPtr, wParam as INT);
        }
        TB_SETIMAGELIST => {
            return TOOLBAR_SetImageList(infoPtr, wParam, lParam as HIMAGELIST);
        }
        TB_SETINDENT => {
            return TOOLBAR_SetIndent(infoPtr, wParam as INT);
        }
        TB_SETINSERTMARK => {
            return TOOLBAR_SetInsertMark(infoPtr, lParam as *mut TBINSERTMARK);
        }
        TB_SETINSERTMARKCOLOR => {
            return TOOLBAR_SetInsertMarkColor(infoPtr, lParam as COLORREF);
        }
        TB_SETMAXTEXTROWS => {
            return TOOLBAR_SetMaxTextRows(infoPtr, wParam as INT);
        }
        TB_SETMETRICS => {
            return TOOLBAR_SetMetrics(infoPtr, lParam as *mut TBMETRICS);
        }
        TB_SETPADDING => {
            return TOOLBAR_SetPadding(infoPtr, lParam);
        }
        TB_SETPARENT => {
            return TOOLBAR_SetParent(infoPtr, wParam as HWND);
        }
        TB_SETROWS => {
            return TOOLBAR_SetRows(infoPtr, wParam, lParam as LPRECT);
        }
        TB_SETSTATE => {
            return TOOLBAR_SetState(infoPtr, wParam as INT, lParam);
        }
        TB_SETSTYLE => {
            return TOOLBAR_SetStyle(infoPtr, lParam as DWORD);
        }
        TB_SETTOOLTIPS => {
            return TOOLBAR_SetToolTips(infoPtr, wParam as HWND);
        }
        TB_SETUNICODEFORMAT => {
            return TOOLBAR_SetUnicodeFormat(infoPtr, wParam);
        }
        TB_SETBOUNDINGSIZE => {
            return TOOLBAR_SetBoundingSize(hwnd, wParam, lParam);
        }
        TB_SETHOTITEM2 => {
            return TOOLBAR_SetHotItem2(infoPtr, wParam as INT, lParam);
        }
        TB_SETLISTGAP => {
            return TOOLBAR_SetListGap(infoPtr, wParam as INT);
        }
        TB_GETIMAGELISTCOUNT => {
            return TOOLBAR_GetImageListCount(infoPtr);
        }
        TB_GETIDEALSIZE => {
            return TOOLBAR_GetIdealSize(infoPtr, wParam, lParam);
        }
        TB_UNKWN464 => {
            return TOOLBAR_Unkwn464(hwnd, wParam, lParam);
        }
        CCM_GETCOLORSCHEME => {
            return TOOLBAR_GetColorScheme(infoPtr, lParam as LPCOLORSCHEME);
        }
        CCM_SETCOLORSCHEME => {
            return TOOLBAR_SetColorScheme(
                infoPtr,
                lParam as LPCOLORSCHEME as *const COLORSCHEME,
            );
        }
        CCM_GETVERSION => {
            return TOOLBAR_GetVersion(infoPtr);
        }
        CCM_SETVERSION => {
            return TOOLBAR_SetVersion(infoPtr, wParam as INT);
        }
        WM_CREATE => {
            return TOOLBAR_Create(hwnd, lParam as *mut CREATESTRUCTW);
        }
        WM_DESTROY => {
            return TOOLBAR_Destroy(infoPtr);
        }
        WM_ERASEBKGND => {
            return TOOLBAR_EraseBackground(infoPtr, wParam, lParam);
        }
        WM_GETFONT => {
            return TOOLBAR_GetFont(infoPtr);
        }
        WM_KEYDOWN => {
            return TOOLBAR_KeyDown(infoPtr, wParam, lParam);
        }
        WM_LBUTTONDBLCLK => {
            return TOOLBAR_LButtonDblClk(infoPtr, wParam, lParam);
        }
        WM_LBUTTONDOWN => {
            return TOOLBAR_LButtonDown(infoPtr, wParam, lParam);
        }
        WM_LBUTTONUP => {
            return TOOLBAR_LButtonUp(infoPtr, wParam, lParam);
        }
        WM_RBUTTONUP => {
            return TOOLBAR_RButtonUp(infoPtr, wParam, lParam);
        }
        WM_RBUTTONDBLCLK => {
            return TOOLBAR_RButtonDblClk(infoPtr, wParam, lParam);
        }
        WM_MOUSEMOVE => {
            return TOOLBAR_MouseMove(infoPtr, wParam, lParam);
        }
        WM_MOUSELEAVE => {
            return TOOLBAR_MouseLeave(infoPtr);
        }
        WM_CAPTURECHANGED => {
            if hwnd == (lParam as HWND) {
                return 0;
            }
            return TOOLBAR_CaptureChanged(infoPtr);
        }
        WM_NCACTIVATE => {
            return TOOLBAR_NCActivate(hwnd, wParam, lParam);
        }
        WM_NCCALCSIZE => {
            return TOOLBAR_NCCalcSize(hwnd, wParam, lParam);
        }
        WM_NCCREATE => {
            return TOOLBAR_NCCreate(
                hwnd,
                wParam,
                lParam as *mut CREATESTRUCTW,
                0,
            );
        }
        WM_NCPAINT => {
            return TOOLBAR_NCPaint(hwnd, wParam, lParam);
        }
        WM_NOTIFY => {
            return TOOLBAR_Notify(infoPtr, lParam as LPNMHDR);
        }
        WM_NOTIFYFORMAT => {
            return TOOLBAR_NotifyFormat(infoPtr, wParam, lParam);
        }
        WM_PRINTCLIENT | WM_PAINT => {
            return TOOLBAR_Paint(infoPtr, wParam);
        }
        WM_SETFOCUS => {
            return TOOLBAR_SetFocus(infoPtr);
        }
        WM_SETFONT => {
            return TOOLBAR_SetFont(infoPtr, wParam as HFONT, lParam as WORD);
        }
        WM_SETREDRAW => {
            return TOOLBAR_SetRedraw(infoPtr, wParam);
        }
        WM_SIZE => {
            return TOOLBAR_Size(infoPtr);
        }
        WM_STYLECHANGED => {
            return TOOLBAR_StyleChanged(
                infoPtr,
                wParam as INT,
                lParam as LPSTYLESTRUCT as *const STYLESTRUCT,
            );
        }
        WM_SYSCOLORCHANGE => {
            return TOOLBAR_SysColorChange();
        }
        WM_THEMECHANGED => {
            return theme_changed(infoPtr);
        }
        WM_CHARTOITEM
        | WM_COMMAND
        | WM_DRAWITEM
        | WM_MEASUREITEM
        | WM_VKEYTOITEM => {
            return SendMessageW((*infoPtr).hwndNotify, uMsg, wParam, lParam);
        }
        PGM_FORWARDMOUSE => {
            return DefWindowProcW(hwnd, uMsg, wParam, lParam);
        }
        _ => {
            if (uMsg >= WM_USER)
                && (uMsg < WM_APP)
                && !(COMCTL32_IsReflectedMessage(uMsg) != 0)
            {
                if ((*__wine_dbch___default).flags as INT & ((1 as INT) << (__WINE_DBCL_ERR as INT))) != 0 {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/comctl32/toolbar.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"ToolbarWindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        7274 as INT,
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
pub unsafe extern "C" fn TOOLBAR_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(wndClass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>() as usize,
    );
    wndClass.style = (CS_GLOBALCLASS | CS_DBLCLKS) as UINT;
    wndClass.lpfnWndProc = Some(
        ToolbarWindowProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
    ) as WNDPROC;
    wndClass.cbClsExtra = 0 as ::core::ffi::c_int;
    wndClass.cbWndExtra =
        core::mem::size_of::<*mut TOOLBAR_INFO>() as ::core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as LPWSTR,
    );
    wndClass.hbrBackground =
        (COLOR_BTNFACE + 1) as HBRUSH;
    wndClass.lpszClassName = TOOLBARCLASSNAMEW.as_ptr() as LPCWSTR;

    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn TOOLBAR_Unregister() {
    UnregisterClassW((TOOLBARCLASSNAMEW).as_ptr(), core::ptr::null_mut());
}

unsafe extern "system" fn ToolbarV6WindowProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if uMsg == WM_NCCREATE {
        return TOOLBAR_NCCreate(hwnd, wParam, lParam as *mut CREATESTRUCTW, 6);
    } else {
        return ToolbarWindowProc(hwnd, uMsg, wParam, lParam);
    }
}
#[no_mangle]
pub unsafe extern "C" fn TOOLBARv6_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(wndClass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>() as usize,
    );
    wndClass.style = (CS_GLOBALCLASS | CS_DBLCLKS) as UINT;
    wndClass.lpfnWndProc =
        Some(ToolbarV6WindowProc);
    wndClass.cbClsExtra = 0 as core::ffi::c_int;
    wndClass.cbWndExtra = core::mem::size_of::<*mut TOOLBAR_INFO>() as core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        (IDC_ARROW as LPWSTR) as LPCWSTR,
    );
    wndClass.hbrBackground =
        (COLOR_BTNFACE + 1) as HBRUSH;
    wndClass.lpszClassName = TOOLBARCLASSNAMEW.as_ptr() as LPCWSTR;

    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn TOOLBARv6_Unregister() {
    UnregisterClassW((TOOLBARCLASSNAMEW).as_ptr(), core::ptr::null_mut());
}

unsafe extern "C" fn TOOLBAR_InsertImageList(pies: *mut *mut PIMLENTRY, cies: *mut INT, himl: HIMAGELIST, id: INT) -> HIMAGELIST {
    let mut himlold: HIMAGELIST = core::ptr::null_mut();
    let mut c: PIMLENTRY = core::ptr::null_mut();

    /* Check if the entry already exists */
    c = TOOLBAR_GetImageListEntry(*pies, *cies, id);

    /* Don't add new entry for NULL imagelist */
    if c.is_null() && himl.is_null() {
        return core::ptr::null_mut();
    }

    /* If this is a new entry we must create it and insert into the array */
    if c.is_null() {
        let mut pnies: *mut PIMLENTRY = core::ptr::null_mut();

        c = Alloc(core::mem::size_of::<IMLENTRY>() as DWORD) as PIMLENTRY;
        (*c).id = id;

        pnies = Alloc(
            (((*cies + 1) as usize).wrapping_mul(core::mem::size_of::<PIMLENTRY>() as usize)) as DWORD,
        ) as *mut PIMLENTRY;
        memcpy(
            pnies as *mut core::ffi::c_void,
            *pies as *const core::ffi::c_void,
            (((*cies) as usize).wrapping_mul(core::mem::size_of::<PIMLENTRY>() as usize)),
        );
        *pnies.offset(*cies as isize) = c;
        *cies += 1;

        Free(*pies as LPVOID);
        *pies = pnies;
    }

    himlold = (*c).himl;
    (*c).himl = himl;

    return himlold;
}

unsafe extern "C" fn TOOLBAR_DeleteImageList(pies: *mut *mut PIMLENTRY, cies: *mut INT) {
    let mut i: ::core::ffi::c_int;
    i = 0;
    while i < *cies {
        Free(*(*pies).offset(i as isize) as LPVOID);
        i += 1;
    }
    Free(*pies as LPVOID);

    *cies = 0;
    *pies = core::ptr::null_mut::<PIMLENTRY>();
}

unsafe extern "C" fn TOOLBAR_GetImageListEntry(pies: *const PIMLENTRY, cies: INT, id: INT) -> PIMLENTRY {
    let mut c: PIMLENTRY = core::ptr::null_mut();

    if !pies.is_null() {
        let mut i: INT = 0;
        while i < cies {
            if (**pies.offset(i as isize)).id == id {
                c = *pies.offset(i as isize);
                break;
            }
            i += 1;
        }
    }

    return c;
}

unsafe extern "C" fn TOOLBAR_GetImageList(pies: *const PIMLENTRY, cies: INT, id: INT) -> HIMAGELIST {
    let mut himlDef: HIMAGELIST = core::ptr::null_mut();
    let mut pie: PIMLENTRY = TOOLBAR_GetImageListEntry(pies, cies, id);

    if !pie.is_null() {
        himlDef = (*pie).himl;
    }

    return himlDef;
}

unsafe extern "C" fn TOOLBAR_GetButtonInfo(infoPtr: *const TOOLBAR_INFO, nmtb: *mut NMTOOLBARW) -> BOOL {
    if (*infoPtr).bUnicode != 0 {
        return TOOLBAR_SendNotify(core::ptr::addr_of_mut!((*nmtb).hdr), infoPtr, (TBN_GETBUTTONINFOW) as UINT) as BOOL;
    } else {
        let mut Buffer: [CHAR; 256] = [0; 256];
        let mut nmtba: NMTOOLBARA = core::mem::zeroed();
        let mut bRet: BOOL = FALSE;

        nmtba.iItem = (*nmtb).iItem;
        nmtba.pszText = core::ptr::addr_of_mut!(Buffer) as *mut CHAR as LPSTR;
        nmtba.cchText = 256 as ::core::ffi::c_int;
        ZeroMemory!(nmtba.pszText as *mut ::core::ffi::c_void, nmtba.cchText as usize);

        if TOOLBAR_SendNotify(core::ptr::addr_of_mut!(nmtba.hdr), infoPtr, (TBN_GETBUTTONINFOA) as UINT) != 0 {
            let mut ccht: ::core::ffi::c_int =
                strlen(nmtba.pszText as *const ::core::ffi::c_char) as ::core::ffi::c_int;
            if ccht != 0 {
                MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    nmtba.pszText as LPCSTR,
                    -(1 as ::core::ffi::c_int),
                    (*nmtb).pszText,
                    (*nmtb).cchText,
                );
            }

            (*nmtb).tbButton = nmtba.tbButton;
            bRet = TRUE as BOOL;
        }

        return bRet;
    }
}

unsafe extern "C" fn TOOLBAR_IsButtonRemovable(infoPtr: *const TOOLBAR_INFO, iItem: i32, btnInfo: *const CUSTOMBUTTON) -> BOOL {
    let mut nmtb: NMTOOLBARW = core::mem::zeroed();

    /* MSDN states that iItem is the index of the button, rather than the
     * command ID as used by every other NMTOOLBAR notification */
    nmtb.iItem = iItem;
    memcpy(
        core::ptr::addr_of_mut!(nmtb.tbButton) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*btnInfo).btn) as *const core::ffi::c_void,
        core::mem::size_of::<TBBUTTON>() as usize,
    );

    return TOOLBAR_SendNotify(
        core::ptr::addr_of_mut!(nmtb.hdr),
        infoPtr,
        (TBN_QUERYDELETE) as UINT,
    ) as BOOL;
}
