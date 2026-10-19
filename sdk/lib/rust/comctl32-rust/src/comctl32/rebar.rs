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
use crate::comctl32::comctl32undoc::Str_GetPtrWtoA;
use crate::comctl32::comctl32undoc::Str_SetPtrAtoW;
use crate::comctl32::commctrl::_TrackMouseEvent;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_rebar);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const REBARCLASSNAMEW: [::core::ffi::c_ushort; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [::core::ffi::c_ushort; 14]>(
        *b"R\0e\0B\0a\0r\0W\0i\0n\0d\0o\0w\x003\x002\0\0\0",
    )
};

pub const WC_COMBOBOXEXW: [::core::ffi::c_ushort; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_ushort; 13]>(
        *b"C\0o\0m\0b\0o\0B\0o\0x\0E\0x\x003\x002\0\0\0",
    )
};

static mut mindragx: UINT = 0 as UINT;

static mut mindragy: UINT = 0 as UINT;

static mut band_stylename: [*const ::core::ffi::c_char; 13] = [
    b"RBBS_BREAK\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBS_FIXEDSIZE\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBS_CHILDEDGE\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBS_HIDDEN\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBS_NOVERT\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBS_FIXEDBMP\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBS_VARIABLEHEIGHT\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBS_GRIPPERALWAYS\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBS_NOGRIPPER\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBS_USECHEVRON\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBS_HIDETITLE\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBS_TOPALIGN\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];

static mut band_maskname: [*const ::core::ffi::c_char; 15] = [
    b"RBBIM_STYLE\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBIM_COLORS\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBIM_TEXT\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBIM_IMAGE\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBIM_CHILD\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBIM_CHILDSIZE\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBIM_SIZE\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBIM_BACKGROUND\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBIM_ID\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBIM_IDEALSIZE\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBIM_LPARAM\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBIM_HEADERSIZE\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBIM_CHEVRONLOCATION\0" as *const u8 as *const ::core::ffi::c_char,
    b"RBBIM_CHEVRONSTATE\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<::core::ffi::c_char>(),
];

static mut themeClass: [WCHAR; 6] = [
    'R' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'b' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

#[allow(non_upper_case_globals)]
pub const GLATESTING: i32 = 0;

#[allow(non_camel_case_types, non_snake_case)]
#[repr(C)]
pub struct REBAR_BAND {
    pub fStyle: u32,
    pub fMask: u32,
    pub clrFore: COLORREF,
    pub clrBack: COLORREF,
    pub iImage: i32,
    pub hwndChild: HWND,
    pub cxMinChild: u32,
    pub cyMinChild: u32,
    pub cx: u32,
    pub hbmBack: HBITMAP,
    pub wID: u32,
    pub cyChild: u32,
    pub cyMaxChild: u32,
    pub cyIntegral: u32,
    pub cxIdeal: u32,
    pub lParam: LPARAM,
    pub cxHeader: u32,
    pub cxEffective: i32,
    pub cyHeader: u32,
    pub cxMinBand: u32,
    pub cyMinBand: u32,
    pub cyRowSoFar: u32,
    pub iRow: i32,
    pub fStatus: u32,
    pub fDraw: u32,
    pub uCDret: u32,
    pub rcBand: RECT,
    pub rcGripper: RECT,
    pub rcCapImage: RECT,
    pub rcCapText: RECT,
    pub rcChild: RECT,
    pub rcChevron: RECT,
    pub lpText: LPWSTR,
    pub hwndPrevParent: HWND,
}

#[allow(non_upper_case_globals)]
pub const CCS_LAYOUT_MASK: u32 = 0x3;

#[allow(non_upper_case_globals)]
pub const HAS_GRIPPER: u32 = 0x00000001;

#[allow(non_upper_case_globals)]
pub const HAS_IMAGE: u32 = 0x00000002;

#[allow(non_upper_case_globals)]
pub const HAS_TEXT: u32 = 0x00000004;

#[allow(non_upper_case_globals)]
pub const DRAW_GRIPPER: u32 = 0x00000001;

#[allow(non_upper_case_globals)]
pub const DRAW_IMAGE: u32 = 0x00000002;

#[allow(non_upper_case_globals)]
pub const DRAW_TEXT: u32 = 0x00000004;

#[allow(non_upper_case_globals)]
pub const DRAW_CHEVRONHOT: u32 = 0x00000040;

#[allow(non_upper_case_globals)]
pub const DRAW_CHEVRONPUSHED: u32 = 0x00000080;

#[allow(non_upper_case_globals)]
pub const NTF_INVALIDATE: u32 = 0x01000000;

#[allow(non_camel_case_types, non_snake_case)]
#[repr(C)]
pub struct REBAR_INFO {
    pub clrBk: COLORREF,
    pub clrText: COLORREF,
    pub clrBtnText: COLORREF,
    pub clrBtnFace: COLORREF,
    pub himl: HIMAGELIST,
    pub uNumBands: u32,
    pub uNumRows: u32,
    pub hwndSelf: HWND,
    pub hwndToolTip: HWND,
    pub hwndNotify: HWND,
    pub hDefaultFont: HFONT,
    pub hFont: HFONT,
    pub imageSize: SIZE,
    pub dwStyle: DWORD,
    pub orgStyle: DWORD,
    pub calcSize: SIZE,
    pub bUnicode: BOOL,
    pub DoRedraw: BOOL,
    pub fStatus: u32,
    pub hcurArrow: HCURSOR,
    pub hcurHorz: HCURSOR,
    pub hcurVert: HCURSOR,
    pub hcurDrag: HCURSOR,
    pub iVersion: i32,
    pub dragStart: POINT,
    pub dragNow: POINT,
    pub iOldBand: i32,
    pub ihitoffset: i32,
    pub ichevronhotBand: i32,
    pub iGrabbedBand: i32,
    pub bands: HDPA,
}

#[allow(non_upper_case_globals)]
pub const BEGIN_DRAG_ISSUED: u32 = 0x00000001;

#[allow(non_upper_case_globals)]
pub const SELF_RESIZE: u32 = 0x00000002;

#[allow(non_upper_case_globals)]
pub const BAND_NEEDS_REDRAW: u32 = 0x00000020;

#[allow(non_upper_case_globals)]
pub const RBBS_UNDOC_FIXEDHEADER: u32 = 0x40000000;

#[allow(non_upper_case_globals)]
pub const SEP_WIDTH_SIZE: u32 = 2;

#[allow(non_upper_case_globals)]
pub const REBAR_ALWAYS_SPACE: u32 = 4;

#[allow(non_upper_case_globals)]
pub const REBAR_POST_IMAGE: u32 = 2;

#[allow(non_upper_case_globals)]
pub const REBAR_POST_TEXT: u32 = 4;

#[allow(non_upper_case_globals)]
pub const GRIPPER_HEIGHT: u32 = 16;

#[allow(non_upper_case_globals)]
pub const REBAR_PRE_GRIPPER: u32 = 2;

#[allow(non_upper_case_globals)]
pub const GRIPPER_WIDTH: u32 = 3;

#[allow(non_upper_case_globals)]
pub const CHEVRON_WIDTH: u32 = 10;

#[allow(non_upper_case_globals)]
pub const REBAR_POST_CHILD: u32 = 4;

#[allow(non_upper_case_globals)]
pub const REBAR_DIVIDER: u32 = 2;

#[allow(non_upper_case_globals)]
pub const REBAR_NO_CHILD_HEIGHT: u32 = 4;

#[allow(non_upper_case_globals)]
pub const REBAR_MINSIZE: u32 = 23;

#[allow(non_upper_case_globals)]
pub const RB_GETBANDINFO_OLD: u32 = 0x0400 + 5; // (WM_USER+5)


unsafe extern "C" fn REBAR_GetBand(infoPtr: *const REBAR_INFO, i: INT) -> *mut REBAR_BAND {
    assert!((i >= 0 && (i as UINT) < (*infoPtr).uNumBands));
    return DPA_GetPtr((*infoPtr).bands, i) as *mut REBAR_BAND;
}

unsafe extern "C" fn REBAR_FmtStyle(mut buffer: *mut ::core::ffi::c_char, mut style: UINT) -> *mut CHAR {
    let mut i: INT = 0;

    *buffer = 0;
    while !band_stylename[i as usize].is_null() {
        if style & ((1 as UINT) << i) != 0 {
            if *buffer != 0 {
                strcat(buffer, b" | \0" as *const u8 as *const ::core::ffi::c_char);
            }
            strcat(buffer, band_stylename[i as usize]);
        }
        i += 1;
    }
    return buffer as *mut CHAR;
}

unsafe extern "C" fn REBAR_FmtMask(buffer: *mut ::core::ffi::c_char, mask: UINT) -> *mut CHAR {
    let mut i: INT = 0 as INT;

    *buffer = 0 as ::core::ffi::c_char;
    while !band_maskname[i as usize].is_null() {
        if mask & (((1 as UINT)) << i) != 0 {
            if *buffer != 0 {
                strcat(buffer, b" | \0".as_ptr() as *const ::core::ffi::c_char);
            }
            strcat(buffer, band_maskname[i as usize]);
        }
        i += 1;
    }
    return buffer as *mut CHAR;
}

unsafe extern "C" fn REBAR_DumpBandInfo(pB: *const REBARBANDINFOW) {
    let mut buff: [i8; 300] = [0; 300];

    if !TRACE_ON!(rebar) {
        return;
    }

    if (*pB).fMask as UINT & RBBIM_ID as UINT != 0 {
        TRACE!(b"ID=%u, \0".as_ptr() as *const i8, (*pB).wID);
    }

    if (*pB).fMask as UINT & RBBIM_COLORS as UINT != 0 {
        TRACE!(
            b", clrF=0x%06x, clrB=0x%06x\0".as_ptr() as *const i8,
            (*pB).clrFore,
            (*pB).clrBack,
        );
    }

    if (*pB).fMask as UINT & RBBIM_STYLE as UINT != 0 {
        TRACE!(
            b"band info: style=0x%08x (%s)\n\0".as_ptr() as *const i8,
            (*pB).fStyle,
            REBAR_FmtStyle(buff.as_mut_ptr(), (*pB).fStyle),
        );
    }
    if (*pB).fMask as UINT
        & (RBBIM_SIZE as UINT
            | RBBIM_IDEALSIZE as UINT
            | RBBIM_HEADERSIZE as UINT
            | RBBIM_LPARAM as UINT)
        != 0
    {
        if (*pB).fMask as UINT & RBBIM_SIZE as UINT != 0 {
            TRACE!(b" cx=%u\0".as_ptr() as *const i8, (*pB).cx);
        }
        if (*pB).fMask as UINT & RBBIM_IDEALSIZE as UINT != 0 {
            TRACE!(b" xIdeal=%u\0".as_ptr() as *const i8, (*pB).cxIdeal);
        }
        if (*pB).fMask as UINT & RBBIM_HEADERSIZE as UINT != 0 {
            TRACE!(b" xHeader=%u\0".as_ptr() as *const i8, (*pB).cxHeader);
        }
        if (*pB).fMask as UINT & RBBIM_LPARAM as UINT != 0 {
            TRACE!(b" lParam=0x%08lx\0".as_ptr() as *const i8, (*pB).lParam);
        }
    }
    if (*pB).fMask as UINT & RBBIM_CHILDSIZE as UINT != 0 {
        TRACE!(
            b"band info: xMin=%u, yMin=%u, yChild=%u, yMax=%u, yIntgl=%u\n\0".as_ptr() as *const i8,
            (*pB).cxMinChild,
            (*pB).cyMinChild,
            (*pB).cyChild,
            (*pB).cyMaxChild,
            (*pB).cyIntegral,
        );
    }
}

unsafe extern "C" fn REBAR_DumpBand(iP: *const REBAR_INFO) {
    let mut buff: [::core::ffi::c_char; 300] = [0; 300];
    let mut pB: *mut REBAR_BAND = ::core::ptr::null_mut::<REBAR_BAND>();
    let mut i: UINT = 0;

    if !TRACE_ON!(rebar) {
        return;
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
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
            387 as ::core::ffi::c_int,
            b"hwnd=%p: color=%08x/%08x, bands=%u, rows=%u, cSize=%d,%d\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*iP).hwndSelf,
            (*iP).clrText,
            (*iP).clrBk,
            (*iP).uNumBands,
            (*iP).uNumRows,
            (*iP).calcSize.cx,
            (*iP).calcSize.cy,
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
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
            391 as ::core::ffi::c_int,
            b"hwnd=%p: flags=%08x, dragStart=%d,%d, dragNow=%d,%d, iGrabbedBand=%d\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*iP).hwndSelf,
            (*iP).fStatus,
            (*iP).dragStart.x,
            (*iP).dragStart.y,
            (*iP).dragNow.x,
            (*iP).dragNow.y,
            (*iP).iGrabbedBand,
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
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
            394 as ::core::ffi::c_int,
            b"hwnd=%p: style=%08x, notify in Unicode=%s, redraw=%s\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*iP).hwndSelf,
            (*iP).dwStyle,
            if (*iP).bUnicode != 0 {
                b"TRUE\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"FALSE\0" as *const u8 as *const ::core::ffi::c_char
            },
            if (*iP).DoRedraw != 0 {
                b"TRUE\0" as *const u8 as *const ::core::ffi::c_char
            } else {
                b"FALSE\0" as *const u8 as *const ::core::ffi::c_char
            },
        );
    }

    i = 0 as UINT;
    while i < (*iP).uNumBands {
        pB = REBAR_GetBand(iP, i as INT);
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                397 as ::core::ffi::c_int,
                b"band # %u:\0" as *const u8 as *const ::core::ffi::c_char,
                i,
            );
        }
        if (*pB).fMask & RBBIM_ID as UINT != 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_3,
                    __dbch_3,
                    b"dll/win32/comctl32/rebar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                    399 as ::core::ffi::c_int,
                    b" ID=%u\0" as *const u8 as *const ::core::ffi::c_char,
                    (*pB).wID,
                );
            }
        }
        if (*pB).fMask & RBBIM_CHILD as UINT != 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_4: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_4,
                    __dbch_4,
                    b"dll/win32/comctl32/rebar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                    401 as ::core::ffi::c_int,
                    b" child=%p\0" as *const u8 as *const ::core::ffi::c_char,
                    (*pB).hwndChild,
                );
            }
        }
        if (*pB).fMask & RBBIM_COLORS as UINT != 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_5: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_5: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_5,
                    __dbch_5,
                    b"dll/win32/comctl32/rebar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                    403 as ::core::ffi::c_int,
                    b" clrF=0x%06x clrB=0x%06x\0" as *const u8
                        as *const ::core::ffi::c_char,
                    (*pB).clrFore,
                    (*pB).clrBack,
                );
            }
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_6: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_6: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_6,
                __dbch_6,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                404 as ::core::ffi::c_int,
                b"\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if (*pB).fMask & RBBIM_STYLE as UINT != 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_7: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_7: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_7,
                    __dbch_7,
                    b"dll/win32/comctl32/rebar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                    405 as ::core::ffi::c_int,
                    b"band # %u: mask=0x%08x (%s)\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    i,
                    (*pB).fMask,
                    REBAR_FmtMask(&raw mut buff as *mut ::core::ffi::c_char, (*pB).fMask),
                );
            }
        }
        if (*pB).fMask & RBBIM_STYLE as UINT != 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_8: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_8: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_8,
                    __dbch_8,
                    b"dll/win32/comctl32/rebar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                    407 as ::core::ffi::c_int,
                    b"band # %u: style=0x%08x (%s)\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    i,
                    (*pB).fStyle,
                    REBAR_FmtStyle(
                        &raw mut buff as *mut ::core::ffi::c_char,
                        (*pB).fStyle,
                    ),
                );
            }
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_9: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_9: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_9,
                __dbch_9,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                409 as ::core::ffi::c_int,
                b"band # %u: xHeader=%u\0" as *const u8 as *const ::core::ffi::c_char,
                i,
                (*pB).cxHeader,
            );
        }
        if (*pB).fMask & (RBBIM_SIZE | RBBIM_IDEALSIZE | RBBIM_LPARAM) as UINT != 0 {
            if (*pB).fMask & RBBIM_SIZE as UINT != 0 {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_10: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_10: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_10,
                        __dbch_10,
                        b"dll/win32/comctl32/rebar.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                        412 as ::core::ffi::c_int,
                        b" cx=%u\0" as *const u8 as *const ::core::ffi::c_char,
                        (*pB).cx,
                    );
                }
            }
            if (*pB).fMask & RBBIM_IDEALSIZE as UINT != 0 {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_11: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_11: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_11,
                        __dbch_11,
                        b"dll/win32/comctl32/rebar.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                        414 as ::core::ffi::c_int,
                        b" xIdeal=%u\0" as *const u8 as *const ::core::ffi::c_char,
                        (*pB).cxIdeal,
                    );
                }
            }
            if (*pB).fMask & RBBIM_LPARAM as UINT != 0 {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_12: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_12: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_12,
                        __dbch_12,
                        b"dll/win32/comctl32/rebar.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                        416 as ::core::ffi::c_int,
                        b" lParam=0x%08lx\0" as *const u8 as *const ::core::ffi::c_char,
                        (*pB).lParam,
                    );
                }
            }
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_13: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_13: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_13,
                __dbch_13,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                418 as ::core::ffi::c_int,
                b"\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        // RBBIM_CHILDSIZE (0x20) is a nonzero constant, so this test is always true
        if 0x20 as UINT != 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_14: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_14: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_14,
                    __dbch_14,
                    b"dll/win32/comctl32/rebar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                    421 as ::core::ffi::c_int,
                    b"band # %u: xMin=%u, yMin=%u, yChild=%u, yMax=%u, yIntgl=%u\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    i,
                    (*pB).cxMinChild,
                    (*pB).cyMinChild,
                    (*pB).cyChild,
                    (*pB).cyMaxChild,
                    (*pB).cyIntegral,
                );
            }
        }
        if (*pB).fMask & RBBIM_TEXT as UINT != 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_15: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_15: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_15,
                    __dbch_15,
                    b"dll/win32/comctl32/rebar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                    424 as ::core::ffi::c_int,
                    b"band # %u: text=%s\n\0" as *const u8 as *const ::core::ffi::c_char,
                    i,
                    if !(*pB).lpText.is_null() {
                        debugstr_w((*pB).lpText as *const WCHAR)
                    } else {
                        b"(null)\0" as *const u8 as *const ::core::ffi::c_char
                    },
                );
            }
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_16: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_16: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_16,
                __dbch_16,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                426 as ::core::ffi::c_int,
                b"band # %u: cxMinBand=%u, cxEffective=%u, cyMinBand=%u\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                i,
                (*pB).cxMinBand,
                (*pB).cxEffective,
                (*pB).cyMinBand,
            );
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_17: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_17: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_17,
                __dbch_17,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                429 as ::core::ffi::c_int,
                b"band # %u: fStatus=%08x, fDraw=%08x, Band=(%s), Grip=(%s)\n\0"
                    as *const u8 as *const ::core::ffi::c_char,
                i,
                (*pB).fStatus,
                (*pB).fDraw,
                wine_dbgstr_rect(&raw mut (*pB).rcBand),
                wine_dbgstr_rect(&raw mut (*pB).rcGripper),
            );
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_18: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_18: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_18,
                __dbch_18,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_DumpBand\0" as *const u8 as *const ::core::ffi::c_char,
                432 as ::core::ffi::c_int,
                b"band # %u: Img=(%s), Txt=(%s), Child=(%s)\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                i,
                wine_dbgstr_rect(&raw mut (*pB).rcCapImage),
                wine_dbgstr_rect(&raw mut (*pB).rcCapText),
                wine_dbgstr_rect(&raw mut (*pB).rcChild),
            );
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn translate_rect(infoPtr: *const REBAR_INFO, dest: *mut RECT, src: *const RECT) {
    if (*infoPtr).dwStyle & CCS_VERT != 0 {
        let mut tmp: i32;
        tmp = (*src).left;
        (*dest).left = (*src).top;
        (*dest).top = tmp;

        tmp = (*src).right;
        (*dest).right = (*src).bottom;
        (*dest).bottom = tmp;
    } else {
        *dest = *src;
    }
}

unsafe extern "C" fn get_rect_cx(infoPtr: *const REBAR_INFO, lpRect: *const RECT) -> i32 {
    if ((*infoPtr).dwStyle & CCS_VERT) != 0 {
        return (*lpRect).bottom - (*lpRect).top;
    }
    return (*lpRect).right - (*lpRect).left;
}

unsafe extern "C" fn get_rect_cy(infoPtr: *const REBAR_INFO, lpRect: *const RECT) -> i32 {
    if ((*infoPtr).dwStyle & (CCS_VERT as u32)) != 0 {
        return (*lpRect).right - (*lpRect).left;
    }
    return (*lpRect).bottom - (*lpRect).top;
}

unsafe extern "C" fn round_child_height(lpBand: *const REBAR_BAND, mut cyHeight: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut cy: ::core::ffi::c_int = 0;
    if (*lpBand).cyIntegral == 0 {
        cyHeight = max(cyHeight, (*lpBand).cyMinChild as ::core::ffi::c_int);
        cyHeight = min(cyHeight, (*lpBand).cyMaxChild as ::core::ffi::c_int);
        return cyHeight;
    }
    cy = max(cyHeight - (*lpBand).cyMinChild as ::core::ffi::c_int, 0 as ::core::ffi::c_int);
    cy = (*lpBand).cyMinChild.wrapping_add(
        (cy as UINT)
            .wrapping_div((*lpBand).cyIntegral)
            .wrapping_mul((*lpBand).cyIntegral),
    ) as ::core::ffi::c_int;
    // The C's `min(cy, lpBand->cyMaxChild)` mixes int and UINT, so it compares UNSIGNED: a
    // cyMaxChild of (UINT)-1, "no maximum", never wins. Casting it to int made it -1 and
    // every such band 0 px tall, explorer's task band among them (an 8 px taskbar).
    // The cyIntegral == 0 arm above casts to int in the C itself, so that one stays signed.
    if (cy as UINT) > (*lpBand).cyMaxChild {
        cy = (*lpBand).cyMaxChild as ::core::ffi::c_int;
    }
    return cy;
}

unsafe extern "C" fn update_min_band_height(infoPtr: *const REBAR_INFO, lpBand: *mut REBAR_BAND) {
    (*lpBand).cyMinBand = max(
        (*lpBand).cyHeader,
        if !(*lpBand).hwndChild.is_null() {
            (*lpBand).cyChild + REBARSPACE!(lpBand)
        } else {
            REBAR_NO_CHILD_HEIGHT
        },
    );
}

unsafe extern "C" fn REBAR_DrawChevron(mut hdc: HDC, mut left: INT, mut top: INT, mut colorRef: INT) {
    let mut x: INT;
    let mut y: INT;
    let mut hPen: HPEN;
    let mut hOldPen: HPEN;

    hPen = CreatePen(
        PS_SOLID as _,
        1 as _,
        GetSysColor(colorRef as _) as COLORREF,
    ) as HPEN;
    if hPen.is_null() {
        return;
    }
    hOldPen = SelectObject(hdc, hPen as HGDIOBJ) as HPEN;
    x = left + 2 as INT;
    y = top;
    MoveToEx(hdc, x as _, y as _, core::ptr::null_mut());
    let fresh15 = y;
    y = y + 1;
    LineTo(hdc, (x + 5 as INT) as _, fresh15 as _);
    x += 1;
    MoveToEx(hdc, x as _, y as _, core::ptr::null_mut());
    let fresh16 = y;
    y = y + 1;
    LineTo(hdc, (x + 3 as INT) as _, fresh16 as _);
    x += 1;
    MoveToEx(hdc, x as _, y as _, core::ptr::null_mut());
    LineTo(hdc, (x + 1 as INT) as _, y as _);
    SelectObject(hdc, hOldPen as HGDIOBJ);
    DeleteObject(hPen as HGDIOBJ);
}

unsafe extern "C" fn REBAR_GetNotifyParent(infoPtr: *const REBAR_INFO) -> HWND {
    let mut parent: HWND = core::ptr::null_mut::<HWND__>();
    let mut owner: HWND = core::ptr::null_mut::<HWND__>();
    parent = (*infoPtr).hwndNotify;
    if parent.is_null() {
        parent = GetParent((*infoPtr).hwndSelf);
        owner = GetWindow((*infoPtr).hwndSelf, GW_OWNER as UINT);
        if !owner.is_null() {
            parent = owner;
        }
    }
    return parent;
}

unsafe extern "C" fn REBAR_Notify(mut nmhdr: *mut NMHDR, infoPtr: *const REBAR_INFO, code: UINT) -> INT {
    let mut parent: HWND = core::ptr::null_mut();
    parent = REBAR_GetNotifyParent(infoPtr);
    (*nmhdr).idFrom = GetDlgCtrlID((*infoPtr).hwndSelf) as UINT_PTR;
    (*nmhdr).hwndFrom = (*infoPtr).hwndSelf;
    (*nmhdr).code = code;
    return SendMessageW(
        parent,
        WM_NOTIFY,
        (*nmhdr).idFrom as WPARAM,
        nmhdr as LPARAM,
    ) as INT;
}

unsafe extern "C" fn REBAR_Notify_NMREBAR(
    infoPtr: *const REBAR_INFO,
    uBand: UINT,
    code: UINT,
) -> INT {
    let mut notify_rebar: NMREBAR = tagNMREBAR {
        hdr: tagNMHDR {
            hwndFrom: ::core::ptr::null_mut::<HWND__>(),
            idFrom: 0,
            code: 0,
        },
        dwMask: 0,
        uBand: 0,
        fStyle: 0,
        wID: 0,
        lParam: 0,
    };
    notify_rebar.dwMask = 0 as DWORD;
    if uBand != (-(1 as ::core::ffi::c_int)) as UINT {
        let mut lpBand: *mut REBAR_BAND = REBAR_GetBand(infoPtr, uBand as INT);
        if ((*lpBand).fMask & (RBBIM_ID as UINT)) != 0 {
            notify_rebar.dwMask |= RBNM_ID as DWORD;
            notify_rebar.wID = (*lpBand).wID;
        }
        if ((*lpBand).fMask & (RBBIM_LPARAM as UINT)) != 0 {
            notify_rebar.dwMask |= RBNM_LPARAM as DWORD;
            notify_rebar.lParam = (*lpBand).lParam;
        }
        if ((*lpBand).fMask & (RBBIM_STYLE as UINT)) != 0 {
            notify_rebar.dwMask |= RBNM_STYLE as DWORD;
            notify_rebar.fStyle = (*lpBand).fStyle;
        }
    }
    notify_rebar.uBand = uBand;
    return REBAR_Notify(
        core::ptr::addr_of_mut!(notify_rebar) as *mut NMHDR,
        infoPtr,
        code,
    );
}

unsafe extern "C" fn REBAR_DrawBand(hdc: HDC, infoPtr: *const REBAR_INFO, lpBand: *mut REBAR_BAND) {
    let mut hOldFont: HFONT = core::ptr::null_mut();
    let mut oldBkMode: INT = 0;
    let mut nmcd: NMCUSTOMDRAW = core::mem::zeroed();
    let mut theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    let mut rcBand: RECT = core::mem::zeroed();

    translate_rect(infoPtr, core::ptr::addr_of_mut!(rcBand), core::ptr::addr_of!((*lpBand).rcBand));

    if ((*lpBand).fDraw & DRAW_TEXT) != 0 {
        hOldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;
        oldBkMode = SetBkMode(hdc, (TRANSPARENT as i32)) as INT;
    }

    /* should test for CDRF_NOTIFYITEMDRAW here */
    nmcd.dwDrawStage = CDDS_ITEMPREPAINT;
    nmcd.hdc = hdc;
    nmcd.rc = rcBand;
    nmcd.rc.right = (*lpBand).rcCapText.right;
    nmcd.rc.bottom = (*lpBand).rcCapText.bottom;
    nmcd.dwItemSpec = (*lpBand).wID as DWORD_PTR;
    nmcd.uItemState = 0;
    nmcd.lItemlParam = (*lpBand).lParam;
    (*lpBand).uCDret = REBAR_Notify(core::ptr::addr_of_mut!(nmcd) as *mut NMHDR, infoPtr, NM_CUSTOMDRAW) as UINT;
    if (*lpBand).uCDret == CDRF_SKIPDEFAULT {
        if oldBkMode != (TRANSPARENT as i32) {
            SetBkMode(hdc, oldBkMode);
        }
        SelectObject(hdc, hOldFont as HGDIOBJ);
        return;
    }

    /* draw gripper */
    if ((*lpBand).fDraw & DRAW_GRIPPER) != 0 {
        if !theme.is_null() {
            let mut rcGripper: RECT = (*lpBand).rcGripper;
            let mut partId: INT = if ((*infoPtr).dwStyle & CCS_VERT) != 0 {
                (RP_GRIPPERVERT) as i32
            } else {
                (RP_GRIPPER) as i32
            };
            GetThemeBackgroundExtent(theme, hdc, partId, 0, core::ptr::addr_of_mut!(rcGripper), core::ptr::addr_of_mut!(rcGripper));
            OffsetRect(core::ptr::addr_of_mut!(rcGripper), (*lpBand).rcGripper.left - rcGripper.left,
                (*lpBand).rcGripper.top - rcGripper.top);
            DrawThemeBackground(theme, hdc, partId, 0, core::ptr::addr_of_mut!(rcGripper), core::ptr::null());
        } else {
            DrawEdge(hdc, core::ptr::addr_of_mut!((*lpBand).rcGripper), BDR_RAISEDINNER, BF_RECT | BF_MIDDLE);
        }
    }

    /* draw caption image */
    if ((*lpBand).fDraw & DRAW_IMAGE) != 0 {
        let mut pt: POINT = core::mem::zeroed();

        /* center image */
        pt.y = ((*lpBand).rcCapImage.bottom + (*lpBand).rcCapImage.top - (*infoPtr).imageSize.cy) / 2;
        pt.x = ((*lpBand).rcCapImage.right + (*lpBand).rcCapImage.left - (*infoPtr).imageSize.cx) / 2;

        ImageList_Draw((*infoPtr).himl, (*lpBand).iImage, hdc,
            pt.x, pt.y,
            ILD_TRANSPARENT);
    }

    /* draw caption text */
    if ((*lpBand).fDraw & DRAW_TEXT) != 0 {
        /* need to handle CDRF_NEWFONT here */
        let mut oldBkMode: INT = SetBkMode(hdc, (TRANSPARENT as i32)) as INT;
        let mut oldcolor: COLORREF = CLR_NONE;
        let mut new: COLORREF = 0;
        if (*lpBand).clrFore != CLR_NONE {
            new = if (*lpBand).clrFore == CLR_DEFAULT {
                (*infoPtr).clrBtnText
            } else {
                (*lpBand).clrFore
            };
            oldcolor = SetTextColor(hdc, new);
        }

        if theme.is_null() {
            DrawTextW(hdc, (*lpBand).lpText, -1, core::ptr::addr_of_mut!((*lpBand).rcCapText), DT_CENTER | DT_VCENTER | DT_SINGLELINE);
        } else {
            DrawThemeText(theme, hdc, 0, 0, (*lpBand).lpText, -1, DT_CENTER | DT_VCENTER | DT_SINGLELINE, 0, core::ptr::addr_of_mut!((*lpBand).rcCapText));
        }

        if oldBkMode != (TRANSPARENT as i32) {
            SetBkMode(hdc, oldBkMode);
        }
        if (*lpBand).clrFore != CLR_NONE {
            SetTextColor(hdc, oldcolor);
        }
        SelectObject(hdc, hOldFont as HGDIOBJ);
    }

    if IsRectEmpty(core::ptr::addr_of!((*lpBand).rcChevron)) == 0 {
        if !theme.is_null() {
            let mut stateId: INT;
            if ((*lpBand).fDraw & DRAW_CHEVRONPUSHED) != 0 {
                stateId = (CHEVS_PRESSED) as i32;
            } else if ((*lpBand).fDraw & DRAW_CHEVRONHOT) != 0 {
                stateId = (CHEVS_HOT) as i32;
            } else {
                stateId = (CHEVS_NORMAL) as i32;
            }
            DrawThemeBackground(theme, hdc, (RP_CHEVRON) as i32, stateId, core::ptr::addr_of_mut!((*lpBand).rcChevron), core::ptr::null());
        } else {
            if ((*lpBand).fDraw & DRAW_CHEVRONPUSHED) != 0 {
                DrawEdge(hdc, core::ptr::addr_of_mut!((*lpBand).rcChevron), BDR_SUNKENOUTER, BF_RECT | BF_MIDDLE);
                REBAR_DrawChevron(hdc, (*lpBand).rcChevron.left + 1, (*lpBand).rcChevron.top + 11, (COLOR_WINDOWFRAME) as INT);
            } else if ((*lpBand).fDraw & DRAW_CHEVRONHOT) != 0 {
                DrawEdge(hdc, core::ptr::addr_of_mut!((*lpBand).rcChevron), BDR_RAISEDINNER, BF_RECT | BF_MIDDLE);
                REBAR_DrawChevron(hdc, (*lpBand).rcChevron.left, (*lpBand).rcChevron.top + 10, (COLOR_WINDOWFRAME) as INT);
            } else {
                REBAR_DrawChevron(hdc, (*lpBand).rcChevron.left, (*lpBand).rcChevron.top + 10, (COLOR_WINDOWFRAME) as INT);
            }
        }
    }

    if (*lpBand).uCDret == (CDRF_NOTIFYPOSTPAINT | CDRF_NOTIFYITEMDRAW) {
        nmcd.dwDrawStage = CDDS_ITEMPOSTPAINT;
        nmcd.hdc = hdc;
        nmcd.rc = rcBand;
        nmcd.rc.right = (*lpBand).rcCapText.right;
        nmcd.rc.bottom = (*lpBand).rcCapText.bottom;
        nmcd.dwItemSpec = (*lpBand).wID as DWORD_PTR;
        nmcd.uItemState = 0;
        nmcd.lItemlParam = (*lpBand).lParam;
        (*lpBand).uCDret = REBAR_Notify(core::ptr::addr_of_mut!(nmcd) as *mut NMHDR, infoPtr, NM_CUSTOMDRAW) as UINT;
    }
}

unsafe extern "C" fn REBAR_Refresh(infoPtr: *const REBAR_INFO, hdc: HDC) {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut::<REBAR_BAND>();
    let mut i: UINT = 0;
    if (*infoPtr).DoRedraw == 0 {
        return;
    }
    i = 0;
    while i < (*infoPtr).uNumBands {
        lpBand = REBAR_GetBand(infoPtr, i as INT);
        if !(HIDDENBAND!(infoPtr, lpBand) != false) {
            /* now draw the band */
            REBAR_DrawBand(hdc, infoPtr, lpBand);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn REBAR_CalcHorzBand(infoPtr: *const REBAR_INFO, rstart: UINT, rend: UINT) {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut();
    let mut i: UINT = 0;
    let mut xoff: UINT = 0;
    let mut work: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    i = rstart;
    while i < rend {
        lpBand = REBAR_GetBand(infoPtr, i as INT);
        if HIDDENBAND!(infoPtr, lpBand) != false {
            SetRect(
                core::ptr::addr_of_mut!((*lpBand).rcChild),
                (*lpBand).rcBand.right as ::core::ffi::c_int,
                (*lpBand).rcBand.top as ::core::ffi::c_int,
                (*lpBand).rcBand.right as ::core::ffi::c_int,
                (*lpBand).rcBand.bottom as ::core::ffi::c_int,
            );
        } else {
            /* set initial gripper rectangle */
            SetRect(
                core::ptr::addr_of_mut!((*lpBand).rcGripper),
                (*lpBand).rcBand.left as ::core::ffi::c_int,
                (*lpBand).rcBand.top as ::core::ffi::c_int,
                (*lpBand).rcBand.left as ::core::ffi::c_int,
                (*lpBand).rcBand.bottom as ::core::ffi::c_int,
            );

            /* calculate gripper rectangle */
            if (*lpBand).fStatus & HAS_GRIPPER as UINT != 0 {
                (*lpBand).fDraw |= DRAW_GRIPPER as UINT;
                (*lpBand).rcGripper.left += REBAR_PRE_GRIPPER as LONG;
                (*lpBand).rcGripper.right =
                    ((*lpBand).rcGripper.left as ::core::ffi::c_int
                        + GRIPPER_WIDTH as ::core::ffi::c_int) as LONG;
                InflateRect(
                    core::ptr::addr_of_mut!((*lpBand).rcGripper),
                    0 as ::core::ffi::c_int,
                    -(2 as ::core::ffi::c_int),
                );
                SetRect(
                    core::ptr::addr_of_mut!((*lpBand).rcCapImage),
                    (*lpBand).rcGripper.right as ::core::ffi::c_int
                        + REBAR_ALWAYS_SPACE as ::core::ffi::c_int,
                    (*lpBand).rcBand.top as ::core::ffi::c_int,
                    (*lpBand).rcGripper.right as ::core::ffi::c_int
                        + REBAR_ALWAYS_SPACE as ::core::ffi::c_int,
                    (*lpBand).rcBand.bottom as ::core::ffi::c_int,
                );
            } else {
                /* no gripper will be drawn */
                xoff = 0 as UINT;
                if (*lpBand).fStatus & (HAS_IMAGE as UINT | HAS_TEXT as UINT) != 0 {
                    /* if no gripper but either image or text, then leave space */
                    xoff = REBAR_ALWAYS_SPACE as UINT;
                }
                SetRect(
                    core::ptr::addr_of_mut!((*lpBand).rcCapImage),
                    ((*lpBand).rcBand.left as UINT).wrapping_add(xoff) as ::core::ffi::c_int,
                    (*lpBand).rcBand.top as ::core::ffi::c_int,
                    ((*lpBand).rcBand.left as UINT).wrapping_add(xoff) as ::core::ffi::c_int,
                    (*lpBand).rcBand.bottom as ::core::ffi::c_int,
                );
            }

            /* image is visible */
            if (*lpBand).fStatus & HAS_IMAGE as UINT != 0 {
                (*lpBand).fDraw |= DRAW_IMAGE as UINT;
                (*lpBand).rcCapImage.right += (*infoPtr).imageSize.cx;
                (*lpBand).rcCapImage.bottom =
                    (*lpBand).rcCapImage.top + (*infoPtr).imageSize.cy;

                /* set initial caption text rectangle */
                SetRect(
                    core::ptr::addr_of_mut!((*lpBand).rcCapText),
                    (*lpBand).rcCapImage.right as ::core::ffi::c_int
                        + REBAR_POST_IMAGE as ::core::ffi::c_int,
                    (*lpBand).rcBand.top as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int,
                    ((*lpBand).rcBand.left as UINT).wrapping_add((*lpBand).cxHeader)
                        as ::core::ffi::c_int,
                    (*lpBand).rcBand.bottom as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int,
                );
            } else {
                /* set initial caption text rectangle */
                SetRect(
                    core::ptr::addr_of_mut!((*lpBand).rcCapText),
                    (*lpBand).rcCapImage.right as ::core::ffi::c_int,
                    (*lpBand).rcBand.top as ::core::ffi::c_int
                        + 1 as ::core::ffi::c_int,
                    ((*lpBand).rcBand.left as UINT).wrapping_add((*lpBand).cxHeader)
                        as ::core::ffi::c_int,
                    (*lpBand).rcBand.bottom as ::core::ffi::c_int
                        - 1 as ::core::ffi::c_int,
                );
            }

            /* text is visible */
            if (*lpBand).fStatus & HAS_TEXT as UINT != 0
                && !((*lpBand).fStyle & RBBS_HIDETITLE as UINT != 0)
            {
                (*lpBand).fDraw |= DRAW_TEXT as UINT;
                (*lpBand).rcCapText.right = max(
                    (*lpBand).rcCapText.left,
                    (*lpBand).rcCapText.right - REBAR_POST_TEXT as LONG,
                );
            }

            /* set initial child window rectangle if there is a child */
            if !(*lpBand).hwndChild.is_null() {
                (*lpBand).rcChild.left =
                    ((*lpBand).rcBand.left as UINT).wrapping_add((*lpBand).cxHeader) as LONG;
                (*lpBand).rcChild.right =
                    (*lpBand).rcBand.right - REBAR_POST_CHILD as LONG;

                if (*lpBand).cyChild > 0 as UINT {
                    let mut yoff: UINT = (((*lpBand).rcBand.bottom - (*lpBand).rcBand.top) as UINT)
                        .wrapping_sub((*lpBand).cyChild)
                        .wrapping_div(2 as UINT);

                    /* center child if height is known */
                    (*lpBand).rcChild.top =
                        ((*lpBand).rcBand.top as UINT).wrapping_add(yoff) as LONG;
                    (*lpBand).rcChild.bottom = ((*lpBand).rcBand.top as UINT)
                        .wrapping_add(yoff)
                        .wrapping_add((*lpBand).cyChild) as LONG;
                } else {
                    (*lpBand).rcChild.top = (*lpBand).rcBand.top;
                    (*lpBand).rcChild.bottom = (*lpBand).rcBand.bottom;
                }

                if (*lpBand).fStyle & RBBS_USECHEVRON as UINT != 0
                    && (((*lpBand).rcChild.right - (*lpBand).rcChild.left) as UINT)
                        < (*lpBand).cxIdeal
                {
                    (*lpBand).rcChild.right -= CHEVRON_WIDTH as LONG;
                    SetRect(
                        core::ptr::addr_of_mut!((*lpBand).rcChevron),
                        (*lpBand).rcChild.right as ::core::ffi::c_int,
                        (*lpBand).rcChild.top as ::core::ffi::c_int,
                        (*lpBand).rcChild.right as ::core::ffi::c_int
                            + CHEVRON_WIDTH as ::core::ffi::c_int,
                        (*lpBand).rcChild.bottom as ::core::ffi::c_int,
                    );
                }
            } else {
                SetRect(
                    core::ptr::addr_of_mut!((*lpBand).rcChild),
                    ((*lpBand).rcBand.left as UINT).wrapping_add((*lpBand).cxHeader)
                        as ::core::ffi::c_int,
                    (*lpBand).rcBand.top as ::core::ffi::c_int,
                    (*lpBand).rcBand.right as ::core::ffi::c_int,
                    (*lpBand).rcBand.bottom as ::core::ffi::c_int,
                );
            }

            /* flag if notify required and invalidate rectangle */
            if (*lpBand).fDraw & NTF_INVALIDATE as UINT != 0 {
                (*lpBand).fDraw &= !(NTF_INVALIDATE as UINT);
                work = (*lpBand).rcBand;
                work.right += SEP_WIDTH!(infoPtr) as LONG;
                work.bottom += SEP_WIDTH!(infoPtr) as LONG;

                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/comctl32/rebar.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"REBAR_CalcHorzBand\0" as *const u8 as *const ::core::ffi::c_char,
                        836 as ::core::ffi::c_int,
                        b"invalidating %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                        wine_dbgstr_rect(core::ptr::addr_of_mut!(work)),
                    );
                }
                InvalidateRect(
                    (*infoPtr).hwndSelf,
                    core::ptr::addr_of_mut!(work),
                    TRUE as BOOL,
                );
                if !(*lpBand).hwndChild.is_null() {
                    InvalidateRect(
                        (*lpBand).hwndChild,
                        core::ptr::null::<RECT>(),
                        TRUE as BOOL,
                    );
                }
            }
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn REBAR_CalcVertBand(
    mut infoPtr: *const REBAR_INFO,
    mut rstart: UINT,
    mut rend: UINT,
) {
    let mut lpBand: *mut REBAR_BAND = ::core::ptr::null_mut::<REBAR_BAND>();
    let mut i: UINT = 0;
    let mut xoff: UINT = 0;
    let mut work: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    i = rstart;
    while i < rend {
        let mut rcBand: RECT = tagRECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        lpBand = REBAR_GetBand(infoPtr, i as INT);
        if !((*lpBand).fStyle & RBBS_HIDDEN as UINT != 0
            || (*infoPtr).dwStyle as DWORD & CCS_VERT as DWORD != 0
                && (*lpBand).fStyle & RBBS_NOVERT as UINT != 0)
        {
            translate_rect(infoPtr, &raw mut rcBand, &raw mut (*lpBand).rcBand);

            /* set initial gripper rectangle */
            SetRect(
                &raw mut (*lpBand).rcGripper,
                rcBand.left as ::core::ffi::c_int,
                rcBand.top as ::core::ffi::c_int,
                rcBand.right as ::core::ffi::c_int,
                rcBand.top as ::core::ffi::c_int,
            );

            /* calculate gripper rectangle */
            if (*lpBand).fStatus & (HAS_GRIPPER) as UINT != 0 {
                (*lpBand).fDraw |= (DRAW_GRIPPER) as UINT;

                if (*infoPtr).dwStyle & (RBS_VERTICALGRIPPER) as DWORD != 0 {
                    /*  vertical gripper  */
                    (*lpBand).rcGripper.left += (3) as ::core::ffi::c_int;
                    (*lpBand).rcGripper.right =
                        ((*lpBand).rcGripper.left as ::core::ffi::c_int
                            + (GRIPPER_WIDTH) as ::core::ffi::c_int) as LONG;
                    (*lpBand).rcGripper.top += (REBAR_PRE_GRIPPER) as ::core::ffi::c_int;
                    (*lpBand).rcGripper.bottom =
                        ((*lpBand).rcGripper.top as ::core::ffi::c_int
                            + (GRIPPER_HEIGHT) as ::core::ffi::c_int) as LONG;

                    /* initialize Caption image rectangle  */
                    SetRect(
                        &raw mut (*lpBand).rcCapImage,
                        rcBand.left as ::core::ffi::c_int,
                        (*lpBand).rcGripper.bottom as ::core::ffi::c_int
                            + (REBAR_ALWAYS_SPACE) as ::core::ffi::c_int,
                        rcBand.right as ::core::ffi::c_int,
                        (*lpBand).rcGripper.bottom as ::core::ffi::c_int
                            + (REBAR_ALWAYS_SPACE) as ::core::ffi::c_int,
                    );
                } else {
                    /*  horizontal gripper  */
                    InflateRect(
                        &raw mut (*lpBand).rcGripper,
                        -((2) as ::core::ffi::c_int),
                        (0) as ::core::ffi::c_int,
                    );
                    (*lpBand).rcGripper.top += (REBAR_PRE_GRIPPER) as ::core::ffi::c_int;
                    (*lpBand).rcGripper.bottom =
                        ((*lpBand).rcGripper.top as ::core::ffi::c_int
                            + (GRIPPER_WIDTH) as ::core::ffi::c_int) as LONG;

                    /* initialize Caption image rectangle  */
                    SetRect(
                        &raw mut (*lpBand).rcCapImage,
                        rcBand.left as ::core::ffi::c_int,
                        (*lpBand).rcGripper.bottom as ::core::ffi::c_int
                            + (REBAR_ALWAYS_SPACE) as ::core::ffi::c_int,
                        rcBand.right as ::core::ffi::c_int,
                        (*lpBand).rcGripper.bottom as ::core::ffi::c_int
                            + (REBAR_ALWAYS_SPACE) as ::core::ffi::c_int,
                    );
                }
            } else {
                /* no gripper will be drawn */
                xoff = (0) as UINT;
                if (*lpBand).fStatus & ((HAS_IMAGE) | (HAS_TEXT)) as UINT
                    != 0
                {
                    /* if no gripper but either image or text, then leave space */
                    xoff = (REBAR_ALWAYS_SPACE) as UINT;
                }
                /* initialize Caption image rectangle  */
                SetRect(
                    &raw mut (*lpBand).rcCapImage,
                    rcBand.left as ::core::ffi::c_int,
                    (rcBand.top as UINT).wrapping_add(xoff) as ::core::ffi::c_int,
                    rcBand.right as ::core::ffi::c_int,
                    (rcBand.top as UINT).wrapping_add(xoff) as ::core::ffi::c_int,
                );
            }

            /* image is visible */
            if (*lpBand).fStatus & (HAS_IMAGE) as UINT != 0 {
                (*lpBand).fDraw |= (DRAW_IMAGE) as UINT;

                (*lpBand).rcCapImage.right =
                    (*lpBand).rcCapImage.left + (*infoPtr).imageSize.cx;
                (*lpBand).rcCapImage.bottom += (*infoPtr).imageSize.cy;

                /* set initial caption text rectangle */
                SetRect(
                    &raw mut (*lpBand).rcCapText,
                    rcBand.left as ::core::ffi::c_int,
                    (*lpBand).rcCapImage.bottom as ::core::ffi::c_int
                        + (REBAR_POST_IMAGE) as ::core::ffi::c_int,
                    rcBand.right as ::core::ffi::c_int,
                    (rcBand.top as UINT).wrapping_add((*lpBand).cxHeader) as ::core::ffi::c_int,
                );
            } else {
                /* set initial caption text rectangle */
                SetRect(
                    &raw mut (*lpBand).rcCapText,
                    rcBand.left as ::core::ffi::c_int,
                    (*lpBand).rcCapImage.bottom as ::core::ffi::c_int,
                    rcBand.right as ::core::ffi::c_int,
                    (rcBand.top as UINT).wrapping_add((*lpBand).cxHeader) as ::core::ffi::c_int,
                );
            }

            /* text is visible */
            if (*lpBand).fStatus & (HAS_TEXT) as UINT != 0
                && !((*lpBand).fStyle & (RBBS_HIDETITLE) as UINT != 0)
            {
                (*lpBand).fDraw |= (DRAW_TEXT) as UINT;
                (*lpBand).rcCapText.bottom = max(
                    (*lpBand).rcCapText.top,
                    (*lpBand).rcCapText.bottom,
                );
            }

            /* set initial child window rectangle if there is a child */
            if !(*lpBand).hwndChild.is_null() {
                let mut cxBand: ::core::ffi::c_int =
                    rcBand.right as ::core::ffi::c_int - rcBand.left as ::core::ffi::c_int;
                xoff = (cxBand as UINT)
                    .wrapping_sub((*lpBand).cyChild)
                    .wrapping_div((2) as UINT);
                SetRect(
                    &raw mut (*lpBand).rcChild,
                    (rcBand.left as UINT).wrapping_add(xoff) as ::core::ffi::c_int,
                    (rcBand.top as UINT).wrapping_add((*lpBand).cxHeader) as ::core::ffi::c_int,
                    (rcBand.left as UINT)
                        .wrapping_add(xoff)
                        .wrapping_add((*lpBand).cyChild) as ::core::ffi::c_int,
                    rcBand.bottom as ::core::ffi::c_int
                        - (REBAR_POST_CHILD) as ::core::ffi::c_int,
                );
            } else {
                SetRect(
                    &raw mut (*lpBand).rcChild,
                    rcBand.left as ::core::ffi::c_int,
                    (rcBand.top as UINT).wrapping_add((*lpBand).cxHeader) as ::core::ffi::c_int,
                    rcBand.right as ::core::ffi::c_int,
                    rcBand.bottom as ::core::ffi::c_int,
                );
            }

            if (*lpBand).fDraw & (NTF_INVALIDATE) as UINT != 0 {
                (*lpBand).fDraw &= !((NTF_INVALIDATE) as UINT);
                work = rcBand;
                work.bottom += (SEP_WIDTH!(infoPtr)) as ::core::ffi::c_int;
                work.right += (SEP_WIDTH!(infoPtr)) as ::core::ffi::c_int;

                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/comctl32/rebar.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"REBAR_CalcVertBand\0" as *const u8
                            as *const ::core::ffi::c_char,
                        952 as ::core::ffi::c_int,
                        b"invalidating %s\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        wine_dbgstr_rect(&raw mut work),
                    );
                }

                InvalidateRect(
                    (*infoPtr).hwndSelf,
                    &raw mut work,
                    (TRUE) as BOOL,
                );
                if !(*lpBand).hwndChild.is_null() {
                    InvalidateRect(
                        (*lpBand).hwndChild,
                        ::core::ptr::null::<RECT>(),
                        (TRUE) as BOOL,
                    );
                }
            }
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn REBAR_ForceResize(infoPtr: *mut REBAR_INFO) {
    let mut x: INT = 0;
    let mut y: INT = 0;
    let mut width: INT = 0;
    let mut height: INT = 0;
    let mut xedge: INT = 0;
    let mut yedge: INT = 0;
    let mut rcSelf: RECT = core::mem::zeroed();

    if ((*infoPtr).dwStyle & CCS_NORESIZE) != 0 {
        return;
    }

    if ((*infoPtr).dwStyle & WS_BORDER) != 0 {
        xedge = GetSystemMetrics((SM_CXEDGE) as i32) as INT;
        yedge = GetSystemMetrics((SM_CYEDGE) as i32) as INT;
        /* swap for CCS_VERT? */
    }

    /* compute rebar window rect in parent client coordinates */
    GetWindowRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcSelf));
    MapWindowPoints(
        HWND_DESKTOP,
        GetParent((*infoPtr).hwndSelf),
        core::ptr::addr_of_mut!(rcSelf) as LPPOINT,
        2,
    );
    translate_rect(
        infoPtr,
        core::ptr::addr_of_mut!(rcSelf),
        core::ptr::addr_of_mut!(rcSelf),
    );

    height = (*infoPtr).calcSize.cy as INT + 2 * yedge;
    if ((*infoPtr).dwStyle & CCS_NOPARENTALIGN) == 0 {
        let mut rcParent: RECT = core::mem::zeroed();

        x = -xedge;
        width = (*infoPtr).calcSize.cx as INT + 2 * xedge;
        y = 0; /* quiet compiler warning */
        match (*infoPtr).dwStyle & CCS_LAYOUT_MASK {
            0 | CCS_TOP => {
                /* shouldn't happen - see NCCreate */
                y = ((if ((*infoPtr).dwStyle & CCS_NODIVIDER) != 0 {
                    0
                } else {
                    REBAR_DIVIDER
                }) - (yedge) as u32) as i32;
            }
            CCS_NOMOVEY => {
                y = rcSelf.top as INT;
            }
            CCS_BOTTOM => {
                GetClientRect(
                    GetParent((*infoPtr).hwndSelf),
                    core::ptr::addr_of_mut!(rcParent),
                );
                translate_rect(
                    infoPtr,
                    core::ptr::addr_of_mut!(rcParent),
                    core::ptr::addr_of_mut!(rcParent),
                );
                y = rcParent.bottom as INT - (*infoPtr).calcSize.cy as INT - yedge;
            }
            _ => {}
        }
    } else {
        x = rcSelf.left as INT;
        /* As on Windows if the CCS_NODIVIDER is not present the control will move
         * 2 pixel down after every layout */
        y = (rcSelf.top as INT
            + ((if ((*infoPtr).dwStyle & CCS_NODIVIDER) != 0 {
                0
            } else {
                REBAR_DIVIDER
            })) as i32) as INT;
        width = (rcSelf.right - rcSelf.left) as INT;
    }

    /* Set flag to ignore next WM_SIZE message and resize the window */
    (*infoPtr).fStatus |= SELF_RESIZE;
    if ((*infoPtr).dwStyle & CCS_VERT) == 0 {
        SetWindowPos(
            (*infoPtr).hwndSelf,
            core::ptr::null_mut(),
            x,
            y,
            width,
            height,
            SWP_NOZORDER,
        );
    } else {
        SetWindowPos(
            (*infoPtr).hwndSelf,
            core::ptr::null_mut(),
            y,
            x,
            height,
            width,
            SWP_NOZORDER,
        );
    }
    (*infoPtr).fStatus &= !SELF_RESIZE;
}

unsafe extern "C" fn REBAR_MoveChildWindows(
    infoPtr: *const REBAR_INFO,
    start: UINT,
    endplus: UINT,
) {
    static strComboBox: [WCHAR; 9] = [0x0043u16, 0x006F, 0x006D, 0x0062, 0x006F, 0x0042, 0x006F, 0x0078, 0x0000];
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut::<REBAR_BAND>();
    let mut szClassName: [WCHAR; 40] = [0; 40];
    let mut i: UINT = 0;
    let mut rbcz: NMREBARCHILDSIZE = core::mem::zeroed();
    let mut deferpos: HDWP = core::ptr::null_mut::<HDWP__>();

    deferpos = BeginDeferWindowPos((*infoPtr).uNumBands as INT);
    if deferpos.is_null() {
        if (*__wine_dbch___default).flags as INT
            & (1 as INT) << __WINE_DBCL_ERR as INT
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_MoveChildWindows\0" as *const u8 as *const ::core::ffi::c_char,
                1041 as INT,
                b"BeginDeferWindowPos returned NULL\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    }

    i = start;
    while i < endplus {
        lpBand = REBAR_GetBand(infoPtr, i as INT);

        if !((*lpBand).fStyle & RBBS_HIDDEN as UINT != 0
            || (*infoPtr).dwStyle as u32 & CCS_VERT as u32 != 0
                && (*lpBand).fStyle & RBBS_NOVERT as UINT != 0)
        {
            if !(*lpBand).hwndChild.is_null() {
                if (*__wine_dbch___default).flags as INT
                    & (1 as INT) << __WINE_DBCL_TRACE as INT
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/comctl32/rebar.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"REBAR_MoveChildWindows\0" as *const u8 as *const ::core::ffi::c_char,
                        1048 as INT,
                        b"hwndChild = %p\n\0" as *const u8 as *const ::core::ffi::c_char,
                        (*lpBand).hwndChild,
                    );
                }

                rbcz.uBand = i;
                rbcz.wID = (*lpBand).wID;
                rbcz.rcChild = (*lpBand).rcChild;
                translate_rect(
                    infoPtr,
                    core::ptr::addr_of_mut!(rbcz.rcBand),
                    core::ptr::addr_of_mut!((*lpBand).rcBand),
                );
                if (*infoPtr).dwStyle as u32 & CCS_VERT as u32 != 0 {
                    rbcz.rcBand.top =
                        (rbcz.rcBand.top as UINT).wrapping_add((*lpBand).cxHeader) as LONG;
                } else {
                    rbcz.rcBand.left =
                        (rbcz.rcBand.left as UINT).wrapping_add((*lpBand).cxHeader) as LONG;
                }
                REBAR_Notify(
                    core::ptr::addr_of_mut!(rbcz) as *mut NMHDR,
                    infoPtr,
                    (RBN_CHILDSIZE) as UINT,
                );
                if EqualRect(
                    core::ptr::addr_of_mut!((*lpBand).rcChild) as LPCRECT,
                    core::ptr::addr_of_mut!(rbcz.rcChild) as LPCRECT,
                ) == 0
                {
                    if (*__wine_dbch___default).flags as INT
                        & (1 as INT) << __WINE_DBCL_TRACE as INT
                        != 0
                    {
                        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                        ros_dbg_log(
                            __dbcl_1,
                            __dbch_1,
                            b"dll/win32/comctl32/rebar.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"REBAR_MoveChildWindows\0" as *const u8 as *const ::core::ffi::c_char,
                            1062 as INT,
                            b"Child rect changed by NOTIFY for band %u\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            i,
                        );
                    }
                    if (*__wine_dbch___default).flags as INT
                        & (1 as INT) << __WINE_DBCL_TRACE as INT
                        != 0
                    {
                        let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
                        ros_dbg_log(
                            __dbcl_2,
                            __dbch_2,
                            b"dll/win32/comctl32/rebar.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"REBAR_MoveChildWindows\0" as *const u8 as *const ::core::ffi::c_char,
                            1065 as INT,
                            b"    from (%s)  to (%s)\n\0" as *const u8
                                as *const ::core::ffi::c_char,
                            wine_dbgstr_rect(core::ptr::addr_of_mut!((*lpBand).rcChild)),
                            wine_dbgstr_rect(core::ptr::addr_of_mut!(rbcz.rcChild)),
                        );
                    }
                    (*lpBand).rcChild = rbcz.rcChild;
                }

                GetClassNameW(
                    (*lpBand).hwndChild,
                    core::ptr::addr_of_mut!(szClassName) as LPWSTR,
                    (core::mem::size_of::<[WCHAR; 40]>()
                        / core::mem::size_of::<WCHAR>()) as INT,
                );
                if lstrcmpW(
                    core::ptr::addr_of_mut!(szClassName) as *mut WCHAR as LPCWSTR,
                    core::ptr::addr_of!(strComboBox) as LPCWSTR,
                ) == 0
                    || lstrcmpW(
                        core::ptr::addr_of_mut!(szClassName) as *mut WCHAR as LPCWSTR,
                        (WC_COMBOBOXEXW).as_ptr(),
                    ) == 0
                {
                    let mut nEditHeight: INT = 0;
                    let mut yPos: INT = 0;
                    let mut rc: RECT = core::mem::zeroed();

                    GetWindowRect((*lpBand).hwndChild, core::ptr::addr_of_mut!(rc));
                    nEditHeight = (rc.bottom - rc.top) as INT;
                    yPos = (((*lpBand).rcChild.bottom as INT
                        + (*lpBand).rcChild.top as INT
                        - nEditHeight as INT)
                        / 2 as INT) as INT;

                    if (*__wine_dbch___default).flags as INT
                        & (1 as INT) << __WINE_DBCL_TRACE as INT
                        != 0
                    {
                        let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
                        ros_dbg_log(
                            __dbcl_3,
                            __dbch_3,
                            b"dll/win32/comctl32/rebar.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"REBAR_MoveChildWindows\0" as *const u8 as *const ::core::ffi::c_char,
                            1088 as INT,
                            b"moving child (Combo(Ex)) %p to (%d,%d) for (%d,%d)\n\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            (*lpBand).hwndChild,
                            (*lpBand).rcChild.left,
                            yPos,
                            (*lpBand).rcChild.right - (*lpBand).rcChild.left,
                            nEditHeight,
                        );
                    }

                    deferpos = DeferWindowPos(
                        deferpos,
                        (*lpBand).hwndChild,
                        HWND_TOP,
                        (*lpBand).rcChild.left as INT,
                        yPos as INT,
                        (*lpBand).rcChild.right as INT - (*lpBand).rcChild.left as INT,
                        nEditHeight as INT,
                        SWP_NOZORDER as UINT,
                    );
                    if deferpos.is_null() {
                        if (*__wine_dbch___default).flags as INT
                            & (1 as INT) << __WINE_DBCL_ERR as INT
                            != 0
                        {
                            let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                            let __dbcl_4: __wine_debug_class = __WINE_DBCL_ERR;
                            ros_dbg_log(
                                __dbcl_4,
                                __dbch_4,
                                b"dll/win32/comctl32/rebar.c\0"
                                    as *const u8 as *const ::core::ffi::c_char,
                                b"REBAR_MoveChildWindows\0" as *const u8 as *const ::core::ffi::c_char,
                                1096 as INT,
                                b"DeferWindowPos returned NULL\n\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    }
                } else {
                    if (*__wine_dbch___default).flags as INT
                        & (1 as INT) << __WINE_DBCL_TRACE as INT
                        != 0
                    {
                        let __dbch_5: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_5: __wine_debug_class = __WINE_DBCL_TRACE;
                        ros_dbg_log(
                            __dbcl_5,
                            __dbch_5,
                            b"dll/win32/comctl32/rebar.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"REBAR_MoveChildWindows\0" as *const u8 as *const ::core::ffi::c_char,
                            1103 as INT,
                            b"moving child (Other) %p to (%d,%d) for (%d,%d)\n\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            (*lpBand).hwndChild,
                            (*lpBand).rcChild.left,
                            (*lpBand).rcChild.top,
                            (*lpBand).rcChild.right - (*lpBand).rcChild.left,
                            (*lpBand).rcChild.bottom - (*lpBand).rcChild.top,
                        );
                    }

                    deferpos = DeferWindowPos(
                        deferpos,
                        (*lpBand).hwndChild,
                        HWND_TOP,
                        (*lpBand).rcChild.left as INT,
                        (*lpBand).rcChild.top as INT,
                        (*lpBand).rcChild.right as INT - (*lpBand).rcChild.left as INT,
                        (*lpBand).rcChild.bottom as INT - (*lpBand).rcChild.top as INT,
                        SWP_NOZORDER as UINT,
                    );
                    if deferpos.is_null() {
                        if (*__wine_dbch___default).flags as INT
                            & (1 as INT) << __WINE_DBCL_ERR as INT
                            != 0
                        {
                            let __dbch_6: *mut __wine_debug_channel = __wine_dbch___default;
                            let __dbcl_6: __wine_debug_class = __WINE_DBCL_ERR;
                            ros_dbg_log(
                                __dbcl_6,
                                __dbch_6,
                                b"dll/win32/comctl32/rebar.c\0"
                                    as *const u8 as *const ::core::ffi::c_char,
                                b"REBAR_MoveChildWindows\0" as *const u8 as *const ::core::ffi::c_char,
                                1111 as INT,
                                b"DeferWindowPos returned NULL\n\0" as *const u8
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    }
                }
            }
        }
        i = i.wrapping_add(1);
    }

    if EndDeferWindowPos(deferpos) == 0 {
        if (*__wine_dbch___default).flags as INT
            & (1 as INT) << __WINE_DBCL_ERR as INT
            != 0
        {
            let __dbch_7: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_7: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_7,
                __dbch_7,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_MoveChildWindows\0" as *const u8 as *const ::core::ffi::c_char,
                1116 as INT,
                b"EndDeferWindowPos returned NULL\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    }

    if (*infoPtr).DoRedraw != 0 {
        UpdateWindow((*infoPtr).hwndSelf);
    }
}

unsafe extern "C" fn next_visible(infoPtr: *const REBAR_INFO, i: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_uint;
    n = (i + 1) as ::core::ffi::c_uint;
    while n < (*infoPtr).uNumBands {
        if !HIDDENBAND!(infoPtr, REBAR_GetBand(infoPtr, n as INT)) {
            break;
        }
        n = n.wrapping_add(1);
    }
    return n as ::core::ffi::c_int;
}

unsafe extern "C" fn prev_visible(infoPtr: *const REBAR_INFO, i: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut n: ::core::ffi::c_int = i - 1;
    while n >= 0 {
        if !HIDDENBAND!(infoPtr, REBAR_GetBand(infoPtr, n as INT)) {
            break;
        }
        n -= 1;
    }
    return n;
}

unsafe extern "C" fn first_visible(infoPtr: *const REBAR_INFO) -> core::ffi::c_int {
    return next_visible(infoPtr, -1);
}

unsafe extern "C" fn get_row_begin_for_band(infoPtr: *const REBAR_INFO, mut iBand: INT) -> i32 {
    let mut iLastBand: i32 = iBand as i32;
    let mut iRow: i32 = (*REBAR_GetBand(infoPtr, iBand)).iRow as i32;
    loop {
        iBand = prev_visible(infoPtr, iBand) as INT;
        if !(iBand >= 0) {
            break;
        }
        if (*REBAR_GetBand(infoPtr, iBand)).iRow as i32 != iRow {
            break;
        }
        iLastBand = iBand as i32;
    }
    return iLastBand;
}

unsafe extern "C" fn get_row_end_for_band(infoPtr: *const REBAR_INFO, mut iBand: INT) -> ::core::ffi::c_int {
    let iRow: ::core::ffi::c_int = (*REBAR_GetBand(infoPtr, iBand)).iRow as ::core::ffi::c_int;
    loop {
        iBand = next_visible(infoPtr, iBand) as INT;
        if !((iBand as UINT) < (*infoPtr).uNumBands) {
            break;
        }
        if (*REBAR_GetBand(infoPtr, iBand)).iRow as ::core::ffi::c_int != iRow {
            break;
        }
    }
    return iBand as ::core::ffi::c_int;
}

unsafe extern "C" fn REBAR_SetRowRectsX(infoPtr: *const REBAR_INFO, iBeginBand: INT, iEndBand: INT) {
    let mut xPos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 0;
    i = iBeginBand as ::core::ffi::c_int;
    while i < iEndBand {
        let mut lpBand: *mut REBAR_BAND = REBAR_GetBand(infoPtr, i as INT);
        if (*lpBand).rcBand.left != xPos
            || (*lpBand).rcBand.right != xPos as INT + (*lpBand).cxEffective
        {
            (*lpBand).fDraw |= NTF_INVALIDATE as UINT;
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/rebar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"REBAR_SetRowRectsX\0" as *const u8 as *const ::core::ffi::c_char,
                    1184 as ::core::ffi::c_int,
                    b"Setting rect %d to %d,%d\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    i,
                    xPos,
                    xPos as INT + (*lpBand).cxEffective,
                );
            }
            (*lpBand).rcBand.left = xPos as LONG;
            (*lpBand).rcBand.right = (xPos as INT + (*lpBand).cxEffective) as LONG;
        }
        xPos += (*lpBand).cxEffective as ::core::ffi::c_int + SEP_WIDTH!(infoPtr) as ::core::ffi::c_int;
        i = next_visible(infoPtr, i as INT) as ::core::ffi::c_int;
    }
}

unsafe extern "C" fn REBAR_FindBandToGrow(infoPtr: *const REBAR_INFO, iBeginBand: INT, iEndBand: INT) -> *mut REBAR_BAND {
    let mut cxMinFirstBand: INT = 0 as INT;
    let mut i: INT = 0;
    cxMinFirstBand = (*REBAR_GetBand(infoPtr, iBeginBand)).cxMinBand as INT;
    i = prev_visible(infoPtr, iEndBand as ::core::ffi::c_int) as INT;
    while i >= iBeginBand {
        if (*REBAR_GetBand(infoPtr, i)).cxEffective > cxMinFirstBand
            && ((*REBAR_GetBand(infoPtr, i)).fStyle & (RBBS_FIXEDSIZE as UINT)) == 0
        {
            break;
        }
        i = prev_visible(infoPtr, i as ::core::ffi::c_int) as INT;
    }
    if i < iBeginBand {
        i = prev_visible(infoPtr, iEndBand as ::core::ffi::c_int) as INT;
        while i >= iBeginBand {
            if (*REBAR_GetBand(infoPtr, i)).cxMinBand == cxMinFirstBand as UINT {
                break;
            }
            i = prev_visible(infoPtr, i as ::core::ffi::c_int) as INT;
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
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_FindBandToGrow\0" as *const u8 as *const ::core::ffi::c_char,
            1214 as ::core::ffi::c_int,
            b"Extra space for row [%d..%d) should be added to band %d\n\0" as *const u8
                as *const ::core::ffi::c_char,
            iBeginBand,
            iEndBand,
            i,
        );
    }
    return REBAR_GetBand(infoPtr, i);
}

unsafe extern "C" fn REBAR_ShrinkBandsRTL(infoPtr: *const REBAR_INFO, iBeginBand: INT, iEndBand: INT, mut cxShrink: INT, bEnforce: BOOL) -> INT {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut();
    let mut width: INT = 0;
    let mut i: INT = 0;
    i = prev_visible(infoPtr, iEndBand);
    while i >= iBeginBand {
        lpBand = REBAR_GetBand(infoPtr, i);

        width = max((*lpBand).cxEffective - cxShrink, (*lpBand).cxMinBand as INT);
        cxShrink -= (*lpBand).cxEffective - width;
        (*lpBand).cxEffective = width;
        if bEnforce != 0 && (*lpBand).cx > (*lpBand).cxEffective as UINT {
            (*lpBand).cx = (*lpBand).cxEffective as UINT;
        }
        if cxShrink == 0 {
            break;
        }
        i = prev_visible(infoPtr, i);
    }
    return cxShrink;
}

unsafe extern "C" fn REBAR_ShrinkBandsLTR(infoPtr: *const REBAR_INFO, iBeginBand: INT, iEndBand: INT, mut cxShrink: INT, bEnforce: BOOL) -> INT {
    let mut lpBand: *mut REBAR_BAND;
    let mut width: INT;
    let mut i: INT;
    i = iBeginBand;
    while i < iEndBand {
        lpBand = REBAR_GetBand(infoPtr, i);

        width = max((*lpBand).cxEffective - cxShrink, (*lpBand).cxMinBand as INT);
        cxShrink -= (*lpBand).cxEffective - width;
        (*lpBand).cxEffective = width;
        if bEnforce != 0 {
            (*lpBand).cx = (*lpBand).cxEffective as UINT;
        }
        if cxShrink == 0 {
            break;
        }
        i = next_visible(infoPtr, i);
    }
    return cxShrink;
}

unsafe extern "C" fn REBAR_MoveBandToRowOffset(
    infoPtr: *mut REBAR_INFO,
    iBand: INT,
    iFirstBand: INT,
    iLastBand: INT,
    xOff: INT,
    reorder: BOOL,
) -> ::core::ffi::c_int {
    let insertBand: *mut REBAR_BAND = REBAR_GetBand(infoPtr, iBand);
    let mut xPos: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int;
    let setBreak: BOOL = ((*REBAR_GetBand(infoPtr, iFirstBand)).fStyle
        & RBBS_BREAK as UINT) as BOOL;

    /* Find the band's new position */
    if reorder != 0 {
        /* Used during an LR band reorder drag */
        i = iFirstBand as ::core::ffi::c_int;
        while i < iLastBand {
            if xPos > xOff {
                break;
            }
            xPos += (*REBAR_GetBand(infoPtr, i as INT)).cxEffective as ::core::ffi::c_int
                + SEP_WIDTH!(infoPtr);
            i = next_visible(infoPtr, i);
        }
    } else {
        /* Used during a UD band insertion drag */
        i = iFirstBand as ::core::ffi::c_int;
        while i < iLastBand {
            let band: *const REBAR_BAND = REBAR_GetBand(infoPtr, i as INT);
            if (xPos as UINT).wrapping_add((*band).cxMinBand / 2 as UINT)
                > xOff as UINT
            {
                break;
            }
            xPos += (*band).cxEffective as ::core::ffi::c_int + SEP_WIDTH!(infoPtr);
            i = next_visible(infoPtr, i);
        }
    }

    /* Move the band to its new position */
    DPA_DeletePtr((*infoPtr).bands, iBand as ::core::ffi::c_int);
    if i > iBand {
        i -= 1;
    }
    DPA_InsertPtr((*infoPtr).bands, i, insertBand as *mut ::core::ffi::c_void);

    /* Ensure only the last band has the RBBS_BREAK flag set */
    (*insertBand).fStyle &= !(RBBS_BREAK as UINT);
    if setBreak != 0 {
        (*REBAR_GetBand(infoPtr, iFirstBand)).fStyle |= RBBS_BREAK as UINT;
    }

    /* Return the currently grabbed band */
    if (*infoPtr).iGrabbedBand == iBand {
        (*infoPtr).iGrabbedBand = i as INT;
        return i;
    } else {
        return -1;
    }
}

unsafe extern "C" fn REBAR_SetBandsHeight(
    infoPtr: *const REBAR_INFO,
    iBeginBand: INT,
    iEndBand: INT,
    yStart: INT,
) -> c_int {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut();
    let mut yMaxHeight: c_int = 0;
    let mut yPos: c_int = yStart;
    let mut row: c_int;
    let mut i: c_int;

    i = iBeginBand;
    while i < iEndBand {
        lpBand = REBAR_GetBand(infoPtr, i);
        (*lpBand).cyRowSoFar = yMaxHeight as UINT;
        yMaxHeight = max(yMaxHeight as UINT, (*lpBand).cyMinBand) as c_int;
        i = next_visible(infoPtr, i);
    }

    row = if iBeginBand < iEndBand {
        (*REBAR_GetBand(infoPtr, iBeginBand)).iRow as c_int
    } else {
        0
    };

    i = iBeginBand;
    while i < iEndBand {
        lpBand = REBAR_GetBand(infoPtr, i);
        /* we may be called for multiple rows if RBS_VARHEIGHT not set */
        if (*lpBand).iRow != row {
            yPos += yMaxHeight + SEP_WIDTH!(infoPtr);
            row = (*lpBand).iRow as c_int;
        }

        if (*lpBand).rcBand.top != yPos || (*lpBand).rcBand.bottom != yPos + yMaxHeight {
            (*lpBand).fDraw |= NTF_INVALIDATE as UINT;
            (*lpBand).rcBand.top = yPos as LONG;
            (*lpBand).rcBand.bottom = (yPos + yMaxHeight) as LONG;
        }
        i = next_visible(infoPtr, i);
    }
    return yPos + yMaxHeight;
}
unsafe extern "C" fn REBAR_LayoutRow(
    infoPtr: *const REBAR_INFO,
    iBeginBand: ::core::ffi::c_int,
    iEndBand: ::core::ffi::c_int,
    cx: ::core::ffi::c_int,
    piRow: *mut ::core::ffi::c_int,
    pyPos: *mut ::core::ffi::c_int,
) {
    let mut lpBand: *mut REBAR_BAND = ::core::ptr::null_mut::<REBAR_BAND>();
    let mut i: ::core::ffi::c_int = 0;
    let mut extra: ::core::ffi::c_int = 0;
    let mut width: ::core::ffi::c_int = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_LayoutRow\0" as *const u8 as *const ::core::ffi::c_char,
            1359 as ::core::ffi::c_int,
            b"Adjusting row [%d;%d). Width: %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            iBeginBand,
            iEndBand,
            cx,
        );
    }
    i = iBeginBand;
    while i < iEndBand {
        (*REBAR_GetBand(infoPtr, i as INT)).iRow = *piRow as INT;
        i += 1;
    }
    i = iBeginBand;
    while i < iEndBand {
        lpBand = REBAR_GetBand(infoPtr, i as INT);
        if i > iBeginBand {
            width += SEP_WIDTH!(infoPtr);
        }
        // max() of two UINTs compares unsigned in the C
        (*lpBand).cxEffective = max((*lpBand).cxMinBand, (*lpBand).cx) as INT;
        width += (*lpBand).cxEffective as ::core::ffi::c_int;
        i = next_visible(infoPtr, i as INT) as ::core::ffi::c_int;
    }
    extra = cx - width;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_LayoutRow\0" as *const u8 as *const ::core::ffi::c_char,
            1374 as ::core::ffi::c_int,
            b"Extra space: %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            extra,
        );
    }
    if extra < 0 {
        let mut ret: ::core::ffi::c_int = REBAR_ShrinkBandsRTL(
            infoPtr,
            iBeginBand as INT,
            iEndBand as INT,
            -(extra as INT),
            FALSE,
        );
        if ret > 0
            && next_visible(infoPtr, iBeginBand as INT) != iEndBand as INT
        {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/rebar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"REBAR_LayoutRow\0" as *const u8 as *const ::core::ffi::c_char,
                    1378 as ::core::ffi::c_int,
                    b"Error layouting row %d - couldn't shrink for %d pixels (%d total shrink)\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    *piRow,
                    ret,
                    -extra,
                );
            }
        }
    } else if extra > 0 {
        lpBand = REBAR_FindBandToGrow(infoPtr, iBeginBand as INT, iEndBand as INT);
        (*lpBand).cxEffective += extra as INT;
    }
    REBAR_SetRowRectsX(infoPtr, iBeginBand as INT, iEndBand as INT);
    if (*infoPtr).dwStyle & RBS_VARHEIGHT as DWORD != 0 {
        if *piRow > 0 {
            *pyPos += SEP_WIDTH!(infoPtr);
        }
        *pyPos = REBAR_SetBandsHeight(infoPtr, iBeginBand as INT, iEndBand as INT, *pyPos as INT)
            as ::core::ffi::c_int;
    }
    *piRow += 1;
}

unsafe extern "C" fn REBAR_Layout(infoPtr: *mut REBAR_INFO) {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut();
    let mut rcAdj: RECT = core::mem::zeroed();
    let mut oldSize: SIZE = core::mem::zeroed();
    let mut adjcx: INT = 0;
    let mut i: INT = 0;
    let mut rowstart: INT = 0;
    let mut row: INT = 0;
    let mut xMin: INT = 0;
    let mut yPos: INT = 0;

    if ((*infoPtr).dwStyle & (CCS_NORESIZE | CCS_NOPARENTALIGN)) != 0
        || GetParent((*infoPtr).hwndSelf).is_null()
    {
        GetClientRect((*infoPtr).hwndSelf, &raw mut rcAdj);
    } else {
        GetClientRect(GetParent((*infoPtr).hwndSelf), &raw mut rcAdj);
    }

    adjcx = get_rect_cx(infoPtr, &raw mut rcAdj) as INT;

    if (*infoPtr).uNumBands == 0 {
        (*infoPtr).calcSize.cx = adjcx as LONG;
        /* the calcSize.cy won't change for a 0 band rebar */
        (*infoPtr).uNumRows = 0;
        REBAR_ForceResize(infoPtr);
        return;
    }

    yPos = 0;
    xMin = 0;
    rowstart = first_visible(infoPtr) as INT;
    /* divide rows */
    i = rowstart;
    while (i as UINT) < (*infoPtr).uNumBands {
        lpBand = REBAR_GetBand(infoPtr, i);

        if i > rowstart
            && ((((*lpBand).fStyle & RBBS_BREAK) != 0)
                || (xMin as UINT).wrapping_add((*lpBand).cxMinBand) > adjcx as UINT)
        {
            REBAR_LayoutRow(infoPtr, rowstart, i, adjcx, &raw mut row, &raw mut yPos);
            rowstart = i;
            xMin = 0;
        } else {
            xMin += SEP_WIDTH!(infoPtr);
        }

        xMin = (xMin as UINT).wrapping_add((*lpBand).cxMinBand) as INT;

        i = next_visible(infoPtr, i) as INT;
    }
    if (rowstart as UINT) < (*infoPtr).uNumBands {
        REBAR_LayoutRow(
            infoPtr,
            rowstart,
            (*infoPtr).uNumBands as INT,
            adjcx,
            &raw mut row,
            &raw mut yPos,
        );
    }

    if ((*infoPtr).dwStyle & RBS_VARHEIGHT) == 0 {
        yPos = REBAR_SetBandsHeight(
            infoPtr,
            first_visible(infoPtr) as INT,
            (*infoPtr).uNumBands as INT,
            0,
        ) as INT;
    }

    (*infoPtr).uNumRows = row as UINT;

    if ((*infoPtr).dwStyle & CCS_VERT) != 0 {
        REBAR_CalcVertBand(infoPtr, 0, (*infoPtr).uNumBands);
    } else {
        REBAR_CalcHorzBand(infoPtr, 0, (*infoPtr).uNumBands);
    }

    /* now compute size of Rebar itself */
    oldSize = (*infoPtr).calcSize;

    (*infoPtr).calcSize.cx = adjcx as LONG;
    (*infoPtr).calcSize.cy = yPos as LONG;

    REBAR_DumpBand(infoPtr);
    REBAR_MoveChildWindows(infoPtr, 0, (*infoPtr).uNumBands);
    REBAR_ForceResize(infoPtr);

    /* note: after a RBN_HEIGHTCHANGE native sends once again all the RBN_CHILDSIZE
     * and does another ForceResize */
    if oldSize.cy != (*infoPtr).calcSize.cy {
        let mut heightchange: NMHDR = core::mem::zeroed();
        REBAR_Notify(&raw mut heightchange, infoPtr, (RBN_HEIGHTCHANGE) as UINT);
        REBAR_AutoSize(infoPtr, FALSE);
    }
}

unsafe extern "C" fn REBAR_SizeChildrenToHeight(
    infoPtr: *const REBAR_INFO,
    iBeginBand: ::core::ffi::c_int,
    iEndBand: ::core::ffi::c_int,
    extra: ::core::ffi::c_int,
    fChanged: *mut BOOL,
) -> ::core::ffi::c_int {
    let mut cyBandsOld: ::core::ffi::c_int;
    let mut cyBandsNew: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_SizeChildrenToHeight\0" as *const u8 as *const ::core::ffi::c_char,
            1485 as ::core::ffi::c_int,
            b"[%d;%d) by %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            iBeginBand,
            iEndBand,
            extra,
        );
    }
    cyBandsOld = ((*REBAR_GetBand(infoPtr, iBeginBand as INT)).rcBand.bottom
        - (*REBAR_GetBand(infoPtr, iBeginBand as INT)).rcBand.top)
        as ::core::ffi::c_int;
    i = iBeginBand;
    while i < iEndBand {
        let lpBand: *mut REBAR_BAND = REBAR_GetBand(infoPtr, i as INT);
        let cyMaxChild: ::core::ffi::c_int = cyBandsOld
            - (REBARSPACE!(lpBand) as ::core::ffi::c_int)
            + extra;
        let cyChild: ::core::ffi::c_int = round_child_height(lpBand, cyMaxChild);
        if !(*lpBand).hwndChild.is_null()
            && cyChild as UINT != (*lpBand).cyChild
            && (*lpBand).fStyle & RBBS_VARIABLEHEIGHT as UINT != 0
        {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/rebar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"REBAR_SizeChildrenToHeight\0" as *const u8
                        as *const ::core::ffi::c_char,
                    1497 as ::core::ffi::c_int,
                    b"Resizing %d: %d -> %d [%d]\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    i,
                    (*lpBand).cyChild,
                    cyChild,
                    (*lpBand).cyMaxChild,
                );
            }
            *fChanged = TRUE as BOOL;
            (*lpBand).cyChild = cyChild as UINT;
            (*lpBand).fDraw |= NTF_INVALIDATE as UINT;
            update_min_band_height(infoPtr, lpBand);
        }
        cyBandsNew = max(cyBandsNew as UINT, (*lpBand).cyMinBand) as ::core::ffi::c_int;
        i = next_visible(infoPtr, i);
    }
    return cyBandsNew - cyBandsOld;
}

unsafe extern "C" fn REBAR_SizeToHeight(infoPtr: *mut REBAR_INFO, mut height: ::core::ffi::c_int) {
    let mut extra: ::core::ffi::c_int = height - (*infoPtr).calcSize.cy as ::core::ffi::c_int;
    let mut fChanged: BOOL = FALSE;
    let mut uNumRows: UINT = (*infoPtr).uNumRows;
    let mut i: ::core::ffi::c_int = 0;

    if uNumRows == 0 as UINT {
        return;
    }

    /* Pass one: break-up/glue rows */
    if extra > 0 {
        i = prev_visible(infoPtr, (*infoPtr).uNumBands as ::core::ffi::c_int);
        while i > 0 {
            let mut lpBand: *mut REBAR_BAND = REBAR_GetBand(infoPtr, i as INT);
            let mut cyBreakExtra: ::core::ffi::c_int = 0;

            height = ((*lpBand).rcBand.bottom - (*lpBand).rcBand.top) as ::core::ffi::c_int;

            if (*infoPtr).dwStyle & RBS_VARHEIGHT as DWORD != 0 {
                cyBreakExtra = (*lpBand).cyRowSoFar as ::core::ffi::c_int;
            } else {
                cyBreakExtra = height;
            }
            cyBreakExtra += SEP_WIDTH!(infoPtr);

            if extra <= cyBreakExtra / 2 {
                break;
            }

            if ((*lpBand).fStyle & RBBS_BREAK as UINT) == 0 {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"dll/win32/comctl32/rebar.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"REBAR_SizeToHeight\0" as *const u8 as *const ::core::ffi::c_char,
                        1543 as ::core::ffi::c_int,
                        b"Adding break on band %d - extra %d -> %d\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        i,
                        extra,
                        extra - cyBreakExtra,
                    );
                }
                (*lpBand).fStyle |= RBBS_BREAK as UINT;
                (*lpBand).fDraw |= NTF_INVALIDATE as UINT;
                fChanged = TRUE;
                extra -= cyBreakExtra;
                uNumRows = uNumRows.wrapping_add(1);
                if (*infoPtr).dwStyle & RBS_VARHEIGHT as DWORD != 0 {
                    (*lpBand).rcBand.bottom =
                        ((*lpBand).rcBand.top as UINT).wrapping_add((*lpBand).cyMinBand) as LONG;
                }
            }
            i = prev_visible(infoPtr, i);
        }
    }

    /* Pass two: increase/decrease control height */
    if (*infoPtr).dwStyle & RBS_VARHEIGHT as DWORD != 0 {
        let mut i_0: ::core::ffi::c_int = first_visible(infoPtr);
        let mut iRow: ::core::ffi::c_int = 0;
        while (i_0 as UINT) < (*infoPtr).uNumBands {
            let mut lpBand_0: *mut REBAR_BAND = REBAR_GetBand(infoPtr, i_0 as INT);
            let mut extraForRow: ::core::ffi::c_int =
                extra / uNumRows.wrapping_sub(iRow as UINT) as ::core::ffi::c_int;
            let mut rowEnd: ::core::ffi::c_int = 0;

            /* we can't use get_row_end_for_band as we might have added RBBS_BREAK in the first phase */
            rowEnd = next_visible(infoPtr, i_0);
            while (rowEnd as UINT) < (*infoPtr).uNumBands {
                if (*REBAR_GetBand(infoPtr, rowEnd as INT)).iRow != (*lpBand_0).iRow
                    || ((*REBAR_GetBand(infoPtr, rowEnd as INT)).fStyle
                        & RBBS_BREAK as UINT)
                        != 0
                {
                    break;
                }
                rowEnd = next_visible(infoPtr, rowEnd);
            }

            extra -= REBAR_SizeChildrenToHeight(infoPtr, i_0, rowEnd, extraForRow, &raw mut fChanged);

            i_0 = rowEnd;
            iRow += 1;
        }
    } else {
        REBAR_SizeChildrenToHeight(
            infoPtr,
            first_visible(infoPtr),
            (*infoPtr).uNumBands as ::core::ffi::c_int,
            (extra as UINT).wrapping_div((*infoPtr).uNumRows) as ::core::ffi::c_int,
            &raw mut fChanged,
        );
    }

    if fChanged != 0 {
        REBAR_Layout(infoPtr);
    }
}

unsafe extern "C" fn REBAR_AutoSize(infoPtr: *mut REBAR_INFO, needsLayout: BOOL) {
    let mut rc: RECT = core::mem::zeroed();
    let mut rcNew: RECT = core::mem::zeroed();
    let mut autosize: NMRBAUTOSIZE = core::mem::zeroed();

    if needsLayout != 0 {
        REBAR_Layout(infoPtr);
    }
    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rc));
    REBAR_SizeToHeight(infoPtr, get_rect_cy(infoPtr, core::ptr::addr_of_mut!(rc)));
    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcNew));

    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(autosize.rcTarget));
    autosize.fChanged = EqualRect(
        core::ptr::addr_of_mut!(rc) as LPCRECT,
        core::ptr::addr_of_mut!(rcNew) as LPCRECT,
    );
    autosize.rcTarget = rc;
    autosize.rcActual = rcNew;
    REBAR_Notify(
        core::ptr::addr_of_mut!(autosize) as *mut NMHDR,
        infoPtr,
        (RBN_AUTOSIZE) as UINT,
    );
}

unsafe extern "C" fn REBAR_ValidateBand(infoPtr: *const REBAR_INFO, lpBand: *mut REBAR_BAND) {
    let mut header: UINT = 0;
    let mut textheight: UINT = 0;
    let mut imageheight: UINT = 0;
    let mut i: UINT;
    let mut nonfixed: UINT;
    let mut tBand: *mut REBAR_BAND;

    (*lpBand).fStatus = 0;
    (*lpBand).cxMinBand = 0;
    (*lpBand).cyMinBand = 0;

    if (*lpBand).cxMinChild > 65535 {
        (*lpBand).cxMinChild = 0;
    }
    if (*lpBand).cyMinChild > 65535 {
        (*lpBand).cyMinChild = 0;
    }
    if (*lpBand).cx > 65535 {
        (*lpBand).cx = 0;
    }
    if (*lpBand).cyChild > 65535 {
        (*lpBand).cyChild = 0;
    }
    if (*lpBand).cyIntegral > 65535 {
        (*lpBand).cyIntegral = 0;
    }
    if (*lpBand).cxIdeal > 65535 {
        (*lpBand).cxIdeal = 0;
    }
    if (*lpBand).cxHeader > 65535 {
        (*lpBand).cxHeader = 0;
    }

    nonfixed = 0;
    i = 0;
    while i < (*infoPtr).uNumBands {
        tBand = REBAR_GetBand(infoPtr, i as INT);
        if !HIDDENBAND!(infoPtr, tBand) && !((*tBand).fStyle & RBBS_FIXEDSIZE != 0) {
            nonfixed = nonfixed.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    if !((*lpBand).fStyle & RBBS_NOGRIPPER != 0)
        && ((*lpBand).fStyle & RBBS_GRIPPERALWAYS != 0
            || (!((*lpBand).fStyle & RBBS_FIXEDSIZE != 0) && nonfixed > 1))
    {
        (*lpBand).fStatus |= HAS_GRIPPER;
        if (*infoPtr).dwStyle & CCS_VERT != 0 {
            if (*infoPtr).dwStyle & RBS_VERTICALGRIPPER != 0 {
                header = header.wrapping_add((GRIPPER_HEIGHT + REBAR_PRE_GRIPPER) as UINT);
            } else {
                header = header.wrapping_add((GRIPPER_WIDTH + REBAR_PRE_GRIPPER) as UINT);
            }
        } else {
            header = header.wrapping_add((REBAR_PRE_GRIPPER + GRIPPER_WIDTH) as UINT);
        }
        header = header.wrapping_add(REBAR_ALWAYS_SPACE as UINT);
    }

    if (*lpBand).iImage != -1 && !(*infoPtr).himl.is_null() {
        (*lpBand).fStatus |= HAS_IMAGE;
        if (*infoPtr).dwStyle & CCS_VERT != 0 {
            header = header.wrapping_add(((*infoPtr).imageSize.cy as INT + (REBAR_POST_IMAGE as i32)) as UINT);
            imageheight = ((*infoPtr).imageSize.cx as INT + 4) as UINT;
        } else {
            header = header.wrapping_add(((*infoPtr).imageSize.cx as INT + (REBAR_POST_IMAGE as i32)) as UINT);
            imageheight = ((*infoPtr).imageSize.cy as INT + 4) as UINT;
        }
    }

    if ((*lpBand).fMask & RBBIM_TEXT != 0)
        && !(*lpBand).lpText.is_null()
        && !((*lpBand).fStyle & RBBS_HIDETITLE != 0)
    {
        let hdc: HDC = GetDC(core::ptr::null_mut());
        let hOldFont: HFONT = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;
        let mut size: SIZE = SIZE { cx: 0, cy: 0 };

        (*lpBand).fStatus |= HAS_TEXT;
        GetTextExtentPoint32W(
            hdc,
            (*lpBand).lpText as LPCWSTR,
            lstrlenW((*lpBand).lpText as LPCWSTR),
            &raw mut size,
        );
        header = header.wrapping_add(
            (if (*infoPtr).dwStyle & CCS_VERT != 0 {
                size.cy as INT + (REBAR_POST_TEXT as i32)
            } else {
                size.cx as INT + (REBAR_POST_TEXT as i32)
            }) as UINT,
        );
        textheight = (if (*infoPtr).dwStyle & CCS_VERT != 0 {
            0
        } else {
            size.cy as INT
        }) as UINT;

        SelectObject(hdc, hOldFont as HGDIOBJ);
        ReleaseDC(core::ptr::null_mut(), hdc);
    }

    if (*lpBand).fStatus & (HAS_IMAGE | HAS_TEXT) != 0
        && !((*lpBand).fStatus & HAS_GRIPPER != 0)
    {
        header = header.wrapping_add(REBAR_ALWAYS_SPACE as UINT);
    }

    if !((*lpBand).fStyle & RBBS_UNDOC_FIXEDHEADER != 0) {
        (*lpBand).cxHeader = header;
    }
    (*lpBand).cyHeader = max(textheight, imageheight);

    update_min_band_height(infoPtr, lpBand);

    (*lpBand).cxMinBand = (*lpBand)
        .cxMinChild
        .wrapping_add((*lpBand).cxHeader)
        .wrapping_add(REBAR_POST_CHILD as UINT);
    if (*lpBand).fStyle & RBBS_USECHEVRON != 0 && (*lpBand).cxMinChild < (*lpBand).cxIdeal {
        (*lpBand).cxMinBand = (*lpBand).cxMinBand.wrapping_add(CHEVRON_WIDTH as UINT);
    }
}

unsafe extern "C" fn REBAR_CommonSetupBand(hwnd: HWND, lprbbi: *const REBARBANDINFOW, lpBand: *mut REBAR_BAND) -> UINT {
    let mut uChanged: UINT = 0x0;

    (*lpBand).fMask |= (*lprbbi).fMask;

    if ((*lprbbi).fMask & RBBIM_STYLE) != 0 &&
        ((*lpBand).fStyle != (*lprbbi).fStyle)
    {
        (*lpBand).fStyle = (*lprbbi).fStyle;
        uChanged |= RBBIM_STYLE;
    }

    if ((*lprbbi).fMask & RBBIM_COLORS) != 0 &&
        (((*lpBand).clrFore != (*lprbbi).clrFore) ||
         ((*lpBand).clrBack != (*lprbbi).clrBack))
    {
        (*lpBand).clrFore = (*lprbbi).clrFore;
        (*lpBand).clrBack = (*lprbbi).clrBack;
        uChanged |= RBBIM_COLORS;
    }

    if ((*lprbbi).fMask & RBBIM_IMAGE) != 0 &&
        ((*lpBand).iImage != (*lprbbi).iImage)
    {
        (*lpBand).iImage = (*lprbbi).iImage;
        uChanged |= RBBIM_IMAGE;
    }

    if ((*lprbbi).fMask & RBBIM_CHILD) != 0 &&
        ((*lprbbi).hwndChild != (*lpBand).hwndChild)
    {
        if !(*lprbbi).hwndChild.is_null() {
            (*lpBand).hwndChild = (*lprbbi).hwndChild;
            (*lpBand).hwndPrevParent = SetParent((*lpBand).hwndChild, hwnd);
            /* below in trace from WinRAR */
            ShowWindow((*lpBand).hwndChild, ((SW_SHOWNOACTIVATE | SW_SHOWNORMAL)) as i32);
            /* above in trace from WinRAR */
        }
        else {
            (*lpBand).hwndChild = core::ptr::null_mut();
            (*lpBand).hwndPrevParent = core::ptr::null_mut();
        }
        uChanged |= RBBIM_CHILD;
    }

    if ((*lprbbi).fMask & RBBIM_CHILDSIZE) != 0 &&
        (((*lpBand).cxMinChild != (*lprbbi).cxMinChild) ||
         ((*lpBand).cyMinChild != (*lprbbi).cyMinChild) ||
         ((((*lprbbi).cbSize as usize) >= REBARBANDINFOA_V6_SIZE && ((*lpBand).fStyle & RBBS_VARIABLEHEIGHT) != 0) &&
          (((*lpBand).cyChild != (*lprbbi).cyChild) ||
           ((*lpBand).cyMaxChild != (*lprbbi).cyMaxChild) ||
           ((*lpBand).cyIntegral != (*lprbbi).cyIntegral))) ||
         ((((*lprbbi).cbSize as usize) < REBARBANDINFOA_V6_SIZE) &&
          (((*lpBand).cyChild != 0) ||
           ((*lpBand).cyMaxChild != 0) ||
           ((*lpBand).cyIntegral != 0))))
    {
        (*lpBand).cxMinChild = (*lprbbi).cxMinChild;
        (*lpBand).cyMinChild = (*lprbbi).cyMinChild;
        /* These fields where added in WIN32_IE == 0x400 and are set only for RBBS_VARIABLEHEIGHT bands */
        if ((*lprbbi).cbSize as usize) >= REBARBANDINFOA_V6_SIZE && ((*lpBand).fStyle & RBBS_VARIABLEHEIGHT) != 0 {
            (*lpBand).cyMaxChild = (*lprbbi).cyMaxChild;
            (*lpBand).cyIntegral = (*lprbbi).cyIntegral;

            (*lpBand).cyChild = round_child_height(lpBand, (*lprbbi).cyChild as ::core::ffi::c_int) as UINT;  /* make (cyChild - cyMinChild) a multiple of cyIntergral */
        }
        else {
            (*lpBand).cyChild = (*lpBand).cyMinChild;
            (*lpBand).cyMaxChild = 0x7fffffff;
            (*lpBand).cyIntegral = 0;
        }
        uChanged |= RBBIM_CHILDSIZE;
    }

    if ((*lprbbi).fMask & RBBIM_SIZE) != 0 &&
        ((*lpBand).cx != (*lprbbi).cx)
    {
        (*lpBand).cx = (*lprbbi).cx;
        uChanged |= RBBIM_SIZE;
    }

    if ((*lprbbi).fMask & RBBIM_BACKGROUND) != 0 &&
        ((*lpBand).hbmBack != (*lprbbi).hbmBack)
    {
        (*lpBand).hbmBack = (*lprbbi).hbmBack;
        uChanged |= RBBIM_BACKGROUND;
    }

    if ((*lprbbi).fMask & RBBIM_ID) != 0 &&
        ((*lpBand).wID != (*lprbbi).wID)
    {
        (*lpBand).wID = (*lprbbi).wID;
        uChanged |= RBBIM_ID;
    }

    /* check for additional data */
    if ((*lprbbi).cbSize as usize) >= REBARBANDINFOA_V6_SIZE {
        if ((*lprbbi).fMask & RBBIM_IDEALSIZE) != 0 &&
            ((*lpBand).cxIdeal != (*lprbbi).cxIdeal)
        {
            (*lpBand).cxIdeal = (*lprbbi).cxIdeal;
            uChanged |= RBBIM_IDEALSIZE;
        }

        if ((*lprbbi).fMask & RBBIM_LPARAM) != 0 &&
            ((*lpBand).lParam != (*lprbbi).lParam)
        {
            (*lpBand).lParam = (*lprbbi).lParam;
            uChanged |= RBBIM_LPARAM;
        }

        if ((*lprbbi).fMask & RBBIM_HEADERSIZE) != 0 &&
            ((*lpBand).cxHeader != (*lprbbi).cxHeader)
        {
            (*lpBand).cxHeader = (*lprbbi).cxHeader;
            (*lpBand).fStyle |= RBBS_UNDOC_FIXEDHEADER;
            uChanged |= RBBIM_HEADERSIZE;
        }
    }

    return uChanged;
}

unsafe extern "C" fn REBAR_EraseBkGnd(infoPtr: *const REBAR_INFO, hdc: HDC) -> LRESULT {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut();
    let mut i: UINT;
    let mut oldrow: INT;
    let mut cr: RECT = core::mem::zeroed();
    let mut old: COLORREF = CLR_NONE;
    let mut new: COLORREF;
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);

    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(cr));

    if !theme.is_null() {
        if IsThemeBackgroundPartiallyTransparent(theme, 0, 0) != 0 {
            DrawThemeParentBackground((*infoPtr).hwndSelf, hdc, core::ptr::addr_of_mut!(cr));
        }
        DrawThemeBackground(theme, hdc, 0, 0, core::ptr::addr_of_mut!(cr), core::ptr::null_mut());
    }

    oldrow = -1;
    i = 0;
    while i < (*infoPtr).uNumBands {
        let mut rcBand: RECT = core::mem::zeroed();
        'body: {
            lpBand = REBAR_GetBand(infoPtr, i as INT);
            if ((*lpBand).fStyle & RBBS_HIDDEN as UINT) != 0
                || (((*infoPtr).dwStyle & CCS_VERT as DWORD) != 0
                    && ((*lpBand).fStyle & RBBS_NOVERT as UINT) != 0)
            {
                break 'body;
            }
            translate_rect(infoPtr, core::ptr::addr_of_mut!(rcBand), core::ptr::addr_of_mut!((*lpBand).rcBand));

            /* draw band separator between rows */
            if (*lpBand).iRow != oldrow {
                oldrow = (*lpBand).iRow;
                if ((*infoPtr).dwStyle & RBS_BANDBORDERS) != 0 {
                    let mut rcRowSep: RECT = core::mem::zeroed();
                    rcRowSep = rcBand;
                    if ((*infoPtr).dwStyle & CCS_VERT) != 0 {
                        rcRowSep.right += (SEP_WIDTH_SIZE as i32);
                        rcRowSep.bottom = (*infoPtr).calcSize.cx;
                        if !theme.is_null() {
                            DrawThemeEdge(theme, hdc, (RP_BAND) as i32, 0, core::ptr::addr_of_mut!(rcRowSep), EDGE_ETCHED, BF_RIGHT, core::ptr::null_mut());
                        } else {
                            DrawEdge(hdc, core::ptr::addr_of_mut!(rcRowSep), EDGE_ETCHED, BF_RIGHT);
                        }
                    } else {
                        rcRowSep.bottom += (SEP_WIDTH_SIZE as i32);
                        rcRowSep.right = (*infoPtr).calcSize.cx;
                        if !theme.is_null() {
                            DrawThemeEdge(theme, hdc, (RP_BAND) as i32, 0, core::ptr::addr_of_mut!(rcRowSep), EDGE_ETCHED, BF_BOTTOM, core::ptr::null_mut());
                        } else {
                            DrawEdge(hdc, core::ptr::addr_of_mut!(rcRowSep), EDGE_ETCHED, BF_BOTTOM);
                        }
                    }
                }
            }

            /* draw band separator between bands in a row */
            if ((*infoPtr).dwStyle & RBS_BANDBORDERS) != 0 && (*lpBand).rcBand.left > 0 {
                let mut rcSep: RECT = core::mem::zeroed();
                rcSep = rcBand;
                if ((*infoPtr).dwStyle & CCS_VERT) != 0 {
                    rcSep.bottom = rcSep.top;
                    rcSep.top -= (SEP_WIDTH_SIZE as i32);
                    if !theme.is_null() {
                        DrawThemeEdge(theme, hdc, (RP_BAND) as i32, 0, core::ptr::addr_of_mut!(rcSep), EDGE_ETCHED, BF_BOTTOM, core::ptr::null_mut());
                    } else {
                        DrawEdge(hdc, core::ptr::addr_of_mut!(rcSep), EDGE_ETCHED, BF_BOTTOM);
                    }
                } else {
                    rcSep.right = rcSep.left;
                    rcSep.left -= (SEP_WIDTH_SIZE as i32);
                    if !theme.is_null() {
                        DrawThemeEdge(theme, hdc, (RP_BAND) as i32, 0, core::ptr::addr_of_mut!(rcSep), EDGE_ETCHED, BF_RIGHT, core::ptr::null_mut());
                    } else {
                        DrawEdge(hdc, core::ptr::addr_of_mut!(rcSep), EDGE_ETCHED, BF_RIGHT);
                    }
                }
            }

            /* draw the actual background */
            if (*lpBand).clrBack != CLR_NONE {
                new = if (*lpBand).clrBack == CLR_DEFAULT {
                    (*infoPtr).clrBtnFace
                } else {
                    (*lpBand).clrBack
                };
                if false {
                    /* testing only - make background green to see it */
                    new = RGB!(0, 128, 0);
                }
            } else {
                /* In the absence of documentation for Rebar vs. CLR_NONE,
                 * we will use the default BtnFace color. */
                new = (*infoPtr).clrBtnFace;
                if false {
                    /* testing only - make background green to see it */
                    new = RGB!(0, 128, 0);
                }
            }

            if !theme.is_null() {
                /* When themed, the background color is ignored (but not a
                 * background bitmap */
                DrawThemeBackground(theme, hdc, 0, 0, core::ptr::addr_of_mut!(cr), core::ptr::addr_of_mut!(rcBand));
            } else {
                old = SetBkColor(hdc, new);
                ExtTextOutW(hdc, 0, 0, ETO_OPAQUE, core::ptr::addr_of_mut!(rcBand), core::ptr::null_mut(), 0, core::ptr::null());
                if (*lpBand).clrBack != CLR_NONE {
                    SetBkColor(hdc, old);
                }
            }
        }
        i = i.wrapping_add(1);
    }
    return (TRUE) as LRESULT;
}

unsafe extern "C" fn REBAR_InternalHitTest(
    infoPtr: *const REBAR_INFO,
    lpPt: *const POINT,
    pFlags: *mut UINT,
    pBand: *mut INT,
) {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut();
    let mut rect: RECT = core::mem::zeroed();
    let mut iCount: UINT = 0;

    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rect));

    *pFlags = RBHT_NOWHERE as UINT;
    if PtInRect(core::ptr::addr_of_mut!(rect) as *const RECT, *lpPt) != 0 {
        if (*infoPtr).uNumBands == 0 as UINT {
            *pFlags = RBHT_NOWHERE as UINT;
            if !pBand.is_null() {
                *pBand = -(1 as INT);
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
                    b"dll/win32/comctl32/rebar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"REBAR_InternalHitTest\0" as *const u8 as *const ::core::ffi::c_char,
                    1993 as ::core::ffi::c_int,
                    b"NOWHERE\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return;
        } else {
            iCount = 0 as UINT;
            while iCount < (*infoPtr).uNumBands {
                let mut rcBand: RECT = core::mem::zeroed();
                lpBand = REBAR_GetBand(infoPtr, iCount as INT);
                translate_rect(
                    infoPtr,
                    core::ptr::addr_of_mut!(rcBand),
                    core::ptr::addr_of_mut!((*lpBand).rcBand),
                );
                if !HIDDENBAND!(infoPtr, lpBand) {
                    if PtInRect(core::ptr::addr_of_mut!(rcBand) as *const RECT, *lpPt) != 0 {
                        if !pBand.is_null() {
                            *pBand = iCount as INT;
                        }
                        if PtInRect(
                            core::ptr::addr_of_mut!((*lpBand).rcGripper) as *const RECT,
                            *lpPt,
                        ) != 0
                        {
                            *pFlags = RBHT_GRABBER as UINT;
                            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                                & ((1 as ::core::ffi::c_int)
                                    << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
                                != 0
                            {
                                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                                ros_dbg_log(
                                    __dbcl_0,
                                    __dbch_0,
                                    b"dll/win32/comctl32/rebar.c\0"
                                        as *const u8 as *const ::core::ffi::c_char,
                                    b"REBAR_InternalHitTest\0" as *const u8 as *const ::core::ffi::c_char,
                                    2008 as ::core::ffi::c_int,
                                    b"ON GRABBER %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                                    iCount,
                                );
                            }
                            return;
                        } else if PtInRect(
                            core::ptr::addr_of_mut!((*lpBand).rcCapImage) as *const RECT,
                            *lpPt,
                        ) != 0
                        {
                            *pFlags = RBHT_CAPTION as UINT;
                            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                                & ((1 as ::core::ffi::c_int)
                                    << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
                                != 0
                            {
                                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                                ros_dbg_log(
                                    __dbcl_1,
                                    __dbch_1,
                                    b"dll/win32/comctl32/rebar.c\0"
                                        as *const u8 as *const ::core::ffi::c_char,
                                    b"REBAR_InternalHitTest\0" as *const u8 as *const ::core::ffi::c_char,
                                    2013 as ::core::ffi::c_int,
                                    b"ON CAPTION %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                                    iCount,
                                );
                            }
                            return;
                        } else if PtInRect(
                            core::ptr::addr_of_mut!((*lpBand).rcCapText) as *const RECT,
                            *lpPt,
                        ) != 0
                        {
                            *pFlags = RBHT_CAPTION as UINT;
                            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                                & ((1 as ::core::ffi::c_int)
                                    << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
                                != 0
                            {
                                let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                                let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
                                ros_dbg_log(
                                    __dbcl_2,
                                    __dbch_2,
                                    b"dll/win32/comctl32/rebar.c\0"
                                        as *const u8 as *const ::core::ffi::c_char,
                                    b"REBAR_InternalHitTest\0" as *const u8 as *const ::core::ffi::c_char,
                                    2018 as ::core::ffi::c_int,
                                    b"ON CAPTION %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                                    iCount,
                                );
                            }
                            return;
                        } else if PtInRect(
                            core::ptr::addr_of_mut!((*lpBand).rcChild) as *const RECT,
                            *lpPt,
                        ) != 0
                        {
                            *pFlags = RBHT_CLIENT as UINT;
                            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                                & ((1 as ::core::ffi::c_int)
                                    << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
                                != 0
                            {
                                let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                                let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
                                ros_dbg_log(
                                    __dbcl_3,
                                    __dbch_3,
                                    b"dll/win32/comctl32/rebar.c\0"
                                        as *const u8 as *const ::core::ffi::c_char,
                                    b"REBAR_InternalHitTest\0" as *const u8 as *const ::core::ffi::c_char,
                                    2023 as ::core::ffi::c_int,
                                    b"ON CLIENT %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                                    iCount,
                                );
                            }
                            return;
                        } else if PtInRect(
                            core::ptr::addr_of_mut!((*lpBand).rcChevron) as *const RECT,
                            *lpPt,
                        ) != 0
                        {
                            *pFlags = RBHT_CHEVRON as UINT;
                            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                                & ((1 as ::core::ffi::c_int)
                                    << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
                                != 0
                            {
                                let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                                let __dbcl_4: __wine_debug_class = __WINE_DBCL_TRACE;
                                ros_dbg_log(
                                    __dbcl_4,
                                    __dbch_4,
                                    b"dll/win32/comctl32/rebar.c\0"
                                        as *const u8 as *const ::core::ffi::c_char,
                                    b"REBAR_InternalHitTest\0" as *const u8 as *const ::core::ffi::c_char,
                                    2028 as ::core::ffi::c_int,
                                    b"ON CHEVRON %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                                    iCount,
                                );
                            }
                            return;
                        } else {
                            *pFlags = RBHT_NOWHERE as UINT;
                            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                                & ((1 as ::core::ffi::c_int)
                                    << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
                                != 0
                            {
                                let __dbch_5: *mut __wine_debug_channel = __wine_dbch___default;
                                let __dbcl_5: __wine_debug_class = __WINE_DBCL_TRACE;
                                ros_dbg_log(
                                    __dbcl_5,
                                    __dbch_5,
                                    b"dll/win32/comctl32/rebar.c\0"
                                        as *const u8 as *const ::core::ffi::c_char,
                                    b"REBAR_InternalHitTest\0" as *const u8 as *const ::core::ffi::c_char,
                                    2033 as ::core::ffi::c_int,
                                    b"NOWHERE %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                                    iCount,
                                );
                            }
                            return;
                        }
                    }
                }
                iCount = iCount.wrapping_add(1);
            }

            *pFlags = RBHT_NOWHERE as UINT;
            if !pBand.is_null() {
                *pBand = -(1 as INT);
            }
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
                != 0
            {
                let __dbch_6: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_6: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_6,
                    __dbch_6,
                    b"dll/win32/comctl32/rebar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"REBAR_InternalHitTest\0" as *const u8 as *const ::core::ffi::c_char,
                    2043 as ::core::ffi::c_int,
                    b"NOWHERE\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return;
        }
    } else {
        *pFlags = RBHT_NOWHERE as UINT;
        if !pBand.is_null() {
            *pBand = -(1 as INT);
        }
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
            != 0
        {
            let __dbch_7: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_7: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_7,
                __dbch_7,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_InternalHitTest\0" as *const u8 as *const ::core::ffi::c_char,
                2051 as ::core::ffi::c_int,
                b"NOWHERE\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return;
    }
}

unsafe extern "C" fn REBAR_HandleLRDrag(mut infoPtr: *mut REBAR_INFO, mut ptsmove: *const POINT) {
    let mut hitBand: *mut REBAR_BAND = core::ptr::null_mut::<REBAR_BAND>();
    let mut iHitBand: INT = 0;
    let mut iRowBegin: INT = 0;
    let mut iRowEnd: INT = 0;
    let mut movement: INT = 0;
    let mut xBand: INT = 0;
    let mut cxLeft: INT = 0;
    let mut shrunkBands: BOOL = FALSE;

    iHitBand = (*infoPtr).iGrabbedBand;
    iRowBegin = get_row_begin_for_band(infoPtr, iHitBand);
    iRowEnd = get_row_end_for_band(infoPtr, iHitBand);
    hitBand = REBAR_GetBand(infoPtr, iHitBand);

    xBand = (*hitBand).rcBand.left as INT;
    movement = (if ((*infoPtr).dwStyle & CCS_VERT) != 0 {
        (*ptsmove).y as INT
    } else {
        (*ptsmove).x as INT
    }) - (xBand + (REBAR_PRE_GRIPPER as i32) - (*infoPtr).ihitoffset);

    /* Dragging the first band in a row cannot cause shrinking */
    if iHitBand != iRowBegin {
        if movement < 0 {
            cxLeft = REBAR_ShrinkBandsRTL(infoPtr, iRowBegin, iHitBand, -movement, TRUE);

            if cxLeft < -movement {
                (*hitBand).cxEffective += -movement - cxLeft;
                (*hitBand).cx = (*hitBand).cxEffective as UINT;
                shrunkBands = TRUE;
            }
        } else if movement > 0 {
            cxLeft = movement;
            if prev_visible(infoPtr, iHitBand) >= 0 {
                cxLeft = REBAR_ShrinkBandsLTR(infoPtr, iHitBand, iRowEnd, movement, TRUE);
            }

            if cxLeft < movement {
                let mut lpPrev: *mut REBAR_BAND = REBAR_GetBand(
                    infoPtr,
                    prev_visible(infoPtr, iHitBand),
                );
                (*lpPrev).cxEffective += movement - cxLeft;
                (*lpPrev).cx = (*hitBand).cxEffective as UINT;
                shrunkBands = TRUE;
            }
        }
    }

    if shrunkBands == 0 {
        /* It was not possible to move the band by shrinking bands.
         * Try relocating the band instead. */
        REBAR_MoveBandToRowOffset(
            infoPtr,
            iHitBand,
            iRowBegin,
            iRowEnd,
            xBand + movement,
            TRUE,
        );
    }

    REBAR_SetRowRectsX(infoPtr, iRowBegin, iRowEnd);
    if ((*infoPtr).dwStyle & CCS_VERT) != 0 {
        REBAR_CalcVertBand(infoPtr, 0, (*infoPtr).uNumBands);
    } else {
        REBAR_CalcHorzBand(infoPtr, 0, (*infoPtr).uNumBands);
    }
    REBAR_MoveChildWindows(infoPtr, (iRowBegin) as UINT, (iRowEnd) as UINT);
}

unsafe extern "C" fn REBAR_HandleUDDrag(mut infoPtr: *mut REBAR_INFO, mut ptsmove: *const POINT) {
    let mut yOff: INT = if (*infoPtr).dwStyle & CCS_VERT as DWORD != 0 {
        (*ptsmove).x as INT
    } else {
        (*ptsmove).y as INT
    };
    let mut iHitBand: INT = 0;
    let mut iRowBegin: INT = 0;
    let mut iNextRowBegin: INT = 0;
    let mut hitBand: *mut REBAR_BAND = ::core::ptr::null_mut::<REBAR_BAND>();
    let mut rowBeginBand: *mut REBAR_BAND = ::core::ptr::null_mut::<REBAR_BAND>();
    if (*infoPtr).uNumBands <= 0 as UINT {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_HandleUDDrag\0" as *const u8 as *const ::core::ffi::c_char,
                2131 as ::core::ffi::c_int,
                b"There are no bands in this rebar\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
    }
    if (*infoPtr).uNumBands <= 1 as UINT {
        return;
    }
    iHitBand = (*infoPtr).iGrabbedBand;
    hitBand = REBAR_GetBand(infoPtr, iHitBand);
    if (*hitBand).fStyle & RBBS_BREAK as UINT != 0
        && (iHitBand as UINT) < (*infoPtr).uNumBands.wrapping_sub(1 as UINT)
    {
        (*REBAR_GetBand(infoPtr, iHitBand + 1 as INT)).fStyle |=
            RBBS_BREAK as UINT;
    }
    if yOff < 0 as INT {
        if iHitBand == 0 as INT
            && ((*infoPtr).uNumBands == 1 as UINT
                || (*REBAR_GetBand(infoPtr, 1 as INT)).fStyle
                    & RBBS_BREAK as UINT
                    != 0)
        {
            return;
        }
        DPA_DeletePtr((*infoPtr).bands, iHitBand as ::core::ffi::c_int);
        (*hitBand).fStyle &= !(RBBS_BREAK as UINT);
        (*REBAR_GetBand(infoPtr, 0 as INT)).fStyle |=
            RBBS_BREAK as UINT;
        (*infoPtr).iGrabbedBand = DPA_InsertPtr(
            (*infoPtr).bands,
            0 as ::core::ffi::c_int,
            hitBand as *mut ::core::ffi::c_void,
        ) as INT;
    } else if yOff
        > (*REBAR_GetBand(
            infoPtr,
            (*infoPtr).uNumBands.wrapping_sub(1 as UINT) as INT,
        ))
        .rcBand
        .bottom
    {
        if iHitBand as UINT
            == (*infoPtr).uNumBands.wrapping_sub(1 as UINT)
            && (*hitBand).fStyle & RBBS_BREAK as UINT != 0
        {
            return;
        }
        DPA_DeletePtr((*infoPtr).bands, iHitBand as ::core::ffi::c_int);
        (*hitBand).fStyle |= RBBS_BREAK as UINT;
        (*infoPtr).iGrabbedBand = DPA_InsertPtr(
            (*infoPtr).bands,
            (*infoPtr).uNumBands.wrapping_sub(1 as UINT) as ::core::ffi::c_int,
            hitBand as *mut ::core::ffi::c_void,
        ) as INT;
    } else {
        iRowBegin = first_visible(infoPtr) as INT;
        while (iRowBegin as UINT) < (*infoPtr).uNumBands {
            iNextRowBegin = get_row_end_for_band(infoPtr, iRowBegin) as INT;
            rowBeginBand = REBAR_GetBand(infoPtr, iRowBegin);
            if (*rowBeginBand).rcBand.bottom > yOff {
                REBAR_MoveBandToRowOffset(
                    infoPtr,
                    iHitBand,
                    iRowBegin,
                    iNextRowBegin,
                    (if (*infoPtr).dwStyle & CCS_VERT as DWORD != 0 {
                        (*ptsmove).y as INT
                    } else {
                        (*ptsmove).x as INT
                    }) - REBAR_PRE_GRIPPER as INT
                        - (*infoPtr).ihitoffset,
                    FALSE as BOOL,
                );
                break;
            } else {
                iRowBegin = iNextRowBegin;
            }
        }
    }
    REBAR_Layout(infoPtr);
}

unsafe extern "C" fn REBAR_DeleteBand(infoPtr: *mut REBAR_INFO, wParam: WPARAM) -> LRESULT {
    let uBand: UINT = wParam as UINT;
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut::<REBAR_BAND>();

    if uBand >= (*infoPtr).uNumBands {
        return FALSE as LRESULT;
    }

    lpBand = REBAR_GetBand(infoPtr, uBand as INT);
    REBAR_Notify_NMREBAR(infoPtr, uBand, RBN_DELETINGBAND as UINT);
    /* TODO: a return of 1 should probably cancel the deletion */

    if !(*lpBand).hwndChild.is_null() {
        ShowWindow((*lpBand).hwndChild, SW_HIDE as ::core::ffi::c_int);
    }
    Free((*lpBand).lpText as LPVOID);
    Free(lpBand as LPVOID);

    (*infoPtr).uNumBands = (*infoPtr).uNumBands.wrapping_sub(1);
    DPA_DeletePtr((*infoPtr).bands, uBand as ::core::ffi::c_int);

    REBAR_Notify_NMREBAR(
        infoPtr,
        (-(1 as ::core::ffi::c_int)) as UINT,
        RBN_DELETEDBAND as UINT,
    );

    /* if only 1 band left the re-validate to possible eliminate gripper */
    if (*infoPtr).uNumBands == 1 as UINT {
        REBAR_ValidateBand(infoPtr, REBAR_GetBand(infoPtr, 0 as INT));
    }

    REBAR_Layout(infoPtr);

    return TRUE as LRESULT;
}

unsafe extern "C" fn REBAR_GetBandBorders(infoPtr: *const REBAR_INFO, uBand: UINT, lpRect: *mut RECT) -> LRESULT {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut::<REBAR_BAND>();
    if lpRect.is_null() {
        return 0 as LRESULT;
    }
    if uBand >= (*infoPtr).uNumBands {
        return 0 as LRESULT;
    }
    lpBand = REBAR_GetBand(infoPtr, uBand as INT);

    /* FIXME - the following values were determined by experimentation */
    /* with the REBAR Control Spy. I have guesses as to what the 4 and */
    /* 1 are, but I am not sure. There doesn't seem to be any actual   */
    /* difference in size of the control area with and without the     */
    /* style.  -  GA                                                   */
    if (*infoPtr).dwStyle & RBS_BANDBORDERS as DWORD != 0 {
        if (*infoPtr).dwStyle & CCS_VERT as DWORD != 0 {
            (*lpRect).left = 1 as LONG;
            (*lpRect).top = (*lpBand).cxHeader.wrapping_add(4 as UINT) as LONG;
            (*lpRect).right = 1 as LONG;
            (*lpRect).bottom = 0 as LONG;
        } else {
            (*lpRect).left = (*lpBand).cxHeader.wrapping_add(4 as UINT) as LONG;
            (*lpRect).top = 1 as LONG;
            (*lpRect).right = 0 as LONG;
            (*lpRect).bottom = 1 as LONG;
        }
    } else {
        (*lpRect).left = (*lpBand).cxHeader as LONG;
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn REBAR_GetBandCount(infoPtr: *const REBAR_INFO) -> LRESULT {
    // TRACE("band count %u!\n", infoPtr->uNumBands);
    ((*infoPtr).uNumBands) as _
}

unsafe extern "C" fn REBAR_GetBandInfoT(
    infoPtr: *const REBAR_INFO,
    uIndex: UINT,
    lprbbi: LPREBARBANDINFOW,
    bUnicode: BOOL,
) -> LRESULT {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut::<REBAR_BAND>();
    if lprbbi.is_null() || ((*lprbbi).cbSize as usize) < REBARBANDINFOA_V3_SIZE as usize {
        return FALSE as LRESULT;
    }
    if uIndex >= (*infoPtr).uNumBands {
        return FALSE as LRESULT;
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
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"REBAR_GetBandInfoT\0" as *const u8 as *const core::ffi::c_char,
            2293 as core::ffi::c_int,
            b"index %u (bUnicode=%d)\n\0" as *const u8 as *const core::ffi::c_char,
            uIndex,
            bUnicode,
        );
    }
    lpBand = REBAR_GetBand(infoPtr, uIndex as INT);
    if (*lprbbi).fMask & RBBIM_STYLE as UINT != 0 {
        (*lprbbi).fStyle = (*lpBand).fStyle;
    }
    if (*lprbbi).fMask & RBBIM_COLORS as UINT != 0 {
        (*lprbbi).clrFore = (*lpBand).clrFore;
        (*lprbbi).clrBack = (*lpBand).clrBack;
        if (*lprbbi).clrBack == CLR_DEFAULT as COLORREF {
            (*lprbbi).clrBack = (*infoPtr).clrBtnFace;
        }
    }
    if (*lprbbi).fMask & RBBIM_TEXT as UINT != 0 {
        if bUnicode != 0 {
            Str_GetPtrW(
                (*lpBand).lpText as LPCWSTR,
                (*lprbbi).lpText,
                (*lprbbi).cch as INT,
            );
        } else {
            Str_GetPtrWtoA(
                (*lpBand).lpText as LPCWSTR,
                (*lprbbi).lpText as LPSTR,
                (*lprbbi).cch as INT,
            );
        }
    }
    if (*lprbbi).fMask & RBBIM_IMAGE as UINT != 0 {
        (*lprbbi).iImage = (*lpBand).iImage as core::ffi::c_int;
    }
    if (*lprbbi).fMask & RBBIM_CHILD as UINT != 0 {
        (*lprbbi).hwndChild = (*lpBand).hwndChild;
    }
    if (*lprbbi).fMask & RBBIM_CHILDSIZE as UINT != 0 {
        (*lprbbi).cxMinChild = (*lpBand).cxMinChild;
        (*lprbbi).cyMinChild = (*lpBand).cyMinChild;
        if (*lprbbi).cbSize as usize >= REBARBANDINFOW_V6_SIZE as usize
            && (*lpBand).fStyle & RBBS_VARIABLEHEIGHT as UINT != 0
        {
            (*lprbbi).cyChild = (*lpBand).cyChild;
            (*lprbbi).cyMaxChild = (*lpBand).cyMaxChild;
            (*lprbbi).cyIntegral = (*lpBand).cyIntegral;
        }
    }
    if (*lprbbi).fMask & RBBIM_SIZE as UINT != 0 {
        (*lprbbi).cx = (*lpBand).cx;
    }
    if (*lprbbi).fMask & RBBIM_BACKGROUND as UINT != 0 {
        (*lprbbi).hbmBack = (*lpBand).hbmBack;
    }
    if (*lprbbi).fMask & RBBIM_ID as UINT != 0 {
        (*lprbbi).wID = (*lpBand).wID;
    }
    if (*lprbbi).cbSize as usize >= REBARBANDINFOW_V6_SIZE as usize {
        if (*lprbbi).fMask & RBBIM_IDEALSIZE as UINT != 0 {
            (*lprbbi).cxIdeal = (*lpBand).cxIdeal;
        }
        if (*lprbbi).fMask & RBBIM_LPARAM as UINT != 0 {
            (*lprbbi).lParam = (*lpBand).lParam;
        }
        if (*lprbbi).fMask & RBBIM_HEADERSIZE as UINT != 0 {
            (*lprbbi).cxHeader = (*lpBand).cxHeader;
        }
    }
    REBAR_DumpBandInfo(lprbbi as *const REBARBANDINFOW);
    return TRUE as LRESULT;
}

unsafe extern "C" fn REBAR_GetBarHeight(infoPtr: *const REBAR_INFO) -> LRESULT {
    let mut nHeight: INT = 0;
    nHeight = (*infoPtr).calcSize.cy as INT;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_GetBarHeight\0" as *const u8 as *const ::core::ffi::c_char,
            2367 as ::core::ffi::c_int,
            b"height = %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            nHeight,
        );
    }
    return nHeight as LRESULT;
}

unsafe extern "C" fn REBAR_GetBarInfo(
    mut infoPtr: *const REBAR_INFO,
    mut lpInfo: LPREBARINFO,
) -> LRESULT {
    if lpInfo.is_null()
        || ((*lpInfo).cbSize as usize) < core::mem::size_of::<REBARINFO>() as usize
    {
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
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_GetBarInfo\0" as *const u8 as *const ::core::ffi::c_char,
            2379 as ::core::ffi::c_int,
            b"getting bar info!\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if !(*infoPtr).himl.is_null() {
        (*lpInfo).himl = ((*infoPtr).himl).cast();
        (*lpInfo).fMask |= RBIM_IMAGELIST as UINT;
    }
    return TRUE as LRESULT;
}

unsafe extern "C" fn REBAR_GetBkColor(infoPtr: *const REBAR_INFO) -> LRESULT {
    let mut clr: COLORREF = (*infoPtr).clrBk;

    if clr == CLR_DEFAULT {
        clr = (*infoPtr).clrBtnFace;
    }

    return clr as LRESULT;
}

unsafe extern "C" fn REBAR_GetPalette(infoPtr: *const REBAR_INFO) -> LRESULT {
    // FIXME("empty stub!\n");
    0
}

unsafe extern "C" fn REBAR_GetRect(infoPtr: *const REBAR_INFO, iBand: INT, lprc: *mut RECT) -> LRESULT {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut::<REBAR_BAND>();
    if iBand < 0 || iBand as UINT >= (*infoPtr).uNumBands {
        return FALSE as LRESULT;
    }
    if lprc.is_null() {
        return FALSE as LRESULT;
    }

    lpBand = REBAR_GetBand(infoPtr, iBand);
    /* For CCS_VERT the coordinates will be swapped - like on Windows */
    *lprc = (*lpBand).rcBand;

    return TRUE as LRESULT;
}

unsafe extern "C" fn REBAR_GetRowCount(infoPtr: *const REBAR_INFO) -> LRESULT {
    // TRACE("%u\n", infoPtr->uNumRows);
    ((*infoPtr).uNumRows) as _
}

unsafe extern "C" fn REBAR_GetRowHeight(infoPtr: *const REBAR_INFO, iRow: INT) -> LRESULT {
    let mut j: ::core::ffi::c_int = 0;
    let mut ret: ::core::ffi::c_int = 0;
    let mut i: UINT;
    let mut lpBand: *mut REBAR_BAND = ::core::ptr::null_mut::<REBAR_BAND>();
    i = 0;
    while i < (*infoPtr).uNumBands {
        lpBand = REBAR_GetBand(infoPtr, i as INT);
        if !(((*lpBand).fStyle & RBBS_HIDDEN as UINT) != 0
            || ((*infoPtr).dwStyle & CCS_VERT as UINT) != 0 && ((*lpBand).fStyle & RBBS_NOVERT as UINT) != 0)
        {
            if !((*lpBand).iRow != iRow) {
                j = ((*lpBand).rcBand.bottom - (*lpBand).rcBand.top) as ::core::ffi::c_int;
                if j > ret {
                    ret = j;
                }
            }
        }
        i = i.wrapping_add(1);
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
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_GetRowHeight\0" as *const u8 as *const ::core::ffi::c_char,
            2461 as ::core::ffi::c_int,
            b"row %d, height %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            iRow,
            ret,
        );
    }
    return ret as LRESULT;
}

unsafe extern "C" fn REBAR_GetTextColor(infoPtr: *const REBAR_INFO) -> LRESULT {
    // TRACE("text color 0x%06x!\n", infoPtr->clrText);
    ((*infoPtr).clrText) as _
}

unsafe extern "C" fn REBAR_GetToolTips(infoPtr: *const REBAR_INFO) -> LRESULT {
    return (*infoPtr).hwndToolTip as LRESULT;
}

unsafe extern "C" fn REBAR_GetUnicodeFormat(infoPtr: *const REBAR_INFO) -> LRESULT {
    // TRACE("%s hwnd=%p\n", infoPtr->bUnicode ? "TRUE" : "FALSE", infoPtr->hwndSelf);
    ((*infoPtr).bUnicode) as _
}

unsafe extern "C" fn REBAR_GetVersion(infoPtr: *const REBAR_INFO) -> LRESULT {
    // TRACE("version %d\n", infoPtr->iVersion);
    ((*infoPtr).iVersion) as _
}

unsafe extern "C" fn REBAR_HitTest(infoPtr: *const REBAR_INFO, lprbht: LPRBHITTESTINFO) -> LRESULT {
    if lprbht.is_null() {
        return -1 as LRESULT;
    }

    REBAR_InternalHitTest(
        infoPtr,
        core::ptr::addr_of_mut!((*lprbht).pt),
        core::ptr::addr_of_mut!((*lprbht).flags),
        core::ptr::addr_of_mut!((*lprbht).iBand),
    );

    return (*lprbht).iBand as LRESULT;
}

unsafe extern "C" fn REBAR_IdToIndex(infoPtr: *const REBAR_INFO, uId: UINT) -> LRESULT {
    let mut i: UINT;

    if (*infoPtr).uNumBands < 1 {
        return -(1 as LRESULT);
    }

    i = 0;
    while i < (*infoPtr).uNumBands {
        if (*REBAR_GetBand(infoPtr, i as INT)).wID == uId {
            return i as LRESULT;
        }
        i = i.wrapping_add(1);
    }

    return -(1 as LRESULT);
}

unsafe extern "C" fn REBAR_InsertBandT(
    mut infoPtr: *mut REBAR_INFO,
    mut iIndex: INT,
    mut lprbbi: *const REBARBANDINFOW,
    mut bUnicode: BOOL,
) -> LRESULT {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut();
    if lprbbi.is_null() || ((*lprbbi).cbSize as usize) < (REBARBANDINFOA_V3_SIZE as usize) {
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
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_InsertBandT\0" as *const u8 as *const ::core::ffi::c_char,
            2542 as ::core::ffi::c_int,
            b"insert band at %d (bUnicode=%d)!\n\0" as *const u8 as *const ::core::ffi::c_char,
            iIndex,
            bUnicode,
        );
    }
    REBAR_DumpBandInfo(lprbbi);
    lpBand = Alloc(::core::mem::size_of::<REBAR_BAND>() as DWORD)
        as *mut REBAR_BAND;
    if lpBand.is_null() {
        return FALSE as LRESULT;
    }
    if iIndex == -(1 as INT) || (iIndex as UINT) > (*infoPtr).uNumBands {
        iIndex = (*infoPtr).uNumBands as INT;
    }
    if DPA_InsertPtr(
        (*infoPtr).bands,
        iIndex as ::core::ffi::c_int,
        lpBand as *mut ::core::ffi::c_void,
    ) == -(1 as ::core::ffi::c_int)
    {
        Free(lpBand as LPVOID);
        return FALSE as LRESULT;
    }
    (*infoPtr).uNumBands = (*infoPtr).uNumBands.wrapping_add(1);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_InsertBandT\0" as *const u8 as *const ::core::ffi::c_char,
            2555 as ::core::ffi::c_int,
            b"index %d!\n\0" as *const u8 as *const ::core::ffi::c_char,
            iIndex,
        );
    }
    memset(
        lpBand as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<REBAR_BAND>() as size_t,
    );
    (*lpBand).clrFore = (if (*infoPtr).clrText == (CLR_NONE as COLORREF) {
        CLR_DEFAULT as COLORREF
    } else {
        (*infoPtr).clrText
    }) as COLORREF;
    (*lpBand).clrBack = (if (*infoPtr).clrBk == (CLR_NONE as COLORREF) {
        CLR_DEFAULT as COLORREF
    } else {
        (*infoPtr).clrBk
    }) as COLORREF;
    (*lpBand).iImage = -(1 as INT);
    REBAR_CommonSetupBand((*infoPtr).hwndSelf, lprbbi, lpBand);
    if ((*lprbbi).cbSize as usize) < (REBARBANDINFOA_V6_SIZE as usize)
        || ((*lpBand).fStyle & (RBBS_VARIABLEHEIGHT as UINT)) == 0
    {
        (*lpBand).cyChild = (*lpBand).cyMinChild;
        (*lpBand).cyMaxChild = 0x7fffffff as UINT;
        (*lpBand).cyIntegral = 0 as UINT;
    }
    if ((*lprbbi).fMask & (RBBIM_TEXT as UINT)) != 0 && !(*lprbbi).lpText.is_null() {
        if bUnicode != 0 {
            Str_SetPtrW(
                core::ptr::addr_of_mut!((*lpBand).lpText),
                (*lprbbi).lpText as LPCWSTR,
            );
        } else {
            Str_SetPtrAtoW(
                core::ptr::addr_of_mut!((*lpBand).lpText),
                (*lprbbi).lpText as LPSTR as LPCSTR,
            );
        }
    }
    REBAR_ValidateBand(infoPtr, lpBand);
    if (*infoPtr).uNumBands == (2 as UINT) {
        REBAR_ValidateBand(
            infoPtr,
            REBAR_GetBand(infoPtr, 0 as INT),
        );
    }
    REBAR_DumpBand(infoPtr);
    REBAR_Layout(infoPtr);
    InvalidateRect(
        (*infoPtr).hwndSelf,
        core::ptr::null::<RECT>(),
        TRUE as BOOL,
    );
    return TRUE as LRESULT;
}

unsafe extern "C" fn REBAR_MaximizeBand(
    infoPtr: *const REBAR_INFO,
    iBand: INT,
    lParam: LPARAM,
) -> LRESULT {
    let mut lpBand: *mut REBAR_BAND = ::core::ptr::null_mut::<REBAR_BAND>();
    let mut iRowBegin: ::core::ffi::c_int = 0;
    let mut iRowEnd: ::core::ffi::c_int = 0;
    let mut cxDesired: ::core::ffi::c_int = 0;
    let mut extra: ::core::ffi::c_int = 0;
    let mut extraOrig: ::core::ffi::c_int = 0;
    let mut cxIdealBand: ::core::ffi::c_int = 0;

    /* Validate */
    if (*infoPtr).uNumBands == 0 || iBand < 0 || iBand as UINT >= (*infoPtr).uNumBands {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_MaximizeBand\0" as *const u8 as *const ::core::ffi::c_char,
                2612 as ::core::ffi::c_int,
                b"Illegal MaximizeBand, requested=%d, current band count=%d\n\0"
                    as *const u8 as *const ::core::ffi::c_char,
                iBand,
                (*infoPtr).uNumBands,
            );
        }
        return (FALSE) as LRESULT;
    }

    lpBand = REBAR_GetBand(infoPtr, iBand);

    if ((*lpBand).fStyle & RBBS_HIDDEN) != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_MaximizeBand\0" as *const u8 as *const ::core::ffi::c_char,
                2621 as ::core::ffi::c_int,
                b"Ignoring maximize request on a hidden band (%d)\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                iBand,
            );
        }
        return (FALSE) as LRESULT;
    }

    cxIdealBand = (*lpBand)
        .cxIdeal
        .wrapping_add((*lpBand).cxHeader)
        .wrapping_add(REBAR_POST_CHILD) as ::core::ffi::c_int;
    if lParam != 0 && (*lpBand).cxEffective < cxIdealBand {
        cxDesired = cxIdealBand;
    } else {
        cxDesired = (*infoPtr).calcSize.cx as ::core::ffi::c_int;
    }

    iRowBegin = get_row_begin_for_band(infoPtr, iBand);
    iRowEnd = get_row_end_for_band(infoPtr, iBand);
    extra = (cxDesired - (*lpBand).cxEffective) as ::core::ffi::c_int;
    extraOrig = extra;
    if extra > 0 {
        extra = REBAR_ShrinkBandsRTL(infoPtr, iRowBegin as INT, iBand, extra as INT, TRUE);
    }
    if extra > 0 {
        extra = REBAR_ShrinkBandsLTR(
            infoPtr,
            next_visible(infoPtr, iBand as ::core::ffi::c_int) as INT,
            iRowEnd as INT,
            extra as INT,
            TRUE,
        );
    }
    (*lpBand).cxEffective += extraOrig - extra;
    (*lpBand).cx = (*lpBand).cxEffective as UINT;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_MaximizeBand\0" as *const u8 as *const ::core::ffi::c_char,
            2640 as ::core::ffi::c_int,
            b"(%d, %ld): Wanted size %d, obtained %d (shrink %d, %d)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            iBand,
            lParam,
            cxDesired,
            (*lpBand).cx,
            extraOrig,
            extra,
        );
    }

    REBAR_SetRowRectsX(infoPtr, (iRowBegin as UINT) as INT, (iRowEnd as UINT) as INT);

    if ((*infoPtr).dwStyle & CCS_VERT) != 0 {
        REBAR_CalcVertBand(infoPtr, iRowBegin as UINT, iRowEnd as UINT);
    } else {
        REBAR_CalcHorzBand(infoPtr, iRowBegin as UINT, iRowEnd as UINT);
    }
    REBAR_MoveChildWindows(infoPtr, iRowBegin as UINT, iRowEnd as UINT);
    return (TRUE) as LRESULT;
}

unsafe extern "C" fn REBAR_MinimizeBand(mut infoPtr: *const REBAR_INFO, mut iBand: INT) -> LRESULT {
    let mut lpBand: *mut REBAR_BAND = ::core::ptr::null_mut::<REBAR_BAND>();
    let mut iPrev: ::core::ffi::c_int = 0;
    let mut iRowBegin: ::core::ffi::c_int = 0;
    let mut iRowEnd: ::core::ffi::c_int = 0;
    if (*infoPtr).uNumBands == 0 as UINT
        || iBand < 0 as INT
        || iBand as UINT >= (*infoPtr).uNumBands
    {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_MinimizeBand\0" as *const u8 as *const ::core::ffi::c_char,
                2668 as ::core::ffi::c_int,
                b"Illegal MinimizeBand, requested=%d, current band count=%d\n\0"
                    as *const u8 as *const ::core::ffi::c_char,
                iBand,
                (*infoPtr).uNumBands,
            );
        }
        return FALSE as LRESULT;
    }
    lpBand = REBAR_GetBand(infoPtr, iBand);
    if (*lpBand).fStyle & RBBS_HIDDEN as UINT != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_MinimizeBand\0" as *const u8 as *const ::core::ffi::c_char,
                2678 as ::core::ffi::c_int,
                b"Ignoring minimize request on a hidden band (%d)\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                iBand,
            );
        }
        return FALSE as LRESULT;
    }
    iPrev = prev_visible(infoPtr, iBand as ::core::ffi::c_int);
    if iPrev < 0 as ::core::ffi::c_int
        || (*REBAR_GetBand(infoPtr, iPrev as INT)).iRow != (*lpBand).iRow
    {
        let mut iNext: ::core::ffi::c_int =
            next_visible(infoPtr, iBand as ::core::ffi::c_int);
        if (iNext as UINT) < (*infoPtr).uNumBands
            && (*REBAR_GetBand(infoPtr, iNext as INT)).iRow == (*lpBand).iRow
        {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/rebar.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"REBAR_MinimizeBand\0" as *const u8 as *const ::core::ffi::c_char,
                    2687 as ::core::ffi::c_int,
                    b"(%d): Minimizing the first band in row is by maximizing the second\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    iBand,
                );
            }
            REBAR_MaximizeBand(infoPtr, iNext as INT, FALSE as LPARAM);
        } else if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_MinimizeBand\0" as *const u8 as *const ::core::ffi::c_char,
                2691 as ::core::ffi::c_int,
                b"(%d): Only one band in row - nothing to do\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                iBand,
            );
        }
        return TRUE as LRESULT;
    }
    let fresh17: *mut REBAR_BAND = REBAR_GetBand(infoPtr, iPrev as INT);
    (*fresh17).cxEffective = ((*fresh17).cxEffective as UINT)
        .wrapping_add(((*lpBand).cxEffective as UINT).wrapping_sub((*lpBand).cxMinBand))
        as INT;
    (*REBAR_GetBand(infoPtr, iPrev as INT)).cx =
        (*REBAR_GetBand(infoPtr, iPrev as INT)).cxEffective as UINT;
    (*lpBand).cxEffective = (*lpBand).cxMinBand as INT;
    (*lpBand).cx = (*lpBand).cxEffective as UINT;
    iRowBegin = get_row_begin_for_band(infoPtr, iBand);
    iRowEnd = get_row_end_for_band(infoPtr, iBand);
    REBAR_SetRowRectsX(infoPtr, iRowBegin as INT, iRowEnd as INT);

    if (*infoPtr).dwStyle & CCS_VERT as DWORD != 0 {
        REBAR_CalcVertBand(infoPtr, iRowBegin as UINT, iRowEnd as UINT);
    } else {
        REBAR_CalcHorzBand(infoPtr, iRowBegin as UINT, iRowEnd as UINT);
    }
    REBAR_MoveChildWindows(infoPtr, iRowBegin as UINT, iRowEnd as UINT);
    return FALSE as LRESULT;
}

unsafe extern "C" fn REBAR_MoveBand(
    mut infoPtr: *mut REBAR_INFO,
    mut iFrom: INT,
    mut iTo: INT,
) -> LRESULT {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut::<REBAR_BAND>();

    /* Validate */
    if (*infoPtr).uNumBands == 0
        || iFrom < 0
        || iFrom as UINT >= (*infoPtr).uNumBands
        || iTo < 0
        || iTo as UINT >= (*infoPtr).uNumBands
    {
        /* error !!! */
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"REBAR_MoveBand\0" as *const u8 as *const core::ffi::c_char,
                2723 as core::ffi::c_int,
                b"Illegal MoveBand, from=%d, to=%d, current band count=%d\n\0"
                    as *const u8 as *const core::ffi::c_char,
                iFrom,
                iTo,
                (*infoPtr).uNumBands,
            );
        }
        return (FALSE) as LRESULT;
    }

    lpBand = REBAR_GetBand(infoPtr, iFrom);
    DPA_DeletePtr((*infoPtr).bands, iFrom as core::ffi::c_int);
    DPA_InsertPtr(
        (*infoPtr).bands,
        iTo as core::ffi::c_int,
        lpBand as *mut core::ffi::c_void,
    );

    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"REBAR_MoveBand\0" as *const u8 as *const core::ffi::c_char,
            2731 as core::ffi::c_int,
            b"moved band %d to index %d\n\0" as *const u8 as *const core::ffi::c_char,
            iFrom,
            iTo,
        );
    }

    /* **************************************************** */
    /*                                                      */
    /* We do not do a REBAR_Layout here because the native  */
    /* control does not do that. The actual layout and      */
    /* repaint is done by the *next* real action, ex.:      */
    /* RB_INSERTBAND, RB_DELETEBAND, RB_SIZETORECT, etc.    */
    /*                                                      */
    /* **************************************************** */
    REBAR_DumpBand(infoPtr);

    return (TRUE) as LRESULT;
}

unsafe extern "C" fn REBAR_strdifW(a: LPCWSTR, b: LPCWSTR) -> BOOL {
    return ((!a.is_null() && b.is_null())
        || (!b.is_null() && a.is_null())
        || (!a.is_null() && !b.is_null() && lstrcmpW(a, b) != 0)) as BOOL;
}

unsafe extern "C" fn REBAR_SetBandInfoT(
    infoPtr: *mut REBAR_INFO,
    iBand: INT,
    lprbbi: *const REBARBANDINFOW,
    bUnicode: BOOL,
) -> LRESULT {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut::<REBAR_BAND>();
    let mut uChanged: UINT = 0;

    if lprbbi.is_null() || ((*lprbbi).cbSize as usize) < (REBARBANDINFOA_V3_SIZE as usize) {
        return FALSE as LRESULT;
    }

    if (iBand as UINT) >= (*infoPtr).uNumBands {
        return FALSE as LRESULT;
    }

    REBAR_DumpBandInfo(lprbbi);

    /* set band information */
    lpBand = REBAR_GetBand(infoPtr, iBand);

    uChanged = REBAR_CommonSetupBand((*infoPtr).hwndSelf, lprbbi, lpBand);
    if ((*lprbbi).fMask & (RBBIM_TEXT as UINT)) != 0 {
        let mut wstr: LPWSTR = core::ptr::null_mut::<WCHAR>();
        if bUnicode != 0 {
            Str_SetPtrW(core::ptr::addr_of_mut!(wstr), (*lprbbi).lpText as LPCWSTR);
        } else {
            Str_SetPtrAtoW(
                core::ptr::addr_of_mut!(wstr),
                (*lprbbi).lpText as LPSTR as LPCSTR,
            );
        }

        if REBAR_strdifW(wstr as LPCWSTR, (*lpBand).lpText as LPCWSTR) != 0 {
            Free((*lpBand).lpText as LPVOID);
            (*lpBand).lpText = wstr;
            uChanged |= RBBIM_TEXT as UINT;
        } else {
            Free(wstr as LPVOID);
        }
    }

    REBAR_ValidateBand(infoPtr, lpBand);

    REBAR_DumpBand(infoPtr);

    if (uChanged
        & (RBBIM_CHILDSIZE as UINT
            | RBBIM_SIZE as UINT
            | RBBIM_STYLE as UINT
            | RBBIM_IMAGE as UINT))
        != 0
    {
        REBAR_Layout(infoPtr);
        InvalidateRect(
            (*infoPtr).hwndSelf,
            core::ptr::null::<RECT>(),
            TRUE as BOOL,
        );
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn REBAR_SetBarInfo(
    mut infoPtr: *mut REBAR_INFO,
    mut lpInfo: *const REBARINFO,
) -> LRESULT {
    let mut lpBand: *mut REBAR_BAND = ::core::ptr::null_mut::<REBAR_BAND>();
    let mut i: UINT = 0;
    if lpInfo.is_null()
        || ((*lpInfo).cbSize as usize) < core::mem::size_of::<REBARINFO>() as usize
    {
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
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_SetBarInfo\0" as *const u8 as *const ::core::ffi::c_char,
            2811 as ::core::ffi::c_int,
            b"setting bar info!\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    if ((*lpInfo).fMask & (RBIM_IMAGELIST as UINT)) != 0 {
        (*infoPtr).himl = ((*lpInfo).himl).cast();
        if !(*infoPtr).himl.is_null() {
            let mut cx: INT = 0;
            let mut cy: INT = 0;
            ImageList_GetIconSize((*infoPtr).himl, &raw mut cx, &raw mut cy);
            (*infoPtr).imageSize.cx = cx as LONG;
            (*infoPtr).imageSize.cy = cy as LONG;
        } else {
            (*infoPtr).imageSize.cx = 0 as LONG;
            (*infoPtr).imageSize.cy = 0 as LONG;
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
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"REBAR_SetBarInfo\0" as *const u8 as *const ::core::ffi::c_char,
                2826 as ::core::ffi::c_int,
                b"new image cx=%d, cy=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
                (*infoPtr).imageSize.cx,
                (*infoPtr).imageSize.cy,
            );
        }
    }
    i = 0 as UINT;
    while i < (*infoPtr).uNumBands {
        lpBand = REBAR_GetBand(infoPtr, i as INT);
        REBAR_ValidateBand(infoPtr, lpBand);
        i = i.wrapping_add(1);
    }
    return TRUE as LRESULT;
}

unsafe extern "C" fn REBAR_SetBkColor(mut infoPtr: *mut REBAR_INFO, mut clr: COLORREF) -> LRESULT {
    let mut clrTemp: COLORREF = 0;
    clrTemp = (*infoPtr).clrBk;
    (*infoPtr).clrBk = clr;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_SetBkColor\0" as *const u8 as *const ::core::ffi::c_char,
            2847 as ::core::ffi::c_int,
            b"background color 0x%06x!\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).clrBk,
        );
    }
    return clrTemp as LRESULT;
}

unsafe extern "C" fn REBAR_SetParent(infoPtr: *mut REBAR_INFO, parent: HWND) -> LRESULT {
    let mut hwndTemp: HWND = (*infoPtr).hwndNotify;
    (*infoPtr).hwndNotify = parent;
    return hwndTemp as LRESULT;
}

unsafe extern "C" fn REBAR_SetTextColor(
    mut infoPtr: *mut REBAR_INFO,
    mut clr: COLORREF,
) -> LRESULT {
    let mut clrTemp: COLORREF = 0;
    clrTemp = (*infoPtr).clrText;
    (*infoPtr).clrText = clr;
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"REBAR_SetTextColor\0" as *const u8 as *const core::ffi::c_char,
            2876 as core::ffi::c_int,
            b"text color 0x%06x!\n\0" as *const u8 as *const core::ffi::c_char,
            (*infoPtr).clrText,
        );
    }
    return clrTemp as LRESULT;
}

unsafe extern "C" fn REBAR_SetUnicodeFormat(infoPtr: *mut REBAR_INFO, unicode: BOOL) -> LRESULT {
    let mut bTemp: BOOL = (*infoPtr).bUnicode;

    (*infoPtr).bUnicode = unicode;

    return bTemp as LRESULT;
}

unsafe extern "C" fn REBAR_SetVersion(infoPtr: *mut REBAR_INFO, iVersion: INT) -> LRESULT {
    let mut iOldVersion: INT = (*infoPtr).iVersion;

    if iVersion > (COMCTL32_VERSION as i32) {
        return -1;
    }

    (*infoPtr).iVersion = iVersion;

    return iOldVersion as LRESULT;
}

unsafe extern "C" fn REBAR_ShowBand(
    mut infoPtr: *mut REBAR_INFO,
    mut iBand: INT,
    mut show: BOOL,
) -> LRESULT {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut::<REBAR_BAND>();
    if iBand < 0 || iBand as UINT >= (*infoPtr).uNumBands {
        return FALSE as LRESULT;
    }
    lpBand = REBAR_GetBand(infoPtr, iBand);
    if show != 0 {
        (*lpBand).fStyle = (*lpBand).fStyle & !RBBS_HIDDEN as UINT;
        if IsWindow((*lpBand).hwndChild) != 0 {
            ShowWindow((*lpBand).hwndChild, (SW_SHOW) as i32);
        }
    } else {
        (*lpBand).fStyle = (*lpBand).fStyle | RBBS_HIDDEN as UINT;
        if IsWindow((*lpBand).hwndChild) != 0 {
            ShowWindow((*lpBand).hwndChild, (SW_HIDE) as i32);
        }
    }
    REBAR_Layout(infoPtr);
    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), TRUE);
    return TRUE as LRESULT;
}

unsafe extern "C" fn REBAR_SizeToRect(
    mut infoPtr: *mut REBAR_INFO,
    mut flags: WPARAM,
    mut lpRect: *mut RECT,
) -> LRESULT {
    if lpRect.is_null() {
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
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_SizeToRect\0" as *const u8 as *const ::core::ffi::c_char,
            2955 as ::core::ffi::c_int,
            b"[%s]\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_rect(lpRect),
        );
    }
    REBAR_SizeToHeight(infoPtr, get_rect_cy(infoPtr, lpRect));

    /* Note that this undocumented flag is available on comctl32 v6 or later */
    if (flags & RBSTR_CHANGERECT as WPARAM) != 0 as WPARAM {
        let mut rcRebar: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcRebar));
        (*lpRect).bottom = (*lpRect).top + (rcRebar.bottom - rcRebar.top);
    }
    return TRUE as LRESULT;
}

unsafe extern "C" fn REBAR_Create(infoPtr: *mut REBAR_INFO, cs: LPCREATESTRUCTW) -> LRESULT {
    let mut wnrc1: RECT = core::mem::zeroed();
    let mut clrc1: RECT = core::mem::zeroed();

    if TRACE_ON!(rebar) {
        GetWindowRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(wnrc1));
        GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(clrc1));
    }

    if !OpenThemeData((*infoPtr).hwndSelf, core::ptr::addr_of!(themeClass) as LPCWSTR).is_null()
    {
        /* native seems to clear WS_BORDER when themed */
        (*infoPtr).dwStyle &= !WS_BORDER;
    }

    return 0;
}

unsafe extern "C" fn REBAR_Destroy(infoPtr: *mut REBAR_INFO) -> LRESULT {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut();
    let mut i: UINT;

    /* clean up each band */
    i = 0 as UINT;
    while i < (*infoPtr).uNumBands {
        lpBand = REBAR_GetBand(infoPtr, i as INT);

        /* delete text strings */
        Free((*lpBand).lpText as LPVOID);
        (*lpBand).lpText = core::ptr::null_mut();
        /* destroy child window */
        DestroyWindow((*lpBand).hwndChild);
        Free(lpBand as LPVOID);
        i = i.wrapping_add(1);
    }

    /* free band array */
    DPA_Destroy((*infoPtr).bands);
    (*infoPtr).bands = core::ptr::null_mut();

    DestroyCursor((*infoPtr).hcurArrow);
    DestroyCursor((*infoPtr).hcurHorz);
    DestroyCursor((*infoPtr).hcurVert);
    DestroyCursor((*infoPtr).hcurDrag);
    if !(*infoPtr).hDefaultFont.is_null() {
        DeleteObject(((*infoPtr).hDefaultFont).cast());
    }
    SetWindowLongPtrW((*infoPtr).hwndSelf, 0, 0);

    CloseThemeData(GetWindowTheme((*infoPtr).hwndSelf));

    /* free rebar info data */
    Free(infoPtr as LPVOID);

    return 0;
}

unsafe extern "C" fn REBAR_GetFont(infoPtr: *const REBAR_INFO) -> LRESULT {
    return (*infoPtr).hFont as LRESULT;
}

unsafe extern "C" fn REBAR_PushChevron(
    infoPtr: *const REBAR_INFO,
    uBand: UINT,
    lParam: LPARAM,
) -> LRESULT {
    if uBand < (*infoPtr).uNumBands {
        let mut nmrbc: NMREBARCHEVRON = tagNMREBARCHEVRON {
            hdr: tagNMHDR {
                hwndFrom: core::ptr::null_mut::<HWND__>(),
                idFrom: 0,
                code: 0,
            },
            uBand: 0,
            wID: 0,
            lParam: 0,
            rc: tagRECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            lParamNM: 0,
        };
        let lpBand: *mut REBAR_BAND = REBAR_GetBand(infoPtr, uBand as INT);
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"REBAR_PushChevron\0" as *const u8 as *const core::ffi::c_char,
                3048 as core::ffi::c_int,
                b"Pressed chevron on band %u\n\0" as *const u8
                    as *const core::ffi::c_char,
                uBand,
            );
        }

        /* redraw chevron in pushed state */
        (*lpBand).fDraw |= DRAW_CHEVRONPUSHED as UINT;
        RedrawWindow(
            (*infoPtr).hwndSelf,
            &raw mut (*lpBand).rcChevron as LPCRECT,
            0 as usize as HRGN,
            (RDW_ERASE | RDW_INVALIDATE | RDW_UPDATENOW) as UINT,
        );

        /* notify app so it can display a popup menu or whatever */
        nmrbc.uBand = uBand;
        nmrbc.wID = (*lpBand).wID;
        nmrbc.lParam = (*lpBand).lParam;
        nmrbc.rc = (*lpBand).rcChevron;
        nmrbc.lParamNM = lParam;
        REBAR_Notify(
            &raw mut nmrbc as *mut NMHDR,
            infoPtr,
            RBN_CHEVRONPUSHED as UINT,
        );

        /* redraw chevron in previous state */
        (*lpBand).fDraw &= !(DRAW_CHEVRONPUSHED as UINT);
        InvalidateRect(
            (*infoPtr).hwndSelf,
            &raw mut (*lpBand).rcChevron as LPCRECT,
            TRUE as BOOL,
        );

        return TRUE as LRESULT;
    }
    return FALSE as LRESULT;
}

unsafe extern "C" fn REBAR_LButtonDown(
    mut infoPtr: *mut REBAR_INFO,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut htFlags: UINT = 0;
    let mut iHitBand: INT = 0;
    let mut ptMouseDown: POINT = POINT { x: 0, y: 0 };
    ptMouseDown.x = (LOWORD!(lParam) as ::core::ffi::c_short) as LONG;
    ptMouseDown.y = (HIWORD!(lParam) as ::core::ffi::c_short) as LONG;

    REBAR_InternalHitTest(
        infoPtr,
        &mut ptMouseDown as *mut POINT,
        &mut htFlags as *mut UINT,
        &mut iHitBand as *mut INT,
    );

    if htFlags == RBHT_CHEVRON {
        REBAR_PushChevron(infoPtr, iHitBand as UINT, 0);
    } else if htFlags == RBHT_GRABBER || htFlags == RBHT_CAPTION {
        let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut::<REBAR_BAND>();

        lpBand = REBAR_GetBand(infoPtr, iHitBand);

        SetCapture((*infoPtr).hwndSelf);
        (*infoPtr).iGrabbedBand = iHitBand;

        /* save off the LOWORD and HIWORD of lParam as initial x,y */
        (*infoPtr).dragStart.x = (LOWORD!(lParam) as ::core::ffi::c_short) as LONG;
        (*infoPtr).dragStart.y = (HIWORD!(lParam) as ::core::ffi::c_short) as LONG;
        (*infoPtr).dragNow = (*infoPtr).dragStart;
        if ((*infoPtr).dwStyle & CCS_VERT) != 0 {
            (*infoPtr).ihitoffset = ((*infoPtr).dragStart.y as ::core::ffi::c_int
                - ((*lpBand).rcBand.left as ::core::ffi::c_int + (REBAR_PRE_GRIPPER as i32)))
                as INT;
        } else {
            (*infoPtr).ihitoffset = ((*infoPtr).dragStart.x as ::core::ffi::c_int
                - ((*lpBand).rcBand.left as ::core::ffi::c_int + (REBAR_PRE_GRIPPER as i32)))
                as INT;
        }
    }
    return 0;
}

unsafe extern "C" fn REBAR_LButtonUp(mut infoPtr: *mut REBAR_INFO) -> LRESULT {
    if (*infoPtr).iGrabbedBand >= 0 {
        let mut layout: NMHDR = core::mem::zeroed();
        let mut rect: RECT = core::mem::zeroed();

        (*infoPtr).dragStart.x = 0;
        (*infoPtr).dragStart.y = 0;
        (*infoPtr).dragNow = (*infoPtr).dragStart;

        ReleaseCapture();

        if ((*infoPtr).fStatus & (BEGIN_DRAG_ISSUED as UINT)) != 0 {
            REBAR_Notify(core::ptr::addr_of_mut!(layout), infoPtr, RBN_LAYOUTCHANGED as UINT);
            REBAR_Notify_NMREBAR(infoPtr, (*infoPtr).iGrabbedBand as UINT, RBN_ENDDRAG as UINT);
            (*infoPtr).fStatus &= !(BEGIN_DRAG_ISSUED as UINT);
        }

        (*infoPtr).iGrabbedBand = -(1 as INT);

        GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rect));
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), TRUE as BOOL);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn REBAR_MouseLeave(infoPtr: *mut REBAR_INFO) -> LRESULT {
    if (*infoPtr).ichevronhotBand >= (0 as INT) {
        let lpChevronBand: *mut REBAR_BAND = REBAR_GetBand(infoPtr, (*infoPtr).ichevronhotBand);
        if ((*lpChevronBand).fDraw & (DRAW_CHEVRONHOT as UINT)) != 0 {
            (*lpChevronBand).fDraw &= !(DRAW_CHEVRONHOT as UINT);
            InvalidateRect(
                (*infoPtr).hwndSelf,
                core::ptr::addr_of_mut!((*lpChevronBand).rcChevron) as *const RECT,
                (TRUE as BOOL),
            );
        }
    }
    (*infoPtr).iOldBand = -(1 as INT);
    (*infoPtr).ichevronhotBand = -(2 as INT);

    return TRUE as LRESULT;
}

unsafe extern "C" fn REBAR_MouseMove(infoPtr: *mut REBAR_INFO, lParam: LPARAM) -> LRESULT {
    let mut lpChevronBand: *mut REBAR_BAND = core::ptr::null_mut();
    let mut ptMove: POINT = POINT { x: 0, y: 0 };

    ptMove.x = (LOWORD!(lParam as DWORD_PTR)) as i16 as LONG;
    ptMove.y = (HIWORD!(lParam as DWORD_PTR)) as i16 as LONG;

    /* if we are currently dragging a band */
    if (*infoPtr).iGrabbedBand >= (0 as INT) {
        let mut band: *mut REBAR_BAND = core::ptr::null_mut();
        let mut yPtMove: core::ffi::c_int =
            if ((*infoPtr).dwStyle & (CCS_VERT as DWORD)) != 0 {
                ptMove.x as core::ffi::c_int
            } else {
                ptMove.y as core::ffi::c_int
            };

        if GetCapture() != (*infoPtr).hwndSelf {
            if (*__wine_dbch___default).flags as core::ffi::c_int
                & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/rebar.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"REBAR_MouseMove\0" as *const u8 as *const core::ffi::c_char,
                    3173 as core::ffi::c_int,
                    b"We are dragging but haven't got capture?!?\n\0" as *const u8
                        as *const core::ffi::c_char,
                );
            }
        }

        band = REBAR_GetBand(infoPtr, (*infoPtr).iGrabbedBand);

        /* if mouse did not move much, exit */
        if ((abs(ptMove.x as core::ffi::c_int - (*infoPtr).dragNow.x as core::ffi::c_int) as UINT)
            <= mindragx)
            && ((abs(ptMove.y as core::ffi::c_int - (*infoPtr).dragNow.y as core::ffi::c_int) as UINT)
                <= mindragy)
        {
            return 0 as LRESULT;
        }

        /* on first significant mouse movement, issue notify */
        if ((*infoPtr).fStatus & (BEGIN_DRAG_ISSUED as UINT)) == 0 {
            if REBAR_Notify_NMREBAR(
                infoPtr,
                (-(1 as core::ffi::c_int)) as UINT,
                RBN_BEGINDRAG as UINT,
            ) != 0
            {
                /* Notify returned TRUE - abort drag */
                (*infoPtr).dragStart.x = 0 as LONG;
                (*infoPtr).dragStart.y = 0 as LONG;
                (*infoPtr).dragNow = (*infoPtr).dragStart;
                (*infoPtr).iGrabbedBand = (-(1 as core::ffi::c_int)) as INT;
                ReleaseCapture();
                return 0 as LRESULT;
            }
            (*infoPtr).fStatus |= BEGIN_DRAG_ISSUED as UINT;
        }

        /* Test for valid drag case - must not be first band in row */
        if (yPtMove < (*band).rcBand.top) || (yPtMove > (*band).rcBand.bottom) {
            REBAR_HandleUDDrag(infoPtr, &mut ptMove as *mut POINT as *const POINT);
        } else {
            REBAR_HandleLRDrag(infoPtr, &mut ptMove as *mut POINT as *const POINT);
        }
    } else {
        let mut iHitBand: INT = 0;
        let mut htFlags: UINT = 0;
        let mut trackinfo: TRACKMOUSEEVENT = TRACKMOUSEEVENT {
            cbSize: 0,
            dwFlags: 0,
            hwndTrack: core::ptr::null_mut(),
            dwHoverTime: 0,
        };

        REBAR_InternalHitTest(
            infoPtr,
            &mut ptMove as *mut POINT as *const POINT,
            &mut htFlags as *mut UINT,
            &mut iHitBand as *mut INT,
        );

        if (*infoPtr).iOldBand >= (0 as INT)
            && (*infoPtr).iOldBand == (*infoPtr).ichevronhotBand
        {
            lpChevronBand = REBAR_GetBand(infoPtr, (*infoPtr).ichevronhotBand);
            if ((*lpChevronBand).fDraw & (DRAW_CHEVRONHOT as UINT)) != 0 {
                (*lpChevronBand).fDraw &= !(DRAW_CHEVRONHOT as UINT);
                InvalidateRect(
                    (*infoPtr).hwndSelf,
                    core::ptr::addr_of_mut!((*lpChevronBand).rcChevron) as LPCRECT,
                    TRUE as BOOL,
                );
            }
            (*infoPtr).ichevronhotBand = (-(2 as core::ffi::c_int)) as INT;
        }

        if htFlags == (RBHT_CHEVRON as UINT) {
            /* fill in the TRACKMOUSEEVENT struct */
            trackinfo.cbSize = core::mem::size_of::<TRACKMOUSEEVENT>() as DWORD;
            trackinfo.dwFlags = TME_QUERY as DWORD;
            trackinfo.hwndTrack = (*infoPtr).hwndSelf;
            trackinfo.dwHoverTime = 0 as DWORD;

            /* call _TrackMouseEvent to see if we are currently tracking for this hwnd */
            _TrackMouseEvent(&mut trackinfo as *mut TRACKMOUSEEVENT);

            /* Make sure tracking is enabled so we receive a WM_MOUSELEAVE message */
            if (trackinfo.dwFlags & (TME_LEAVE as DWORD)) == 0 {
                trackinfo.dwFlags = TME_LEAVE as DWORD; /* notify upon leaving */

                /* call TRACKMOUSEEVENT so we receive a WM_MOUSELEAVE message */
                /* and can properly deactivate the hot chevron */
                _TrackMouseEvent(&mut trackinfo as *mut TRACKMOUSEEVENT);
            }

            lpChevronBand = REBAR_GetBand(infoPtr, iHitBand);
            if ((*lpChevronBand).fDraw & (DRAW_CHEVRONHOT as UINT)) == 0 {
                (*lpChevronBand).fDraw |= DRAW_CHEVRONHOT as UINT;
                InvalidateRect(
                    (*infoPtr).hwndSelf,
                    core::ptr::addr_of_mut!((*lpChevronBand).rcChevron) as LPCRECT,
                    TRUE as BOOL,
                );
                (*infoPtr).ichevronhotBand = iHitBand;
            }
        }
        (*infoPtr).iOldBand = iHitBand;
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn REBAR_NCCalcSize(
    mut infoPtr: *const REBAR_INFO,
    mut rect: *mut RECT,
) -> LRESULT {
    let mut theme: HTHEME = ::core::ptr::null_mut::<::core::ffi::c_void>();
    if (*infoPtr).dwStyle & WS_BORDER != 0 {
        (*rect).left = min(
            (*rect).left + GetSystemMetrics((SM_CXEDGE) as i32),
            (*rect).right,
        );
        (*rect).right = max(
            (*rect).right - GetSystemMetrics((SM_CXEDGE) as i32),
            (*rect).left,
        );
        (*rect).top = min(
            (*rect).top + GetSystemMetrics((SM_CYEDGE) as i32),
            (*rect).bottom,
        );
        (*rect).bottom = max(
            (*rect).bottom - GetSystemMetrics((SM_CYEDGE) as i32),
            (*rect).top,
        );
    } else {
        theme = GetWindowTheme((*infoPtr).hwndSelf);
        if !theme.is_null() {
            /* FIXME: should use GetThemeInt */
            (*rect).top = if ((*rect).top as ::core::ffi::c_int + 1 as ::core::ffi::c_int)
                < (*rect).bottom
            {
                (*rect).top
            } else {
                (*rect).bottom
            };
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
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_NCCalcSize\0" as *const u8 as *const ::core::ffi::c_char,
            3279 as ::core::ffi::c_int,
            b"new client=(%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            wine_dbgstr_rect(rect),
        );
    }
    return 0;
}

unsafe extern "C" fn REBAR_NCCreate(hwnd: HWND, cs: *const CREATESTRUCTW) -> LRESULT {
    let mut infoPtr: *mut REBAR_INFO = REBAR_GetInfoPtr!(hwnd);
    let mut wnrc1: RECT = core::mem::zeroed();
    let mut clrc1: RECT = core::mem::zeroed();
    let mut ncm: NONCLIENTMETRICSW = core::mem::zeroed();
    let mut tfont: HFONT;

    if !infoPtr.is_null() {
        return FALSE as LRESULT;
    }

    if TRACE_ON!(rebar) {
        GetWindowRect(hwnd, core::ptr::addr_of_mut!(wnrc1));
        GetClientRect(hwnd, core::ptr::addr_of_mut!(clrc1));
    }

    /* allocate memory for info structure */
    infoPtr = Alloc(core::mem::size_of::<REBAR_INFO>() as DWORD) as *mut REBAR_INFO;
    SetWindowLongPtrW(hwnd, 0 as ::core::ffi::c_int, infoPtr as DWORD_PTR as LONG_PTR);

    /* initialize info structure - initial values are 0 */
    (*infoPtr).clrBk = CLR_NONE as COLORREF;
    (*infoPtr).clrText = CLR_NONE as COLORREF;
    (*infoPtr).clrBtnText = comctl32_color.clrBtnText;
    (*infoPtr).clrBtnFace = comctl32_color.clrBtnFace;
    (*infoPtr).iOldBand = -(1 as INT);
    (*infoPtr).ichevronhotBand = -(2 as INT);
    (*infoPtr).iGrabbedBand = -(1 as INT);
    (*infoPtr).hwndSelf = hwnd;
    (*infoPtr).DoRedraw = TRUE as BOOL;
    (*infoPtr).hcurArrow = LoadCursorW(core::ptr::null_mut(), IDC_ARROW as LPCWSTR);
    (*infoPtr).hcurHorz = LoadCursorW(core::ptr::null_mut(), IDC_SIZEWE as LPCWSTR);
    (*infoPtr).hcurVert = LoadCursorW(core::ptr::null_mut(), IDC_SIZENS as LPCWSTR);
    (*infoPtr).hcurDrag = LoadCursorW(core::ptr::null_mut(), IDC_SIZE as LPCWSTR);
    (*infoPtr).fStatus = 0 as UINT;
    (*infoPtr).hFont = GetStockObject((SYSTEM_FONT as i32)) as HFONT;
    (*infoPtr).bands = DPA_Create(8 as ::core::ffi::c_int);

    /* issue WM_NOTIFYFORMAT to get unicode status of parent */
    REBAR_NotifyFormat(infoPtr, NF_REQUERY as LPARAM);

    /* Stow away the original style */
    (*infoPtr).orgStyle = (*cs).style as DWORD;
    /* add necessary styles to the requested styles */
    (*infoPtr).dwStyle = ((*cs).style | ((WS_VISIBLE as DWORD)) as i32) as u32;
    if ((*infoPtr).dwStyle & (CCS_LAYOUT_MASK as DWORD)) == (0 as DWORD) {
        (*infoPtr).dwStyle |= CCS_TOP as DWORD;
    }
    SetWindowLongW(hwnd, GWL_STYLE as ::core::ffi::c_int, (*infoPtr).dwStyle as LONG);

    /* get font handle for Caption Font */
    ncm.cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as UINT;
    SystemParametersInfoW(
        SPI_GETNONCLIENTMETRICS as UINT,
        ncm.cbSize,
        core::ptr::addr_of_mut!(ncm) as PVOID,
        0 as UINT,
    );
    /* if the font is bold, set to normal */
    if ncm.lfCaptionFont.lfWeight > (FW_NORMAL as LONG) {
        ncm.lfCaptionFont.lfWeight = FW_NORMAL as LONG;
    }
    tfont = CreateFontIndirectW(core::ptr::addr_of_mut!(ncm.lfCaptionFont));
    if !tfont.is_null() {
        (*infoPtr).hDefaultFont = tfont;
        (*infoPtr).hFont = (*infoPtr).hDefaultFont;
    }

    return TRUE as LRESULT;
}

unsafe extern "C" fn REBAR_NCHitTest(infoPtr: *const REBAR_INFO, lParam: LPARAM) -> LRESULT {
    let mut nmmouse: NMMOUSE = core::mem::zeroed();
    let mut clpt: POINT = core::mem::zeroed();
    let mut i: INT = 0;
    let mut scrap: UINT = 0;
    let mut ret: LRESULT = HTCLIENT as LRESULT;

    /*
     * Differences from doc at MSDN (as observed with version 4.71 of
     *      comctl32.dll
     * 1. doc says nmmouse.pt is in screen coord, trace shows client coord.
     * 2. if band is not identified .dwItemSpec is 0xffffffff.
     * 3. native always seems to return HTCLIENT if notify return is 0.
     */

    clpt.x = LOWORD!(lParam as DWORD_PTR) as i16 as LONG;
    clpt.y = HIWORD!(lParam as DWORD_PTR) as i16 as LONG;
    ScreenToClient((*infoPtr).hwndSelf, &raw mut clpt);
    REBAR_InternalHitTest(
        infoPtr,
        &raw mut clpt,
        &raw mut scrap,
        &raw mut nmmouse.dwItemSpec as *mut INT,
    );
    nmmouse.dwItemData = 0 as DWORD_PTR;
    nmmouse.pt = clpt;
    nmmouse.dwHitInfo = scrap as LPARAM;
    i = REBAR_Notify(
        &raw mut nmmouse as *mut NMHDR,
        infoPtr,
        NM_NCHITTEST as UINT,
    );
    if i != 0 {
        ret = i as LRESULT;
    }
    return ret;
}

unsafe extern "C" fn REBAR_NCPaint(infoPtr: *const REBAR_INFO) -> LRESULT {
    let mut rcWindow: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut hdc: HDC = core::ptr::null_mut();
    let mut theme: HTHEME = core::ptr::null_mut();

    if ((*infoPtr).dwStyle & WS_MINIMIZE) != 0 {
        return 0 as LRESULT; /* Nothing to do */
    }

    if ((*infoPtr).dwStyle & WS_BORDER) != 0 {
        /* adjust rectangle and draw the necessary edge */
        hdc = GetDCEx(
            (*infoPtr).hwndSelf,
            core::ptr::null_mut(),
            (DCX_USESTYLE | DCX_WINDOW) as DWORD,
        );
        if hdc.is_null() {
            return 0 as LRESULT;
        }
        GetWindowRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcWindow));
        OffsetRect(
            core::ptr::addr_of_mut!(rcWindow),
            -(rcWindow.left as core::ffi::c_int),
            -(rcWindow.top as core::ffi::c_int),
        );

        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/rebar.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"REBAR_NCPaint\0" as *const u8 as *const core::ffi::c_char,
                3410 as core::ffi::c_int,
                b"rect (%s)\n\0" as *const u8 as *const core::ffi::c_char,
                wine_dbgstr_rect(core::ptr::addr_of_mut!(rcWindow)),
            );
        }

        DrawEdge(
            hdc,
            core::ptr::addr_of_mut!(rcWindow),
            EDGE_ETCHED as UINT,
            BF_RECT as UINT,
        );
        ReleaseDC((*infoPtr).hwndSelf, hdc);
    } else {
        theme = GetWindowTheme((*infoPtr).hwndSelf);
        if !theme.is_null() {
            /* adjust rectangle and draw the necessary edge */
            hdc = GetDCEx(
                (*infoPtr).hwndSelf,
                core::ptr::null_mut(),
                (DCX_USESTYLE | DCX_WINDOW) as DWORD,
            );
            if hdc.is_null() {
                return 0 as LRESULT;
            }
            GetWindowRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcWindow));
            OffsetRect(
                core::ptr::addr_of_mut!(rcWindow),
                -(rcWindow.left as core::ffi::c_int),
                -(rcWindow.top as core::ffi::c_int),
            );

            if (*__wine_dbch___default).flags as core::ffi::c_int
                & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/rebar.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"REBAR_NCPaint\0" as *const u8 as *const core::ffi::c_char,
                    3421 as core::ffi::c_int,
                    b"rect (%s)\n\0" as *const u8 as *const core::ffi::c_char,
                    wine_dbgstr_rect(core::ptr::addr_of_mut!(rcWindow)),
                );
            }

            DrawThemeEdge(
                theme,
                hdc,
                0,
                0,
                core::ptr::addr_of_mut!(rcWindow),
                BDR_RAISEDINNER as UINT,
                BF_TOP as UINT,
                core::ptr::null_mut::<RECT>(),
            );
            ReleaseDC((*infoPtr).hwndSelf, hdc);
        }
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn REBAR_NotifyFormat(infoPtr: *mut REBAR_INFO, cmd: LPARAM) -> LRESULT {
    let mut i: INT = 0;

    if cmd == NF_REQUERY as LPARAM {
        i = SendMessageW(
            REBAR_GetNotifyParent(infoPtr),
            WM_NOTIFYFORMAT as UINT,
            (*infoPtr).hwndSelf as WPARAM,
            NF_QUERY as LPARAM,
        ) as INT;
        if i != NFR_ANSI as INT && i != NFR_UNICODE as INT {
            i = NFR_ANSI as INT;
        }
        (*infoPtr).bUnicode = (i == NFR_UNICODE as INT) as BOOL;
        return i as LRESULT;
    }

    return (if (*infoPtr).bUnicode != 0 {
        NFR_UNICODE
    } else {
        NFR_ANSI
    }) as LRESULT;
}

unsafe extern "C" fn REBAR_Paint(infoPtr: *const REBAR_INFO, mut hdc: HDC) -> LRESULT {
    if !hdc.is_null() {
        REBAR_EraseBkGnd(infoPtr, hdc);
        REBAR_Refresh(infoPtr, hdc);
    } else {
        let mut ps: PAINTSTRUCT = core::mem::zeroed();
        hdc = BeginPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
        if ps.fErase != 0 {
            /* Erase area of paint if requested */
            REBAR_EraseBkGnd(infoPtr, hdc);
        }
        REBAR_Refresh(infoPtr, hdc);
        EndPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
    }

    return 0;
}

unsafe extern "C" fn REBAR_SetCursor(
    infoPtr: *const REBAR_INFO,
    lParam: LPARAM,
) -> LRESULT {
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut flags: UINT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_SetCursor\0" as *const u8 as *const ::core::ffi::c_char,
            3480 as ::core::ffi::c_int,
            b"code=0x%X  id=0x%X\n\0" as *const u8 as *const ::core::ffi::c_char,
            LOWORD!(lParam as DWORD) as ::core::ffi::c_int,
            HIWORD!(lParam as DWORD) as ::core::ffi::c_int,
        );
    }

    GetCursorPos(core::ptr::addr_of_mut!(pt));
    ScreenToClient((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(pt));

    REBAR_InternalHitTest(
        infoPtr,
        core::ptr::addr_of_mut!(pt),
        core::ptr::addr_of_mut!(flags),
        core::ptr::null_mut::<INT>(),
    );

    if flags == RBHT_GRABBER {
        if ((*infoPtr).dwStyle & CCS_VERT) != 0
            && ((*infoPtr).dwStyle & RBS_VERTICALGRIPPER) == 0
        {
            SetCursor((*infoPtr).hcurVert);
        } else {
            SetCursor((*infoPtr).hcurHorz);
        }
    } else if flags != RBHT_CLIENT {
        SetCursor((*infoPtr).hcurArrow);
    }

    return 0;
}

unsafe extern "C" fn REBAR_SetFont(infoPtr: *mut REBAR_INFO, font: HFONT) -> LRESULT {
    let mut lpBand: *mut REBAR_BAND = core::ptr::null_mut();
    let mut i: UINT;

    (*infoPtr).hFont = font;

    /* revalidate all bands to change sizes of text in headers of bands */
    i = 0;
    while i < (*infoPtr).uNumBands {
        lpBand = REBAR_GetBand(infoPtr, i as INT);
        REBAR_ValidateBand(infoPtr, lpBand);
        i = i.wrapping_add(1);
    }

    REBAR_Layout(infoPtr);
    return 0;
}

unsafe extern "C" fn REBAR_SetRedraw(infoPtr: *mut REBAR_INFO, redraw: BOOL) -> LRESULT {
    let oldredraw: BOOL = (*infoPtr).DoRedraw;
    (*infoPtr).DoRedraw = redraw;
    if redraw != 0 {
        if ((*infoPtr).fStatus & (BAND_NEEDS_REDRAW as UINT)) != 0 {
            REBAR_MoveChildWindows(infoPtr, 0 as UINT, (*infoPtr).uNumBands);
            REBAR_ForceResize(infoPtr);
            InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), TRUE as BOOL);
        }
        (*infoPtr).fStatus &= !(BAND_NEEDS_REDRAW as UINT);
    }
    return (if oldredraw != 0 {
        -1
    } else {
        0
    }) as LRESULT;
}

unsafe extern "C" fn REBAR_Size(infoPtr: *mut REBAR_INFO, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    /* avoid _Layout resize recursion (but it shouldn't be infinite and it seems Windows does recurse) */
    if ((*infoPtr).fStatus & SELF_RESIZE) != 0 {
        (*infoPtr).fStatus &= !(SELF_RESIZE);
        return 0;
    }

    if ((*infoPtr).dwStyle & RBS_AUTOSIZE) != 0 {
        REBAR_AutoSize(infoPtr, TRUE);
    } else {
        REBAR_Layout(infoPtr);
    }

    return 0;
}

unsafe extern "C" fn REBAR_StyleChanged(
    mut infoPtr: *mut REBAR_INFO,
    mut nType: INT,
    mut lpStyle: *const STYLESTRUCT,
) -> LRESULT {
    if nType == GWL_STYLE {
        (*infoPtr).dwStyle = (*lpStyle).styleNew;
        (*infoPtr).orgStyle = (*infoPtr).dwStyle;
        if !GetWindowTheme((*infoPtr).hwndSelf).is_null() {
            (*infoPtr).dwStyle &= !(WS_BORDER) as DWORD;
        }
        /* maybe it should be COMMON_STYLES like in toolbar */
        if (((*lpStyle).styleNew ^ (*lpStyle).styleOld) & CCS_VERT) != 0 {
            REBAR_Layout(infoPtr);
        }
    }
    return FALSE as LRESULT;
}

unsafe extern "C" fn theme_changed(mut infoPtr: *mut REBAR_INFO) -> LRESULT {
    let mut theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    CloseThemeData(theme);
    theme = OpenThemeData((*infoPtr).hwndSelf, core::ptr::addr_of!(themeClass) as LPCWSTR);
    /* WS_BORDER disappears when theming is enabled and reappears when
     * disabled... */
    (*infoPtr).dwStyle &= !(WS_BORDER as DWORD);
    (*infoPtr).dwStyle |= if !theme.is_null() {
        0 as DWORD
    } else {
        (*infoPtr).orgStyle & (WS_BORDER as DWORD)
    };
    return 0 as LRESULT;
}

unsafe extern "C" fn REBAR_WindowPosChanged(
    infoPtr: *const REBAR_INFO,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut ret: LRESULT = 0;
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    ret = DefWindowProcW(
        (*infoPtr).hwndSelf,
        WM_WINDOWPOSCHANGED as UINT,
        wParam,
        lParam,
    );
    GetWindowRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rc));

    return ret;
}

unsafe extern "system" fn REBAR_WindowProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: *mut REBAR_INFO = REBAR_GetInfoPtr!(hwnd) as *mut REBAR_INFO;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/rebar.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"REBAR_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            3622 as ::core::ffi::c_int,
            b"hwnd=%p msg=%x wparam=%lx lparam=%lx\n\0" as *const u8 as *const ::core::ffi::c_char,
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
        RB_DELETEBAND => return REBAR_DeleteBand(infoPtr, wParam),
        RB_GETBANDBORDERS => return REBAR_GetBandBorders(infoPtr, (wParam) as UINT, lParam as LPRECT),
        RB_GETBANDCOUNT => return REBAR_GetBandCount(infoPtr),
        RB_GETBANDINFO_OLD | RB_GETBANDINFOA | RB_GETBANDINFOW => {
            return REBAR_GetBandInfoT(
                infoPtr,
                (wParam) as UINT,
                lParam as LPREBARBANDINFOW,
                (uMsg == RB_GETBANDINFOW) as BOOL,
            );
        }
        RB_GETBARHEIGHT => return REBAR_GetBarHeight(infoPtr),
        RB_GETBARINFO => return REBAR_GetBarInfo(infoPtr, lParam as LPREBARINFO),
        RB_GETBKCOLOR => return REBAR_GetBkColor(infoPtr),
        RB_GETPALETTE => return REBAR_GetPalette(infoPtr),
        RB_GETRECT => return REBAR_GetRect(infoPtr, (wParam) as INT, lParam as LPRECT),
        RB_GETROWCOUNT => return REBAR_GetRowCount(infoPtr),
        RB_GETROWHEIGHT => return REBAR_GetRowHeight(infoPtr, (wParam) as INT),
        RB_GETTEXTCOLOR => return REBAR_GetTextColor(infoPtr),
        RB_GETTOOLTIPS => return REBAR_GetToolTips(infoPtr),
        RB_GETUNICODEFORMAT => return REBAR_GetUnicodeFormat(infoPtr),
        CCM_GETVERSION => return REBAR_GetVersion(infoPtr),
        RB_HITTEST => return REBAR_HitTest(infoPtr, lParam as LPRBHITTESTINFO),
        RB_IDTOINDEX => return REBAR_IdToIndex(infoPtr, (wParam) as UINT),
        RB_INSERTBANDA | RB_INSERTBANDW => {
            return REBAR_InsertBandT(
                infoPtr,
                (wParam) as INT,
                lParam as LPREBARBANDINFOW,
                (uMsg == RB_INSERTBANDW) as BOOL,
            );
        }
        RB_MAXIMIZEBAND => return REBAR_MaximizeBand(infoPtr, (wParam) as INT, lParam),
        RB_MINIMIZEBAND => return REBAR_MinimizeBand(infoPtr, (wParam) as INT),
        RB_MOVEBAND => return REBAR_MoveBand(infoPtr, (wParam) as INT, (lParam) as INT),
        RB_PUSHCHEVRON => return REBAR_PushChevron(infoPtr, (wParam) as UINT, lParam),
        RB_SETBANDINFOA | RB_SETBANDINFOW => {
            return REBAR_SetBandInfoT(
                infoPtr,
                (wParam) as INT,
                lParam as LPREBARBANDINFOW,
                (uMsg == RB_SETBANDINFOW) as BOOL,
            );
        }
        RB_SETBARINFO => return REBAR_SetBarInfo(infoPtr, lParam as LPREBARINFO),
        RB_SETBKCOLOR => return REBAR_SetBkColor(infoPtr, (lParam) as COLORREF),
        RB_SETPARENT => return REBAR_SetParent(infoPtr, wParam as HWND),
        RB_SETTEXTCOLOR => return REBAR_SetTextColor(infoPtr, (lParam) as COLORREF),
        RB_SETUNICODEFORMAT => return REBAR_SetUnicodeFormat(infoPtr, (wParam) as BOOL),
        CCM_SETVERSION => return REBAR_SetVersion(infoPtr, wParam as INT),
        RB_SHOWBAND => return REBAR_ShowBand(infoPtr, (wParam) as INT, (lParam) as BOOL),
        RB_SIZETORECT => return REBAR_SizeToRect(infoPtr, wParam, lParam as LPRECT),

        /*    Messages passed to parent */
        WM_COMMAND
        | WM_DRAWITEM
        | WM_NOTIFY
        | WM_MEASUREITEM => {
            return SendMessageW(REBAR_GetNotifyParent(infoPtr), uMsg, wParam, lParam);
        }

        WM_CREATE => {
            return REBAR_Create(infoPtr, lParam as LPCREATESTRUCTW);
        }
        WM_DESTROY => return REBAR_Destroy(infoPtr),
        WM_ERASEBKGND => return REBAR_EraseBkGnd(infoPtr, wParam as HDC),
        WM_GETFONT => return REBAR_GetFont(infoPtr),
        WM_LBUTTONDOWN => return REBAR_LButtonDown(infoPtr, lParam),
        WM_LBUTTONUP => return REBAR_LButtonUp(infoPtr),
        WM_MOUSEMOVE => return REBAR_MouseMove(infoPtr, lParam),
        WM_MOUSELEAVE => return REBAR_MouseLeave(infoPtr),
        WM_NCCALCSIZE => return REBAR_NCCalcSize(infoPtr, lParam as *mut RECT),
        WM_NCCREATE => {
            return REBAR_NCCreate(hwnd, lParam as LPCREATESTRUCTW);
        }
        WM_NCHITTEST => return REBAR_NCHitTest(infoPtr, lParam),
        WM_NCPAINT => return REBAR_NCPaint(infoPtr),
        WM_NOTIFYFORMAT => return REBAR_NotifyFormat(infoPtr, lParam),
        WM_PRINTCLIENT | WM_PAINT => {
            return REBAR_Paint(infoPtr, wParam as HDC);
        }

        WM_SETCURSOR => return REBAR_SetCursor(infoPtr, lParam),
        WM_SETFONT => return REBAR_SetFont(infoPtr, wParam as HFONT),
        WM_SETREDRAW => return REBAR_SetRedraw(infoPtr, wParam as BOOL),
        WM_SIZE => return REBAR_Size(infoPtr, wParam, lParam),
        WM_STYLECHANGED => {
            return REBAR_StyleChanged(infoPtr, wParam as INT, lParam as LPSTYLESTRUCT);
        }
        WM_THEMECHANGED => return theme_changed(infoPtr),
        WM_SYSCOLORCHANGE => {
            COMCTL32_RefreshSysColors();
            /* r51522 - Properly support WM_SYSCOLORCHANGE */
            (*infoPtr).clrBtnText = comctl32_color.clrBtnText;
            (*infoPtr).clrBtnFace = comctl32_color.clrBtnFace;
            return 0;
        }

        WM_WINDOWPOSCHANGED => return REBAR_WindowPosChanged(infoPtr, wParam, lParam),

        _ => {
            if (uMsg >= WM_USER)
                && (uMsg < WM_APP)
                && (COMCTL32_IsReflectedMessage(uMsg) == 0)
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
                        b"dll/win32/comctl32/rebar.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"REBAR_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        3838 as ::core::ffi::c_int,
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
    };
}
#[no_mangle]
pub unsafe extern "C" fn REBAR_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(&raw mut wndClass as *mut core::ffi::c_void, core::mem::size_of::<WNDCLASSW>());
    wndClass.style = (CS_GLOBALCLASS | CS_DBLCLKS) as UINT;
    wndClass.lpfnWndProc =
        Some(REBAR_WindowProc as _);
    wndClass.cbClsExtra = 0 as core::ffi::c_int;
    wndClass.cbWndExtra = core::mem::size_of::<*mut REBAR_INFO>() as core::ffi::c_int;
    wndClass.hCursor = core::ptr::null_mut();
    wndClass.hbrBackground = (COLOR_BTNFACE + 1) as HBRUSH;
    if false {
        wndClass.hbrBackground = CreateSolidBrush(RGB!(
            0,
            128,
            0
        ));
    }
    wndClass.lpszClassName = (REBARCLASSNAMEW).as_ptr();

    RegisterClassW(&raw mut wndClass);

    mindragx = GetSystemMetrics((SM_CXDRAG) as i32) as UINT;
    mindragy = GetSystemMetrics((SM_CYDRAG) as i32) as UINT;
}
#[no_mangle]
pub unsafe extern "C" fn REBAR_Unregister() {
    UnregisterClassW((REBARCLASSNAMEW).as_ptr(), core::ptr::null_mut());
}
