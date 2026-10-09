use crate::comctl32::comctl32undoc::Alloc;
use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::comctl32::comctl32undoc::Free;
use crate::comctl32::comctl32undoc::ReAlloc;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_trackbar);
// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct TRACKBAR_INFO {
    pub hwndSelf: HWND,
    pub dwStyle: DWORD,
    pub lRangeMin: LONG,
    pub lRangeMax: LONG,
    pub lLineSize: LONG,
    pub lPageSize: LONG,
    pub lSelMin: LONG,
    pub lSelMax: LONG,
    pub lPos: LONG,
    pub uThumbLen: UINT,
    pub uNumTics: UINT,
    pub uTicFreq: UINT,
    pub hwndNotify: HWND,
    pub hwndToolTip: HWND,
    pub hwndBuddyLA: HWND,
    pub hwndBuddyRB: HWND,
    pub fLocation: INT,
    pub flags: DWORD,
    pub bUnicode: BOOL,
    pub rcChannel: RECT,
    pub rcSelection: RECT,
    pub rcThumb: RECT,
    pub tics: LPLONG,
}

pub type PTRACKBAR_INFO = *mut TRACKBAR_INFO;


pub const TB_REFRESH_TIMER: usize = 1;
pub const TB_REFRESH_DELAY: u32 = 500;
pub const TOOLTIP_OFFSET: i32 = 2; // distance from ctrl edge to tooltip
pub const TB_DEFAULTPAGESIZE: i32 = 20;

pub const TB_THUMBPOSCHANGED: u32 = 0x00000001;
pub const TB_THUMBSIZECHANGED: u32 = 0x00000002;
pub const TB_THUMBCHANGED: u32 = TB_THUMBPOSCHANGED | TB_THUMBSIZECHANGED;
pub const TB_SELECTIONCHANGED: u32 = 0x00000004;
pub const TB_DRAG_MODE: u32 = 0x00000008; // we're dragging the slider
pub const TB_AUTO_PAGE_LEFT: u32 = 0x00000010;
pub const TB_AUTO_PAGE_RIGHT: u32 = 0x00000020;
pub const TB_AUTO_PAGE: u32 = TB_AUTO_PAGE_LEFT | TB_AUTO_PAGE_RIGHT;
pub const TB_THUMB_HOT: u32 = 0x00000040; // mouse hovers above thumb
pub const TB_USER_PAGE: u32 = 0x00000080;
pub const TB_IS_FOCUSED: u32 = 0x00000100;

pub const TIC_EDGE: u32 = 0x20;
pub const TIC_SELECTIONMARKMAX: u32 = 0x80;
pub const TIC_SELECTIONMARKMIN: u32 = 0x100;
pub const TIC_SELECTIONMARK: u32 = TIC_SELECTIONMARKMAX | TIC_SELECTIONMARKMIN;

pub static themeClass: [u16; 9] = [0x0054u16, 0x0072, 0x0061, 0x0063, 0x006B, 0x0062, 0x0061, 0x0072, 0x0000];




unsafe extern "C" fn notify_customdraw(infoPtr: *const TRACKBAR_INFO, pnmcd: *mut NMCUSTOMDRAW, stage: i32) -> i32 {
    (*pnmcd).dwDrawStage = stage as DWORD;
    return SendMessageW((*infoPtr).hwndNotify, WM_NOTIFY, (*pnmcd).hdr.idFrom, pnmcd as LPARAM) as i32;
}

unsafe extern "C" fn notify_hdr(infoPtr: *const TRACKBAR_INFO, code: INT, pnmh: LPNMHDR) -> LRESULT {
    let mut result: LRESULT;

    (*pnmh).hwndFrom = (*infoPtr).hwndSelf;
    (*pnmh).idFrom = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID) as _;
    (*pnmh).code = code as _;
    result = SendMessageW((*infoPtr).hwndNotify, WM_NOTIFY, (*pnmh).idFrom as _, pnmh as LPARAM);

    return result;
}

unsafe extern "C" fn notify(infoPtr: *const TRACKBAR_INFO, code: INT) -> i32 {
    let mut nmh: NMHDR = core::mem::zeroed();
    return notify_hdr(infoPtr, code, core::ptr::addr_of_mut!(nmh)) as i32;
}

unsafe extern "C" fn notify_with_scroll(infoPtr: *const TRACKBAR_INFO, code: UINT) {
    let scroll: UINT = if ((*infoPtr).dwStyle & TBS_VERT) != 0 { WM_VSCROLL } else { WM_HSCROLL };

    SendMessageW((*infoPtr).hwndNotify, scroll, (code) as WPARAM, (*infoPtr).hwndSelf as LPARAM);
}

unsafe extern "C" fn TRACKBAR_RecalculateTics(infoPtr: *mut TRACKBAR_INFO) {
    let mut tic: i32;
    let mut nrTics: u32;
    let mut i: u32;

    if (*infoPtr).uTicFreq != 0 && (*infoPtr).lRangeMax >= (*infoPtr).lRangeMin {
        nrTics = ((*infoPtr).lRangeMax.wrapping_sub((*infoPtr).lRangeMin) as u32) / (*infoPtr).uTicFreq;
        /* don't add extra tic if there's no remainder */
        if nrTics != 0 && (((*infoPtr).lRangeMax.wrapping_sub((*infoPtr).lRangeMin) as u32) % (*infoPtr).uTicFreq == 0) {
            nrTics -= 1;
        }
    } else {
        Free((*infoPtr).tics as *mut core::ffi::c_void);
        (*infoPtr).tics = core::ptr::null_mut();
        (*infoPtr).uNumTics = 0;
        return;
    }

    if nrTics != (*infoPtr).uNumTics {
        (*infoPtr).tics = ReAlloc(
            (*infoPtr).tics as *mut core::ffi::c_void,
            ((nrTics as usize + 1 as usize) * core::mem::size_of::<DWORD>()) as DWORD,
        ) as *mut LONG;
        if (*infoPtr).tics.is_null() {
            (*infoPtr).uNumTics = 0;
            notify(infoPtr, (NM_OUTOFMEMORY) as i32);
            return;
        }
        (*infoPtr).uNumTics = nrTics;
    }

    tic = ((*infoPtr).lRangeMin as u32).wrapping_add((*infoPtr).uTicFreq) as i32;
    i = 0;
    while i < nrTics {
        *(*infoPtr).tics.add(i as usize) = tic;
        i += 1;
        tic = tic.wrapping_add((*infoPtr).uTicFreq as i32);
    }
}

unsafe extern "C" fn TRACKBAR_ConvertPlaceToPosition(infoPtr: *const TRACKBAR_INFO, place: i32) -> LONG {
    let mut range: f64 = 0.0;
    let mut width: f64 = 0.0;
    let mut pos: f64 = 0.0;
    let mut offsetthumb: f64 = 0.0;

    range = ((*infoPtr).lRangeMax - (*infoPtr).lRangeMin) as f64;
    if ((*infoPtr).dwStyle & TBS_VERT) != 0 {
        offsetthumb = (((*infoPtr).rcThumb.bottom - (*infoPtr).rcThumb.top) as f64) / (2 as f64);
        width = ((*infoPtr).rcChannel.bottom - (*infoPtr).rcChannel.top) as f64 - (offsetthumb * (2 as f64)) - (1 as f64);
        pos = (range * ((place - (*infoPtr).rcChannel.top) as f64 - offsetthumb)) / width;
    } else {
        offsetthumb = (((*infoPtr).rcThumb.right - (*infoPtr).rcThumb.left) as f64) / (2 as f64);
        width = ((*infoPtr).rcChannel.right - (*infoPtr).rcChannel.left) as f64 - (offsetthumb * (2 as f64)) - (1 as f64);
        pos = (range * ((place - (*infoPtr).rcChannel.left) as f64 - offsetthumb)) / width;
    }
    pos += (*infoPtr).lRangeMin as f64;
    if pos > (*infoPtr).lRangeMax as f64 {
        pos = (*infoPtr).lRangeMax as f64;
    } else if pos < (*infoPtr).lRangeMin as f64 {
        pos = (*infoPtr).lRangeMin as f64;
    }

    return floor(pos + 0.5) as LONG;
}

unsafe extern "C" fn TRACKBAR_GetAutoPageDirection(infoPtr: *const TRACKBAR_INFO, clickPoint: POINT) -> LONG {
    let mut pageRect: RECT = core::mem::zeroed();

    if ((*infoPtr).dwStyle & TBS_VERT) != 0 {
        pageRect.top = (*infoPtr).rcChannel.top;
        pageRect.bottom = (*infoPtr).rcChannel.bottom;
        pageRect.left = (*infoPtr).rcThumb.left;
        pageRect.right = (*infoPtr).rcThumb.right;
    } else {
        pageRect.top = (*infoPtr).rcThumb.top;
        pageRect.bottom = (*infoPtr).rcThumb.bottom;
        pageRect.left = (*infoPtr).rcChannel.left;
        pageRect.right = (*infoPtr).rcChannel.right;
    }

    if PtInRect(core::ptr::addr_of!(pageRect), clickPoint) != 0 {
        let clickPlace: i32 = if ((*infoPtr).dwStyle & TBS_VERT) != 0 {
            clickPoint.y
        } else {
            clickPoint.x
        };

        let clickPos: LONG = TRACKBAR_ConvertPlaceToPosition(infoPtr, clickPlace);

        return clickPos - (*infoPtr).lPos;
    }

    return 0;
}

unsafe extern "C" fn TRACKBAR_PageDown(infoPtr: *mut TRACKBAR_INFO) {
    if (*infoPtr).lPos == (*infoPtr).lRangeMax {
        return;
    }

    (*infoPtr).lPos += (*infoPtr).lPageSize;
    if (*infoPtr).lPos > (*infoPtr).lRangeMax {
        (*infoPtr).lPos = (*infoPtr).lRangeMax;
    }
    notify_with_scroll(infoPtr, TB_PAGEDOWN);
}

unsafe extern "C" fn TRACKBAR_PageUp(infoPtr: *mut TRACKBAR_INFO) {
    if (*infoPtr).lPos == (*infoPtr).lRangeMin {
        return;
    }

    (*infoPtr).lPos -= (*infoPtr).lPageSize;
    if (*infoPtr).lPos < (*infoPtr).lRangeMin {
        (*infoPtr).lPos = (*infoPtr).lRangeMin;
    }
    notify_with_scroll(infoPtr, TB_PAGEUP);
}

unsafe extern "C" fn TRACKBAR_LineUp(infoPtr: *mut TRACKBAR_INFO) {
    if (*infoPtr).lPos == (*infoPtr).lRangeMin {
        return;
    }
    (*infoPtr).lPos -= (*infoPtr).lLineSize;
    if (*infoPtr).lPos < (*infoPtr).lRangeMin {
        (*infoPtr).lPos = (*infoPtr).lRangeMin;
    }
    notify_with_scroll(infoPtr, TB_LINEUP);
}

unsafe extern "C" fn TRACKBAR_LineDown(infoPtr: *mut TRACKBAR_INFO) {
    if (*infoPtr).lPos == (*infoPtr).lRangeMax {
        return;
    }
    (*infoPtr).lPos += (*infoPtr).lLineSize;
    if (*infoPtr).lPos > (*infoPtr).lRangeMax {
        (*infoPtr).lPos = (*infoPtr).lRangeMax;
    }
    notify_with_scroll(infoPtr, TB_LINEDOWN);
}

unsafe extern "C" fn TRACKBAR_CalcChannel(infoPtr: *mut TRACKBAR_INFO) {
    let mut cyChannel: i32;
    let mut offsetthumb: i32;
    let mut offsetedge: i32;
    let mut lpRect: RECT = core::mem::zeroed();
    let channel: *mut RECT = core::ptr::addr_of_mut!((*infoPtr).rcChannel);

    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(lpRect));

    offsetthumb = ((*infoPtr).uThumbLen / (4 as u32)) as i32;
    offsetedge = offsetthumb.wrapping_add(3 as i32);
    cyChannel = if ((*infoPtr).dwStyle & TBS_ENABLESELRANGE) != 0 {
        offsetthumb.wrapping_mul(3 as i32)
    } else {
        4 as i32
    };
    if ((*infoPtr).dwStyle & TBS_VERT) != 0 {
        (*channel).top = lpRect.top.wrapping_add(offsetedge);
        (*channel).bottom = lpRect.bottom.wrapping_sub(offsetedge);
        if ((*infoPtr).dwStyle & TBS_ENABLESELRANGE) != 0 {
            (*channel).left = lpRect.left.wrapping_add(
                (((*infoPtr).uThumbLen.wrapping_sub(cyChannel as u32))
                    .wrapping_add(2 as u32)
                    / (2 as u32)) as i32,
            );
        } else {
            (*channel).left = lpRect
                .left
                .wrapping_add(((*infoPtr).uThumbLen / (2 as u32)) as i32)
                .wrapping_sub(1 as i32);
        }
        if ((*infoPtr).dwStyle & TBS_BOTH) != 0 {
            if ((*infoPtr).dwStyle & TBS_NOTICKS) != 0 {
                (*channel).left = (*channel).left.wrapping_add(1 as i32);
            } else {
                (*channel).left = (*channel).left.wrapping_add(9 as i32);
            }
        } else if ((*infoPtr).dwStyle & TBS_TOP) != 0 {
            if ((*infoPtr).dwStyle & TBS_NOTICKS) != 0 {
                (*channel).left = (*channel).left.wrapping_add(2 as i32);
            } else {
                (*channel).left = (*channel).left.wrapping_add(10 as i32);
            }
        }
        (*channel).right = (*channel).left.wrapping_add(cyChannel);
    } else {
        (*channel).left = lpRect.left.wrapping_add(offsetedge);
        (*channel).right = lpRect.right.wrapping_sub(offsetedge);
        if ((*infoPtr).dwStyle & TBS_ENABLESELRANGE) != 0 {
            (*channel).top = lpRect.top.wrapping_add(
                (((*infoPtr).uThumbLen.wrapping_sub(cyChannel as u32))
                    .wrapping_add(2 as u32)
                    / (2 as u32)) as i32,
            );
        } else {
            (*channel).top = lpRect
                .top
                .wrapping_add(((*infoPtr).uThumbLen / (2 as u32)) as i32)
                .wrapping_sub(1 as i32);
        }
        if ((*infoPtr).dwStyle & TBS_BOTH) != 0 {
            if ((*infoPtr).dwStyle & TBS_NOTICKS) != 0 {
                (*channel).top = (*channel).top.wrapping_add(1 as i32);
            } else {
                (*channel).top = (*channel).top.wrapping_add(9 as i32);
            }
        } else if ((*infoPtr).dwStyle & TBS_TOP) != 0 {
            if ((*infoPtr).dwStyle & TBS_NOTICKS) != 0 {
                (*channel).top = (*channel).top.wrapping_add(2 as i32);
            } else {
                (*channel).top = (*channel).top.wrapping_add(10 as i32);
            }
        }
        (*channel).bottom = (*channel).top.wrapping_add(cyChannel);
    }
}

unsafe extern "C" fn TRACKBAR_CalcThumb(infoPtr: *const TRACKBAR_INFO, lPos: LONG, thumb: *mut RECT) {
    let mut range: i32;
    let mut width: i32;
    let mut height: i32;
    let thumbwidth: i32;
    let mut lpRect: RECT = core::mem::zeroed();

    range = (*infoPtr).lRangeMax - (*infoPtr).lRangeMin;
    thumbwidth = (((*infoPtr).uThumbLen / 2) | 1) as i32;

    if range == 0 {
        range = 1;
    }

    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(lpRect));
    if ((*infoPtr).dwStyle & TBS_VERT) != 0 {
        height = (*infoPtr).rcChannel.bottom - (*infoPtr).rcChannel.top - thumbwidth;

        if (((*infoPtr).dwStyle & (TBS_BOTH | TBS_LEFT)) != 0)
            && ((*infoPtr).dwStyle & TBS_NOTICKS) == 0
        {
            (*thumb).left = 10;
        } else {
            (*thumb).left = 2;
        }
        (*thumb).right = (*thumb).left + (*infoPtr).uThumbLen as i32;
        (*thumb).top = (*infoPtr).rcChannel.top
            + (height * (lPos - (*infoPtr).lRangeMin)) / range;
        (*thumb).bottom = (*thumb).top + thumbwidth;
    } else {
        width = (*infoPtr).rcChannel.right - (*infoPtr).rcChannel.left - thumbwidth;

        (*thumb).left = (*infoPtr).rcChannel.left
            + (width * (lPos - (*infoPtr).lRangeMin)) / range;
        (*thumb).right = (*thumb).left + thumbwidth;
        if (((*infoPtr).dwStyle & (TBS_BOTH | TBS_TOP)) != 0)
            && ((*infoPtr).dwStyle & TBS_NOTICKS) == 0
        {
            (*thumb).top = 10;
        } else {
            (*thumb).top = 2;
        }
        (*thumb).bottom = (*thumb).top + (*infoPtr).uThumbLen as i32;
    }
}

unsafe extern "C" fn TRACKBAR_UpdateThumb(infoPtr: *mut TRACKBAR_INFO) {
    TRACKBAR_CalcThumb(infoPtr, (*infoPtr).lPos, core::ptr::addr_of_mut!((*infoPtr).rcThumb));
}

unsafe extern "C" fn TRACKBAR_InvalidateAll(infoPtr: *const TRACKBAR_INFO) {
    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), FALSE);
}

unsafe extern "C" fn TRACKBAR_InvalidateThumb(infoPtr: *const TRACKBAR_INFO, thumbPos: LONG) {
    let mut rcThumb: RECT = core::mem::zeroed();

    TRACKBAR_CalcThumb(infoPtr, thumbPos, core::ptr::addr_of_mut!(rcThumb));
    InflateRect(core::ptr::addr_of_mut!(rcThumb), 1, 1);
    InvalidateRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcThumb), FALSE);
}

unsafe extern "C" fn TRACKBAR_InvalidateThumbMove(infoPtr: *const TRACKBAR_INFO, oldPos: LONG, newPos: LONG) {
    TRACKBAR_InvalidateThumb(infoPtr, oldPos);
    if newPos != oldPos {
        TRACKBAR_InvalidateThumb(infoPtr, newPos);
    }
}

unsafe extern "C" fn TRACKBAR_HasSelection(infoPtr: *const TRACKBAR_INFO) -> BOOL {
    return ((*infoPtr).lSelMin != (*infoPtr).lSelMax) as BOOL;
}

unsafe extern "C" fn TRACKBAR_CalcSelection(infoPtr: *mut TRACKBAR_INFO) {
    let selection: *mut RECT = core::ptr::addr_of_mut!((*infoPtr).rcSelection);
    let range: i32 = (*infoPtr).lRangeMax - (*infoPtr).lRangeMin;
    let offsetthumb: i32;
    let height: i32;
    let width: i32;

    if range <= 0 {
        SetRectEmpty(selection);
    } else {
        if (((*infoPtr).dwStyle & TBS_VERT) != 0) {
            offsetthumb = ((*infoPtr).rcThumb.bottom - (*infoPtr).rcThumb.top) / 2;
            height = (*infoPtr).rcChannel.bottom - (*infoPtr).rcChannel.top - offsetthumb * 2;
            (*selection).top = (*infoPtr).rcChannel.top + offsetthumb
                + (height * (*infoPtr).lSelMin) / range;
            (*selection).bottom = (*infoPtr).rcChannel.top + offsetthumb
                + (height * (*infoPtr).lSelMax) / range;
            (*selection).left = (*infoPtr).rcChannel.left + 3;
            (*selection).right = (*infoPtr).rcChannel.right - 3;
        } else {
            offsetthumb = ((*infoPtr).rcThumb.right - (*infoPtr).rcThumb.left) / 2;
            width = (*infoPtr).rcChannel.right - (*infoPtr).rcChannel.left - offsetthumb * 2;
            (*selection).left = (*infoPtr).rcChannel.left + offsetthumb
                + (width * (*infoPtr).lSelMin) / range;
            (*selection).right = (*infoPtr).rcChannel.left + offsetthumb
                + (width * (*infoPtr).lSelMax) / range;
            (*selection).top = (*infoPtr).rcChannel.top + 3;
            (*selection).bottom = (*infoPtr).rcChannel.bottom - 3;
        }
    }
}

unsafe extern "C" fn TRACKBAR_AutoPage(infoPtr: *mut TRACKBAR_INFO, clickPoint: POINT) -> BOOL {
    let dir: LONG = TRACKBAR_GetAutoPageDirection(infoPtr, clickPoint);
    let prevPos: LONG = (*infoPtr).lPos;

    if dir > 0 && ((*infoPtr).flags & TB_AUTO_PAGE_RIGHT) != 0 {
        TRACKBAR_PageDown(infoPtr);
    } else if dir < 0 && ((*infoPtr).flags & TB_AUTO_PAGE_LEFT) != 0 {
        TRACKBAR_PageUp(infoPtr);
    } else {
        return FALSE;
    }

    TRACKBAR_UpdateThumb(infoPtr);
    TRACKBAR_InvalidateThumbMove(infoPtr, prevPos, (*infoPtr).lPos);

    return TRUE;
}

unsafe extern "C" fn TRACKBAR_DrawChannel(infoPtr: *const TRACKBAR_INFO, hdc: HDC) {
    let mut rcChannel: RECT = (*infoPtr).rcChannel;
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);

    if !theme.is_null() {
        DrawThemeBackground(
            theme,
            hdc,
            if ((*infoPtr).dwStyle & TBS_VERT) != 0 {
                (TKP_TRACKVERT) as i32
            } else {
                (TKP_TRACK) as i32
            },
            (TKS_NORMAL) as i32,
            core::ptr::addr_of_mut!(rcChannel),
            core::ptr::null(),
        );
    } else {
        DrawEdge(
            hdc,
            core::ptr::addr_of_mut!(rcChannel),
            EDGE_SUNKEN,
            BF_RECT | BF_ADJUST,
        );
        if ((*infoPtr).dwStyle & TBS_ENABLESELRANGE) != 0 {
            // fill the channel
            FillRect(
                hdc,
                core::ptr::addr_of_mut!(rcChannel),
                (GetStockObject((WHITE_BRUSH as i32))).cast(),
            );
            if TRACKBAR_HasSelection(infoPtr) != 0 {
                FillRect(
                    hdc,
                    core::ptr::addr_of_mut!((*(infoPtr as *mut TRACKBAR_INFO)).rcSelection),
                    GetSysColorBrush((COLOR_HIGHLIGHT) as i32),
                );
            }
        }
    }
}

unsafe extern "C" fn TRACKBAR_DrawOneTic(infoPtr: *const TRACKBAR_INFO, hdc: HDC, ticPos: LONG, flags: i32) {
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut ox: i32 = 0;
    let mut oy: i32 = 0;
    let mut range: i32 = 0;
    let mut side: i32 = 0;
    let mut indent: i32 = 0;
    let mut len: i32 = 3;
    let mut offsetthumb: i32;
    let mut rcTics: RECT = core::mem::zeroed();

    if (flags & (TBS_VERT as i32)) != 0 {
        offsetthumb = ((*infoPtr).rcThumb.bottom - (*infoPtr).rcThumb.top) / 2;
        SetRect(core::ptr::addr_of_mut!(rcTics), (*infoPtr).rcThumb.left - 2, (*infoPtr).rcChannel.top + offsetthumb,
                (*infoPtr).rcThumb.right + 2, (*infoPtr).rcChannel.bottom - offsetthumb - 1);
    } else {
        offsetthumb = ((*infoPtr).rcThumb.right - (*infoPtr).rcThumb.left) / 2;
        SetRect(core::ptr::addr_of_mut!(rcTics), (*infoPtr).rcChannel.left + offsetthumb, (*infoPtr).rcThumb.top - 2,
                (*infoPtr).rcChannel.right - offsetthumb - 1, (*infoPtr).rcThumb.bottom + 2);
    }

    if (flags & ((TBS_TOP | TBS_LEFT) as i32)) != 0 {
        x = rcTics.left;
        y = rcTics.top;
        side = -1;
    } else {
        x = rcTics.right;
        y = rcTics.bottom;
        side = 1;
    }

    range = (*infoPtr).lRangeMax - (*infoPtr).lRangeMin;
    if range <= 0 {
        range = 1; /* to avoid division by zero */
    }

    if (flags & (TIC_SELECTIONMARK as i32)) != 0 {
        indent = if (flags & (TIC_SELECTIONMARKMIN as i32)) != 0 { -1 } else { 1 };
    } else if (flags & (TIC_EDGE as i32)) != 0 {
        len += 1;
    }

    if (flags & (TBS_VERT as i32)) != 0 {
        let height: i32 = rcTics.bottom - rcTics.top;
        y = rcTics.top + (height * (ticPos - (*infoPtr).lRangeMin)) / range;
    } else {
        let width: i32 = rcTics.right - rcTics.left;
        x = rcTics.left + (width * (ticPos - (*infoPtr).lRangeMin)) / range;
    }

    ox = x;
    oy = y;
    MoveToEx(hdc, x, y, core::ptr::null_mut());
    if (flags & (TBS_VERT as i32)) != 0 {
        x += len * side;
    } else {
        y += len * side;
    }
    LineTo(hdc, x, y);

    if (flags & (TIC_SELECTIONMARK as i32)) != 0 {
        if (flags & (TBS_VERT as i32)) != 0 {
            x -= side;
        } else {
            y -= side;
        }
        MoveToEx(hdc, x, y, core::ptr::null_mut());
        if (flags & (TBS_VERT as i32)) != 0 {
            y += 2 * indent;
        } else {
            x += 2 * indent;
        }

        LineTo(hdc, x, y);
        LineTo(hdc, ox, oy);
    }
}

unsafe extern "C" fn TRACKBAR_DrawTic(infoPtr: *const TRACKBAR_INFO, hdc: HDC, ticPos: LONG, flags: i32) {
    if (flags & ((TBS_LEFT | TBS_TOP) as i32)) != 0 || (flags & (TBS_BOTH as i32)) != 0 {
        TRACKBAR_DrawOneTic(infoPtr, hdc, ticPos, flags | (TBS_LEFT as i32));
    }

    if !((flags & ((TBS_LEFT | TBS_TOP) as i32)) != 0) || (flags & (TBS_BOTH as i32)) != 0 {
        TRACKBAR_DrawOneTic(infoPtr, hdc, ticPos, flags & (!TBS_LEFT) as i32);
    }
}

unsafe extern "C" fn TRACKBAR_DrawTics(infoPtr: *const TRACKBAR_INFO, hdc: HDC) {
    let mut i: u32;
    let ticFlags: i32 = ((*infoPtr).dwStyle & 0x0f) as i32;
    let mut ticPen: LOGPEN = LOGPEN {
        lopnStyle: PS_SOLID,
        lopnWidth: POINT {
            x: 1,
            y: 0,
        },
        lopnColor: GetSysColor((COLOR_3DDKSHADOW) as i32),
    };
    let hOldPen: HPEN;
    let hTicPen: HPEN;
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);

    if !theme.is_null() {
        let part: i32 = if ((*infoPtr).dwStyle & TBS_VERT) != 0 {
            (TKP_TICSVERT) as i32
        } else {
            (TKP_TICS) as i32
        };
        GetThemeColor(
            theme,
            part,
            (TSS_NORMAL) as i32,
            (TMT_COLOR as i32),
            core::ptr::addr_of_mut!(ticPen.lopnColor),
        );
    }
    /* create the pen to draw the tics with */
    hTicPen = CreatePenIndirect(core::ptr::addr_of_mut!(ticPen));
    hOldPen = if !hTicPen.is_null() {
        (SelectObject(hdc, (hTicPen).cast())).cast()
    } else {
        core::ptr::null_mut()
    };

    /* actually draw the tics */
    i = 0;
    while i < (*infoPtr).uNumTics {
        TRACKBAR_DrawTic(
            infoPtr,
            hdc,
            *(*infoPtr).tics.add(i as usize),
            ticFlags,
        );
        i += 1;
    }

    TRACKBAR_DrawTic(
        infoPtr,
        hdc,
        (*infoPtr).lRangeMin,
        ticFlags | (TIC_EDGE as i32),
    );
    TRACKBAR_DrawTic(
        infoPtr,
        hdc,
        (*infoPtr).lRangeMax,
        ticFlags | (TIC_EDGE as i32),
    );

    if ((*infoPtr).dwStyle & TBS_ENABLESELRANGE) != 0 && TRACKBAR_HasSelection(infoPtr) != 0 {
        TRACKBAR_DrawTic(
            infoPtr,
            hdc,
            (*infoPtr).lSelMin,
            ticFlags | (TIC_SELECTIONMARKMIN as i32),
        );
        TRACKBAR_DrawTic(
            infoPtr,
            hdc,
            (*infoPtr).lSelMax,
            ticFlags | (TIC_SELECTIONMARKMAX as i32),
        );
    }

    /* clean up the pen, if we created one */
    if !hTicPen.is_null() {
        SelectObject(hdc, (hOldPen).cast());
        DeleteObject((hTicPen).cast());
    }
}

unsafe extern "C" fn TRACKBAR_FillThumb(infoPtr: *const TRACKBAR_INFO, hdc: HDC, hbrush: HBRUSH) -> i32 {
    let thumb: *const RECT = core::ptr::addr_of!((*infoPtr).rcThumb);
    let mut points: [POINT; 6] = core::mem::zeroed();
    let mut PointDepth: i32 = 0;
    let mut oldbr: HBRUSH;

    if ((*infoPtr).dwStyle & TBS_BOTH) != 0 {
        FillRect(hdc, thumb, hbrush);
        return 0;
    }

    if ((*infoPtr).dwStyle & TBS_VERT) != 0 {
        PointDepth = ((*thumb).bottom - (*thumb).top) / 2;
        if ((*infoPtr).dwStyle & TBS_LEFT) != 0 {
            points[0].x = (*thumb).right - 1;
            points[0].y = (*thumb).top;
            points[1].x = (*thumb).right - 1;
            points[1].y = (*thumb).bottom - 1;
            points[2].x = (*thumb).left + PointDepth;
            points[2].y = (*thumb).bottom - 1;
            points[3].x = (*thumb).left;
            points[3].y = (*thumb).top + PointDepth;
            points[4].x = (*thumb).left + PointDepth;
            points[4].y = (*thumb).top;
            points[5].x = points[0].x;
            points[5].y = points[0].y;
        } else {
            points[0].x = (*thumb).right;
            points[0].y = (*thumb).top + PointDepth;
            points[1].x = (*thumb).right - PointDepth;
            points[1].y = (*thumb).bottom - 1;
            points[2].x = (*thumb).left;
            points[2].y = (*thumb).bottom - 1;
            points[3].x = (*thumb).left;
            points[3].y = (*thumb).top;
            points[4].x = (*thumb).right - PointDepth;
            points[4].y = (*thumb).top;
            points[5].x = points[0].x;
            points[5].y = points[0].y;
        }
    } else {
        PointDepth = ((*thumb).right - (*thumb).left) / 2;
        if ((*infoPtr).dwStyle & TBS_TOP) != 0 {
            points[0].x = (*thumb).left + PointDepth;
            points[0].y = (*thumb).top + 1;
            points[1].x = (*thumb).right - 1;
            points[1].y = (*thumb).top + PointDepth + 1;
            points[2].x = (*thumb).right - 1;
            points[2].y = (*thumb).bottom - 1;
            points[3].x = (*thumb).left;
            points[3].y = (*thumb).bottom - 1;
            points[4].x = (*thumb).left;
            points[4].y = (*thumb).top + PointDepth + 1;
            points[5].x = points[0].x;
            points[5].y = points[0].y;
        } else {
            points[0].x = (*thumb).right - 1;
            points[0].y = (*thumb).top;
            points[1].x = (*thumb).right - 1;
            points[1].y = (*thumb).bottom - PointDepth - 1;
            points[2].x = (*thumb).left + PointDepth;
            points[2].y = (*thumb).bottom - 1;
            points[3].x = (*thumb).left;
            points[3].y = (*thumb).bottom - PointDepth - 1;
            points[4].x = (*thumb).left;
            points[4].y = (*thumb).top;
            points[5].x = points[0].x;
            points[5].y = points[0].y;
        }
    }

    oldbr = (SelectObject(hdc, (hbrush).cast())).cast();
    SetPolyFillMode(hdc, (WINDING as i32));
    Polygon(hdc, points.as_mut_ptr(), (ARRAY_SIZE!(points)) as i32);
    SelectObject(hdc, (oldbr).cast());

    return PointDepth;
}

unsafe extern "C" fn TRACKBAR_DrawThumb(infoPtr: *mut TRACKBAR_INFO, hdc: HDC) {
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    let PointDepth: i32;
    let brush: HBRUSH;

    if !theme.is_null() {
        let partId: i32;
        let stateId: i32;
        if ((*infoPtr).dwStyle & TBS_BOTH) != 0 {
            partId = if ((*infoPtr).dwStyle & TBS_VERT) != 0 { (TKP_THUMBVERT) as i32 } else { (TKP_THUMB) as i32 };
        } else if ((*infoPtr).dwStyle & TBS_LEFT) != 0 {
            partId = if ((*infoPtr).dwStyle & TBS_VERT) != 0 { (TKP_THUMBLEFT) as i32 } else { (TKP_THUMBTOP) as i32 };
        } else {
            partId = if ((*infoPtr).dwStyle & TBS_VERT) != 0 { (TKP_THUMBRIGHT) as i32 } else { (TKP_THUMBBOTTOM) as i32 };
        }

        if ((*infoPtr).dwStyle & WS_DISABLED) != 0 {
            stateId = (TUS_DISABLED) as i32;
        } else if ((*infoPtr).flags & TB_DRAG_MODE) != 0 {
            stateId = (TUS_PRESSED) as i32;
        } else if ((*infoPtr).flags & TB_THUMB_HOT) != 0 {
            stateId = (TUS_HOT) as i32;
        } else {
            stateId = (TUS_NORMAL) as i32;
        }

        DrawThemeBackground(theme, hdc, partId, stateId, core::ptr::addr_of_mut!((*infoPtr).rcThumb), core::ptr::null_mut());

        return;
    }

    if ((*infoPtr).dwStyle & WS_DISABLED) != 0 || ((*infoPtr).flags & TB_DRAG_MODE) != 0 {
        if comctl32_color.clr3dHilight == comctl32_color.clrWindow {
            brush = COMCTL32_hPattern55AABrush;
        } else {
            brush = GetSysColorBrush((COLOR_SCROLLBAR) as i32);
        }

        SetTextColor(hdc, comctl32_color.clr3dFace);
        SetBkColor(hdc, comctl32_color.clr3dHilight);
    } else {
        brush = GetSysColorBrush((COLOR_BTNFACE) as i32);
    }

    PointDepth = TRACKBAR_FillThumb(infoPtr, hdc, brush);

    if ((*infoPtr).dwStyle & TBS_BOTH) != 0 {
        DrawEdge(hdc, core::ptr::addr_of_mut!((*infoPtr).rcThumb), EDGE_RAISED, BF_RECT | BF_SOFT);
        return;
    } else {
        let mut thumb: RECT = (*infoPtr).rcThumb;

        if ((*infoPtr).dwStyle & TBS_VERT) != 0 {
            if ((*infoPtr).dwStyle & TBS_LEFT) != 0 {
                /* rectangular part */
                thumb.left += PointDepth;
                DrawEdge(hdc, core::ptr::addr_of_mut!(thumb), EDGE_RAISED, BF_TOP | BF_RIGHT | BF_BOTTOM | BF_SOFT);

                /* light edge */
                thumb.left -= PointDepth;
                thumb.right = thumb.left + PointDepth;
                thumb.bottom = (*infoPtr).rcThumb.top + PointDepth + 1;
                thumb.top = (*infoPtr).rcThumb.top;
                DrawEdge(hdc, core::ptr::addr_of_mut!(thumb), EDGE_RAISED, BF_DIAGONAL_ENDTOPRIGHT | BF_SOFT);

                /* shadowed edge */
                thumb.top += PointDepth;
                thumb.bottom += PointDepth;
                DrawEdge(hdc, core::ptr::addr_of_mut!(thumb), EDGE_SUNKEN, BF_DIAGONAL_ENDTOPLEFT | BF_SOFT);
                return;
            } else {
                /* rectangular part */
                thumb.right -= PointDepth;
                DrawEdge(hdc, core::ptr::addr_of_mut!(thumb), EDGE_RAISED, BF_TOP | BF_LEFT | BF_BOTTOM | BF_SOFT);

                /* light edge */
                thumb.left = thumb.right;
                thumb.right += PointDepth + 1;
                thumb.bottom = (*infoPtr).rcThumb.top + PointDepth + 1;
                thumb.top = (*infoPtr).rcThumb.top;
                DrawEdge(hdc, core::ptr::addr_of_mut!(thumb), EDGE_RAISED, BF_DIAGONAL_ENDTOPLEFT | BF_SOFT);

                /* shadowed edge */
                thumb.top += PointDepth;
                thumb.bottom += PointDepth;
                DrawEdge(hdc, core::ptr::addr_of_mut!(thumb), EDGE_RAISED, BF_DIAGONAL_ENDBOTTOMLEFT | BF_SOFT);
            }
        } else {
            if ((*infoPtr).dwStyle & TBS_TOP) != 0 {
                /* rectangular part */
                thumb.top += PointDepth;
                DrawEdge(hdc, core::ptr::addr_of_mut!(thumb), EDGE_RAISED, BF_LEFT | BF_BOTTOM | BF_RIGHT | BF_SOFT);

                /* light edge */
                thumb.left = (*infoPtr).rcThumb.left;
                thumb.right = thumb.left + PointDepth;
                thumb.bottom = (*infoPtr).rcThumb.top + PointDepth + 1;
                thumb.top -= PointDepth;
                DrawEdge(hdc, core::ptr::addr_of_mut!(thumb), EDGE_RAISED, BF_DIAGONAL_ENDTOPRIGHT | BF_SOFT);

                /* shadowed edge */
                thumb.left += PointDepth;
                thumb.right += PointDepth;
                DrawEdge(hdc, core::ptr::addr_of_mut!(thumb), EDGE_RAISED, BF_DIAGONAL_ENDBOTTOMRIGHT | BF_SOFT);
            } else {
                /* rectangular part */
                thumb.bottom -= PointDepth;
                DrawEdge(hdc, core::ptr::addr_of_mut!(thumb), EDGE_RAISED, BF_LEFT | BF_TOP | BF_RIGHT | BF_SOFT);

                /* light edge */
                thumb.left = (*infoPtr).rcThumb.left;
                thumb.right = thumb.left + PointDepth;
                thumb.top = (*infoPtr).rcThumb.bottom - PointDepth - 1;
                thumb.bottom += PointDepth;
                DrawEdge(hdc, core::ptr::addr_of_mut!(thumb), EDGE_RAISED, BF_DIAGONAL_ENDTOPLEFT | BF_SOFT);

                /* shadowed edge */
                thumb.left += PointDepth;
                thumb.right += PointDepth;
                DrawEdge(hdc, core::ptr::addr_of_mut!(thumb), EDGE_RAISED, BF_DIAGONAL_ENDBOTTOMLEFT | BF_SOFT);
            }
        }
    }
}

unsafe extern "C" fn TRACKBAR_ActivateToolTip(infoPtr: *const TRACKBAR_INFO, fShow: BOOL) {
    let mut ti = core::mem::MaybeUninit::<TTTOOLINFOW>::uninit();
    let ti: *mut TTTOOLINFOW = ti.as_mut_ptr();

    if (*infoPtr).hwndToolTip.is_null() {
        return;
    }

    ZeroMemory!(ti as *mut core::ffi::c_void, core::mem::size_of::<TTTOOLINFOW>());
    (*ti).cbSize = core::mem::size_of::<TTTOOLINFOW>() as UINT;
    (*ti).hwnd = (*infoPtr).hwndSelf;

    SendMessageW((*infoPtr).hwndToolTip, TTM_TRACKACTIVATE, (fShow) as WPARAM, ti as LPARAM);
}

unsafe extern "C" fn TRACKBAR_UpdateToolTip(infoPtr: *const TRACKBAR_INFO) {
    let mut buf = [0u16; 80];
    static fmt: [u16; 4] = [0x0025u16, 0x006C, 0x0064, 0x0000];
    let mut ti: TTTOOLINFOW = core::mem::zeroed();
    let mut pt: POINT = core::mem::zeroed();
    let mut rcClient: RECT = core::mem::zeroed();
    let mut size: LRESULT;

    if (*infoPtr).hwndToolTip.is_null() {
        return;
    }

    ZeroMemory!(core::ptr::addr_of_mut!(ti) as *mut _, core::mem::size_of::<TTTOOLINFOW>());
    ti.cbSize = core::mem::size_of::<TTTOOLINFOW>() as u32;
    ti.hwnd = (*infoPtr).hwndSelf;
    ti.uFlags = TTF_IDISHWND | TTF_TRACK | TTF_ABSOLUTE;

    wsprintfW(buf.as_mut_ptr(), fmt.as_ptr(), (*infoPtr).lPos);
    ti.lpszText = buf.as_mut_ptr();
    SendMessageW((*infoPtr).hwndToolTip, TTM_UPDATETIPTEXTW, 0, core::ptr::addr_of_mut!(ti) as LPARAM);

    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcClient));
    size = SendMessageW((*infoPtr).hwndToolTip, TTM_GETBUBBLESIZE, 0, core::ptr::addr_of_mut!(ti) as LPARAM);
    if ((*infoPtr).dwStyle & TBS_VERT) != 0 {
        if (*infoPtr).fLocation == (TBTS_LEFT as i32) {
            pt.x = 0 - (LOWORD!(size) as i32) - TOOLTIP_OFFSET;
        } else {
            pt.x = rcClient.right + TOOLTIP_OFFSET;
        }
        pt.y = ((*infoPtr).rcThumb.top + (*infoPtr).rcThumb.bottom - (HIWORD!(size) as i32)) / 2;
    } else {
        if (*infoPtr).fLocation == (TBTS_TOP as i32) {
            pt.y = 0 - (HIWORD!(size) as i32) - TOOLTIP_OFFSET;
        } else {
            pt.y = rcClient.bottom + TOOLTIP_OFFSET;
        }
        pt.x = ((*infoPtr).rcThumb.left + (*infoPtr).rcThumb.right - (LOWORD!(size) as i32)) / 2;
    }
    ClientToScreen((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(pt));

    SendMessageW((*infoPtr).hwndToolTip, TTM_TRACKPOSITION,
        0, MAKELPARAM!(pt.x as u16, pt.y as u16));
}

unsafe extern "C" fn TRACKBAR_Refresh(infoPtr: *mut TRACKBAR_INFO, hdcDst: HDC) {
    let mut rcClient: RECT = core::mem::zeroed();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hOldBmp: HBITMAP = core::ptr::null_mut();
    let mut hOffScreenBmp: HBITMAP = core::ptr::null_mut();
    let mut nmcd: NMCUSTOMDRAW = core::mem::zeroed();
    let mut gcdrf: i32 = 0;
    let mut icdrf: i32 = 0;

    if ((*infoPtr).flags & TB_THUMBCHANGED) != 0 {
        TRACKBAR_UpdateThumb(infoPtr);
        if ((*infoPtr).flags & TB_THUMBSIZECHANGED) != 0 {
            TRACKBAR_CalcChannel(infoPtr);
        }
    }
    if ((*infoPtr).flags & TB_SELECTIONCHANGED) != 0 {
        TRACKBAR_CalcSelection(infoPtr);
    }

    if ((*infoPtr).flags & TB_DRAG_MODE) != 0 {
        TRACKBAR_UpdateToolTip(infoPtr);
    }

    (*infoPtr).flags &= !(TB_THUMBCHANGED | TB_SELECTIONCHANGED);

    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcClient));

    /* try to render offscreen, if we fail, carrry onscreen */
    hdc = CreateCompatibleDC(hdcDst);
    if !hdc.is_null() {
        hOffScreenBmp = CreateCompatibleBitmap(hdcDst, rcClient.right, rcClient.bottom);
        if !hOffScreenBmp.is_null() {
            hOldBmp = (SelectObject(hdc, (hOffScreenBmp).cast())).cast();
        } else {
            DeleteObject((hdc).cast());
            hdc = hdcDst;
        }
    } else {
        hdc = hdcDst;
    }

    ZeroMemory!(core::ptr::addr_of_mut!(nmcd) as *mut _, core::mem::size_of::<NMCUSTOMDRAW>());
    nmcd.hdr.hwndFrom = (*infoPtr).hwndSelf;
    nmcd.hdr.idFrom = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_ID) as UINT_PTR;
    nmcd.hdr.code = NM_CUSTOMDRAW;
    nmcd.hdc = hdc;

    /* start the paint cycle */
    nmcd.rc = rcClient;
    gcdrf = notify_customdraw(infoPtr, core::ptr::addr_of_mut!(nmcd), (CDDS_PREPAINT as i32));
    'cleanup: {
        if (gcdrf & (CDRF_SKIPDEFAULT as i32)) != 0 {
            break 'cleanup;
        }

        /* Erase background */
        if gcdrf == (CDRF_DODEFAULT as i32)
            || notify_customdraw(infoPtr, core::ptr::addr_of_mut!(nmcd), (CDDS_PREERASE as i32)) != (CDRF_SKIPDEFAULT as i32)
        {
            if GetWindowTheme((*infoPtr).hwndSelf) != core::ptr::null_mut() {
                DrawThemeParentBackground((*infoPtr).hwndSelf, hdc, core::ptr::null_mut());
            }
            {
                let brush: HBRUSH = SendMessageW(
                    (*infoPtr).hwndNotify,
                    WM_CTLCOLORSTATIC,
                    hdc as WPARAM,
                    (*infoPtr).hwndSelf as LPARAM,
                ) as HBRUSH;
                FillRect(
                    hdc,
                    core::ptr::addr_of_mut!(rcClient),
                    if !brush.is_null() { brush } else { GetSysColorBrush((COLOR_BTNFACE) as i32) },
                );
            }
            if gcdrf != (CDRF_DODEFAULT as i32) {
                notify_customdraw(infoPtr, core::ptr::addr_of_mut!(nmcd), (CDDS_POSTERASE as i32));
            }
        }

        /* draw channel */
        if (gcdrf & (CDRF_NOTIFYITEMDRAW as i32)) != 0 {
            nmcd.dwItemSpec = (TBCD_CHANNEL as ULONG_PTR);
            nmcd.uItemState = CDIS_DEFAULT;
            nmcd.rc = (*infoPtr).rcChannel;
            icdrf = notify_customdraw(infoPtr, core::ptr::addr_of_mut!(nmcd), (CDDS_ITEMPREPAINT as i32));
        } else {
            icdrf = (CDRF_DODEFAULT as i32);
        }
        if (icdrf & (CDRF_SKIPDEFAULT as i32)) == 0 {
            TRACKBAR_DrawChannel(infoPtr, hdc);
            if (icdrf & (CDRF_NOTIFYPOSTPAINT as i32)) != 0 {
                notify_customdraw(infoPtr, core::ptr::addr_of_mut!(nmcd), (CDDS_ITEMPOSTPAINT as i32));
            }
        }

        /* draw tics */
        if ((*infoPtr).dwStyle & TBS_NOTICKS) == 0 {
            if (gcdrf & (CDRF_NOTIFYITEMDRAW as i32)) != 0 {
                nmcd.dwItemSpec = (TBCD_TICS as ULONG_PTR);
                nmcd.uItemState = CDIS_DEFAULT;
                nmcd.rc = rcClient;
                icdrf = notify_customdraw(infoPtr, core::ptr::addr_of_mut!(nmcd), (CDDS_ITEMPREPAINT as i32));
            } else {
                icdrf = (CDRF_DODEFAULT as i32);
            }
            if (icdrf & (CDRF_SKIPDEFAULT as i32)) == 0 {
                TRACKBAR_DrawTics(infoPtr, hdc);
                if (icdrf & (CDRF_NOTIFYPOSTPAINT as i32)) != 0 {
                    notify_customdraw(infoPtr, core::ptr::addr_of_mut!(nmcd), (CDDS_ITEMPOSTPAINT as i32));
                }
            }
        }

        /* draw thumb */
        if ((*infoPtr).dwStyle & TBS_NOTHUMB) == 0 {
            if (gcdrf & (CDRF_NOTIFYITEMDRAW as i32)) != 0 {
                nmcd.dwItemSpec = (TBCD_THUMB as ULONG_PTR);
                nmcd.uItemState = if ((*infoPtr).flags & TB_DRAG_MODE) != 0 {
                    CDIS_HOT
                } else {
                    CDIS_DEFAULT
                };
                nmcd.rc = (*infoPtr).rcThumb;
                icdrf = notify_customdraw(infoPtr, core::ptr::addr_of_mut!(nmcd), (CDDS_ITEMPREPAINT as i32));
            } else {
                icdrf = (CDRF_DODEFAULT as i32);
            }
            if (icdrf & (CDRF_SKIPDEFAULT as i32)) == 0 {
                TRACKBAR_DrawThumb(infoPtr, hdc);
                if (icdrf & (CDRF_NOTIFYPOSTPAINT as i32)) != 0 {
                    notify_customdraw(infoPtr, core::ptr::addr_of_mut!(nmcd), (CDDS_ITEMPOSTPAINT as i32));
                }
            }
        }

        /* draw focus rectangle */
        if ((*infoPtr).flags & TB_IS_FOCUSED) != 0 {
            DrawFocusRect(hdc, core::ptr::addr_of_mut!(rcClient));
        }

        /* finish up the painting */
        if (gcdrf & (CDRF_NOTIFYPOSTPAINT as i32)) != 0 {
            notify_customdraw(infoPtr, core::ptr::addr_of_mut!(nmcd), (CDDS_POSTPAINT as i32));
        }
    }

    /* cleanup, if we rendered offscreen */
    if hdc != hdcDst {
        BitBlt(
            hdcDst,
            0,
            0,
            rcClient.right,
            rcClient.bottom,
            hdc,
            0,
            0,
            SRCCOPY,
        );
        SelectObject(hdc, (hOldBmp).cast());
        DeleteObject((hOffScreenBmp).cast());
        DeleteObject((hdc).cast());
    }
}

unsafe extern "C" fn TRACKBAR_AlignBuddies(infoPtr: *const TRACKBAR_INFO) {
    let hwndParent: HWND = GetParent((*infoPtr).hwndSelf);
    let mut rcSelf: RECT = core::mem::zeroed();
    let mut rcBuddy: RECT = core::mem::zeroed();
    let mut x: INT = 0;
    let mut y: INT = 0;

    GetWindowRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rcSelf));
    MapWindowPoints(HWND_DESKTOP, hwndParent, core::ptr::addr_of_mut!(rcSelf) as LPPOINT, 2);

    /* align buddy left or above */
    if (*infoPtr).hwndBuddyLA as usize != 0 {
        GetWindowRect((*infoPtr).hwndBuddyLA, core::ptr::addr_of_mut!(rcBuddy));
        MapWindowPoints(HWND_DESKTOP, hwndParent, core::ptr::addr_of_mut!(rcBuddy) as LPPOINT, 2);

        if (*infoPtr).dwStyle & TBS_VERT != 0 {
            x = ((*infoPtr).rcChannel.right + (*infoPtr).rcChannel.left) / 2 -
                (rcBuddy.right - rcBuddy.left) / 2 + rcSelf.left;
            y = rcSelf.top - (rcBuddy.bottom - rcBuddy.top);
        } else {
            x = rcSelf.left - (rcBuddy.right - rcBuddy.left);
            y = ((*infoPtr).rcChannel.bottom + (*infoPtr).rcChannel.top) / 2 -
                (rcBuddy.bottom - rcBuddy.top) / 2 + rcSelf.top;
        }

        SetWindowPos((*infoPtr).hwndBuddyLA, core::ptr::null_mut(), x, y, 0, 0,
            SWP_NOZORDER | SWP_NOSIZE);
    }

    /* align buddy right or below */
    if (*infoPtr).hwndBuddyRB as usize != 0 {
        GetWindowRect((*infoPtr).hwndBuddyRB, core::ptr::addr_of_mut!(rcBuddy));
        MapWindowPoints(HWND_DESKTOP, hwndParent, core::ptr::addr_of_mut!(rcBuddy) as LPPOINT, 2);

        if (*infoPtr).dwStyle & TBS_VERT != 0 {
            x = ((*infoPtr).rcChannel.right + (*infoPtr).rcChannel.left) / 2 -
                (rcBuddy.right - rcBuddy.left) / 2 + rcSelf.left;
            y = rcSelf.bottom;
        } else {
            x = rcSelf.right;
            y = ((*infoPtr).rcChannel.bottom + (*infoPtr).rcChannel.top) / 2 -
                (rcBuddy.bottom - rcBuddy.top) / 2 + rcSelf.top;
        }
        SetWindowPos((*infoPtr).hwndBuddyRB, core::ptr::null_mut(), x, y, 0, 0,
            SWP_NOZORDER | SWP_NOSIZE);
    }
}

unsafe extern "C" fn TRACKBAR_ClearSel(infoPtr: *mut TRACKBAR_INFO, fRedraw: BOOL) -> LRESULT {
    (*infoPtr).lSelMin = 0;
    (*infoPtr).lSelMax = 0;
    (*infoPtr).flags |= TB_SELECTIONCHANGED;

    if fRedraw != 0 {
        TRACKBAR_InvalidateAll(infoPtr);
    }

    return 0;
}

unsafe extern "C" fn TRACKBAR_ClearTics(infoPtr: *mut TRACKBAR_INFO, fRedraw: BOOL) -> LRESULT {
    if !(*infoPtr).tics.is_null() {
        Free((*infoPtr).tics as *mut core::ffi::c_void);
        (*infoPtr).tics = core::ptr::null_mut();
        (*infoPtr).uNumTics = 0;
    }

    if fRedraw != 0 {
        TRACKBAR_InvalidateAll(infoPtr);
    }

    return 0;
}

unsafe extern "C" fn TRACKBAR_GetChannelRect(infoPtr: *const TRACKBAR_INFO, lprc: LPRECT) -> LRESULT {
    if lprc.is_null() { return 0; }

    (*lprc).left   = (*infoPtr).rcChannel.left;
    (*lprc).right  = (*infoPtr).rcChannel.right;
    (*lprc).bottom = (*infoPtr).rcChannel.bottom;
    (*lprc).top    = (*infoPtr).rcChannel.top;

    return 0;
}

unsafe extern "C" fn TRACKBAR_GetNumTics(infoPtr: *const TRACKBAR_INFO) -> LONG {
    if ((*infoPtr).dwStyle & TBS_NOTICKS) != 0 {
        return 0;
    }

    return ((*infoPtr).uNumTics + 2) as LONG;
}

unsafe extern "C" fn comp_tics(ap: *const core::ffi::c_void, bp: *const core::ffi::c_void) -> i32 {
    let a: DWORD = *(ap as *const DWORD);
    let b: DWORD = *(bp as *const DWORD);

    if a < b { return -1; }
    if a > b { return 1; }
    return 0;
}

unsafe extern "C" fn TRACKBAR_GetTic(infoPtr: *const TRACKBAR_INFO, iTic: INT) -> LONG {
    if (iTic < 0) || ((iTic as u32) >= (*infoPtr).uNumTics) || (*infoPtr).tics.is_null() {
        return -1;
    }

    qsort(
        (*infoPtr).tics as *mut core::ffi::c_void,
        (*infoPtr).uNumTics as usize,
        core::mem::size_of::<DWORD>(),
        Some(comp_tics),
    );
    return *(*infoPtr).tics.offset(iTic as isize);
}

unsafe extern "C" fn TRACKBAR_GetTicPos(infoPtr: *const TRACKBAR_INFO, iTic: INT) -> LONG {
    let mut range: LONG;
    let mut width: LONG;
    let mut pos: LONG;
    let mut tic: LONG;
    let mut offsetthumb: i32;

    if (iTic < 0) || ((iTic as UINT) >= (*infoPtr).uNumTics) || (*infoPtr).tics.is_null() {
        return -1;
    }

    tic = TRACKBAR_GetTic(infoPtr, iTic);
    range = (*infoPtr).lRangeMax - (*infoPtr).lRangeMin;
    if range <= 0 {
        range = 1;
    }
    offsetthumb = ((*infoPtr).rcThumb.right - (*infoPtr).rcThumb.left) / 2;
    width = (*infoPtr).rcChannel.right - (*infoPtr).rcChannel.left - offsetthumb * 2;
    pos = (*infoPtr).rcChannel.left + offsetthumb + (width * tic) / range;

    return pos;
}

unsafe extern "C" fn TRACKBAR_SetBuddy(infoPtr: *mut TRACKBAR_INFO, fLocation: BOOL, hwndBuddy: HWND) -> HWND {
    let mut hwndTemp: HWND;

    if fLocation != 0 {
        /* buddy is left or above */
        hwndTemp = (*infoPtr).hwndBuddyLA;
        (*infoPtr).hwndBuddyLA = hwndBuddy;
    }
    else {
        /* buddy is right or below */
        hwndTemp = (*infoPtr).hwndBuddyRB;
        (*infoPtr).hwndBuddyRB = hwndBuddy;
    }

    TRACKBAR_AlignBuddies(infoPtr);

    return hwndTemp;
}

unsafe extern "C" fn TRACKBAR_SetLineSize(infoPtr: *mut TRACKBAR_INFO, lLineSize: LONG) -> LONG {
    let mut lTemp: LONG = (*infoPtr).lLineSize;

    (*infoPtr).lLineSize = lLineSize;

    return lTemp;
}

unsafe extern "C" fn TRACKBAR_UpdatePageSize(infoPtr: *mut TRACKBAR_INFO) {
    if (*infoPtr).flags & TB_USER_PAGE != 0 {
        return;
    }

    (*infoPtr).lPageSize = ((*infoPtr).lRangeMax - (*infoPtr).lRangeMin) / 5;
    if (*infoPtr).lPageSize == 0 {
        (*infoPtr).lPageSize = 1;
    }
}

unsafe extern "C" fn TRACKBAR_SetPageSize(infoPtr: *mut TRACKBAR_INFO, lPageSize: LONG) -> LONG {
    let lTemp: LONG = (*infoPtr).lPageSize;

    if lPageSize == -1 {
        (*infoPtr).flags &= !TB_USER_PAGE;
        TRACKBAR_UpdatePageSize(infoPtr);
    } else {
        (*infoPtr).flags |= TB_USER_PAGE;
        (*infoPtr).lPageSize = lPageSize;
    }

    return lTemp;
}

unsafe extern "C" fn TRACKBAR_SetPos(infoPtr: *mut TRACKBAR_INFO, fPosition: BOOL, lPosition: LONG) -> LRESULT {
    let mut oldPos: LONG = (*infoPtr).lPos;
    (*infoPtr).lPos = lPosition;

    if (*infoPtr).lPos < (*infoPtr).lRangeMin {
        (*infoPtr).lPos = (*infoPtr).lRangeMin;
    }

    if (*infoPtr).lPos > (*infoPtr).lRangeMax {
        (*infoPtr).lPos = (*infoPtr).lRangeMax;
    }

    if fPosition != 0 && oldPos != lPosition {
        TRACKBAR_UpdateThumb(infoPtr);
        TRACKBAR_InvalidateThumbMove(infoPtr, oldPos, lPosition);
    }

    return 0;
}

unsafe extern "C" fn TRACKBAR_SetRange(infoPtr: *mut TRACKBAR_INFO, redraw: BOOL, range: LONG) -> LRESULT {
    let changed: BOOL = ((*infoPtr).lRangeMin != (LOWORD!(range as DWORD) as SHORT) as LONG
        || (*infoPtr).lRangeMax != (HIWORD!(range as DWORD) as SHORT) as LONG) as BOOL;

    (*infoPtr).lRangeMin = (LOWORD!(range as DWORD) as SHORT) as LONG;
    (*infoPtr).lRangeMax = (HIWORD!(range as DWORD) as SHORT) as LONG;

    /* clip position to new min/max limit */
    if (*infoPtr).lPos < (*infoPtr).lRangeMin {
        (*infoPtr).lPos = (*infoPtr).lRangeMin;
    }

    if (*infoPtr).lPos > (*infoPtr).lRangeMax {
        (*infoPtr).lPos = (*infoPtr).lRangeMax;
    }

    TRACKBAR_UpdatePageSize(infoPtr);

    if changed != 0 {
        if (*infoPtr).dwStyle & TBS_AUTOTICKS != 0 {
            TRACKBAR_RecalculateTics(infoPtr);
        }
        (*infoPtr).flags |= TB_THUMBPOSCHANGED;
    }

    if redraw != 0 {
        TRACKBAR_InvalidateAll(infoPtr);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn TRACKBAR_SetRangeMax(infoPtr: *mut TRACKBAR_INFO, redraw: BOOL, lMax: LONG) -> LRESULT {
    let changed: BOOL = ((*infoPtr).lRangeMax != lMax) as BOOL;
    let rightmost: LONG = max(lMax, (*infoPtr).lRangeMin);

    (*infoPtr).lRangeMax = lMax;
    if (*infoPtr).lPos > rightmost {
        (*infoPtr).lPos = rightmost;
        (*infoPtr).flags |= TB_THUMBPOSCHANGED;
    }

    TRACKBAR_UpdatePageSize(infoPtr);

    if changed != 0 && ((*infoPtr).dwStyle & TBS_AUTOTICKS) != 0 {
        TRACKBAR_RecalculateTics(infoPtr);
    }

    if redraw != 0 {
        TRACKBAR_InvalidateAll(infoPtr);
    }

    return 0;
}

unsafe extern "C" fn TRACKBAR_SetRangeMin(infoPtr: *mut TRACKBAR_INFO, redraw: BOOL, lMin: LONG) -> LRESULT {
    let changed: BOOL = ((*infoPtr).lRangeMin != lMin) as BOOL;

    (*infoPtr).lRangeMin = lMin;
    if (*infoPtr).lPos < (*infoPtr).lRangeMin {
        (*infoPtr).lPos = (*infoPtr).lRangeMin;
        (*infoPtr).flags |= TB_THUMBPOSCHANGED;
    }

    TRACKBAR_UpdatePageSize(infoPtr);

    if changed != 0 && ((*infoPtr).dwStyle & TBS_AUTOTICKS) != 0 {
        TRACKBAR_RecalculateTics(infoPtr);
    }

    if redraw != 0 {
        TRACKBAR_InvalidateAll(infoPtr);
    }

    return 0;
}

unsafe extern "C" fn TRACKBAR_SetSel(infoPtr: *mut TRACKBAR_INFO, fRedraw: BOOL, lSel: LONG) -> LRESULT {
    if ((*infoPtr).dwStyle & TBS_ENABLESELRANGE) == 0 {
        (*infoPtr).lSelMin = 0;
        (*infoPtr).lSelMax = 0;
        return 0;
    }

    (*infoPtr).lSelMin = (LOWORD!(lSel as DWORD) as SHORT) as LONG;
    (*infoPtr).lSelMax = (HIWORD!(lSel as DWORD) as SHORT) as LONG;
    (*infoPtr).flags |= TB_SELECTIONCHANGED;

    if (*infoPtr).lSelMin < (*infoPtr).lRangeMin {
        (*infoPtr).lSelMin = (*infoPtr).lRangeMin;
    }
    if (*infoPtr).lSelMax > (*infoPtr).lRangeMax {
        (*infoPtr).lSelMax = (*infoPtr).lRangeMax;
    }

    if fRedraw != 0 {
        TRACKBAR_InvalidateAll(infoPtr);
    }

    return 0;
}

unsafe extern "C" fn TRACKBAR_SetSelEnd(infoPtr: *mut TRACKBAR_INFO, fRedraw: BOOL, lEnd: LONG) -> LRESULT {
    if ((*infoPtr).dwStyle & TBS_ENABLESELRANGE) == 0 {
        (*infoPtr).lSelMax = 0;
        return 0;
    }

    (*infoPtr).lSelMax = lEnd;
    (*infoPtr).flags |= TB_SELECTIONCHANGED;

    if (*infoPtr).lSelMax > (*infoPtr).lRangeMax {
        (*infoPtr).lSelMax = (*infoPtr).lRangeMax;
    }

    if fRedraw != 0 {
        TRACKBAR_InvalidateAll(infoPtr);
    }

    return 0;
}

unsafe extern "C" fn TRACKBAR_SetSelStart(infoPtr: *mut TRACKBAR_INFO, fRedraw: BOOL, lStart: LONG) -> LRESULT {
    if ((*infoPtr).dwStyle & TBS_ENABLESELRANGE) == 0 {
        (*infoPtr).lSelMin = 0;
        return 0;
    }

    (*infoPtr).lSelMin = lStart;
    (*infoPtr).flags |= TB_SELECTIONCHANGED;

    if (*infoPtr).lSelMin < (*infoPtr).lRangeMin {
        (*infoPtr).lSelMin = (*infoPtr).lRangeMin;
    }

    if fRedraw != 0 {
        TRACKBAR_InvalidateAll(infoPtr);
    }

    return 0;
}

unsafe extern "C" fn TRACKBAR_SetThumbLength(infoPtr: *mut TRACKBAR_INFO, iLength: UINT) -> LRESULT {
    if (((*infoPtr).dwStyle & TBS_FIXEDLENGTH) != 0) {
        /* We're not supposed to check if it's really changed or not,
           just repaint in any case. */
        (*infoPtr).uThumbLen = iLength;
        (*infoPtr).flags |= TB_THUMBSIZECHANGED;
        TRACKBAR_InvalidateAll(infoPtr);
    }

    0
}

unsafe extern "C" fn TRACKBAR_SetTic(infoPtr: *mut TRACKBAR_INFO, lPos: LONG) -> LRESULT {
    if (lPos < (*infoPtr).lRangeMin) || (lPos > (*infoPtr).lRangeMax) {
        return (FALSE) as LRESULT;
    }

    (*infoPtr).uNumTics += 1;
    (*infoPtr).tics = ReAlloc(
        (*infoPtr).tics as *mut core::ffi::c_void,
        (((*infoPtr).uNumTics as usize) * core::mem::size_of::<DWORD>()) as DWORD,
    ) as LPLONG;
    if (*infoPtr).tics.is_null() {
        (*infoPtr).uNumTics = 0;
        notify(infoPtr, (NM_OUTOFMEMORY) as i32);
        return (FALSE) as LRESULT;
    }
    *(*infoPtr).tics.add(((*infoPtr).uNumTics - 1) as usize) = lPos;

    TRACKBAR_InvalidateAll(infoPtr);

    return (TRUE) as LRESULT;
}

unsafe extern "C" fn TRACKBAR_SetTicFreq(infoPtr: *mut TRACKBAR_INFO, wFreq: WORD) -> LRESULT {
    if (*infoPtr).dwStyle & TBS_AUTOTICKS != 0 {
        (*infoPtr).uTicFreq = wFreq as UINT;
        TRACKBAR_RecalculateTics(infoPtr);
        TRACKBAR_InvalidateAll(infoPtr);
    }

    TRACKBAR_UpdateThumb(infoPtr);
    return 0;
}

unsafe extern "C" fn TRACKBAR_SetTipSide(infoPtr: *mut TRACKBAR_INFO, fLocation: INT) -> INT {
    let fTemp: INT = (*infoPtr).fLocation;

    (*infoPtr).fLocation = fLocation;

    return fTemp;
}

unsafe extern "C" fn TRACKBAR_SetToolTips(infoPtr: *mut TRACKBAR_INFO, hwndTT: HWND) -> LRESULT {
    (*infoPtr).hwndToolTip = hwndTT;

    return 0;
}

unsafe extern "C" fn TRACKBAR_SetUnicodeFormat(infoPtr: *mut TRACKBAR_INFO, fUnicode: BOOL) -> BOOL {
    let mut bTemp: BOOL = (*infoPtr).bUnicode;

    (*infoPtr).bUnicode = fUnicode;

    return bTemp;
}

unsafe extern "C" fn get_scaled_metric(infoPtr: *const TRACKBAR_INFO, value: i32) -> i32 {
    unsafe {
        return MulDiv(value, GetDpiForWindow((*infoPtr).hwndSelf) as i32, 96);
    }
}

unsafe extern "C" fn TRACKBAR_InitializeThumb(infoPtr: *mut TRACKBAR_INFO) -> LRESULT {
    let mut client_size: i32;
    let mut rect: RECT = core::mem::zeroed();

    (*infoPtr).uThumbLen = get_scaled_metric(infoPtr, if ((*infoPtr).dwStyle & TBS_ENABLESELRANGE) != 0 { 23 } else { 21 }) as UINT;

    if ((*infoPtr).dwStyle & TBS_FIXEDLENGTH) == 0 {
        GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rect));
        if ((*infoPtr).dwStyle & TBS_VERT) != 0 {
            client_size = rect.right - rect.left;
        } else {
            client_size = rect.bottom - rect.top;
        }

        if ((client_size as UINT) < (*infoPtr).uThumbLen) {
            (*infoPtr).uThumbLen = (if client_size > get_scaled_metric(infoPtr, 9) {
                client_size - get_scaled_metric(infoPtr, 5)
            } else {
                get_scaled_metric(infoPtr, 4)
            }) as UINT;
        }
    }

    TRACKBAR_CalcChannel(infoPtr);
    TRACKBAR_UpdateThumb(infoPtr);
    (*infoPtr).flags &= !TB_SELECTIONCHANGED;

    return 0;
}

unsafe extern "C" fn TRACKBAR_Create(hwnd: HWND, lpcs: *const CREATESTRUCTW) -> LRESULT {
    let mut infoPtr: *mut TRACKBAR_INFO;

    infoPtr = Alloc((core::mem::size_of::<TRACKBAR_INFO>()) as DWORD) as *mut TRACKBAR_INFO;
    if infoPtr.is_null() {
        return -1;
    }
    SetWindowLongPtrW(hwnd, 0, (infoPtr as DWORD_PTR) as LONG_PTR);

    /* set default values */
    (*infoPtr).hwndSelf = hwnd;
    (*infoPtr).dwStyle = ((*lpcs).style) as u32;
    (*infoPtr).lRangeMin = 0;
    (*infoPtr).lRangeMax = 100;
    (*infoPtr).lLineSize = 1;
    (*infoPtr).lPageSize = TB_DEFAULTPAGESIZE;
    (*infoPtr).lSelMin = 0;
    (*infoPtr).lSelMax = 0;
    (*infoPtr).lPos = 0;
    (*infoPtr).fLocation = (TBTS_TOP as i32);
    (*infoPtr).uNumTics = 0; /* start and end tic are not included in count*/
    (*infoPtr).uTicFreq = 1;
    (*infoPtr).tics = core::ptr::null_mut();
    (*infoPtr).hwndNotify = (*lpcs).hwndParent;

    TRACKBAR_InitializeThumb(infoPtr);

    /* Create tooltip control */
    if ((*infoPtr).dwStyle & TBS_TOOLTIPS) != 0 {
        (*infoPtr).hwndToolTip = CreateWindowExW(
            0,
            TOOLTIPS_CLASSW.as_ptr(),
            core::ptr::null_mut(),
            WS_POPUP,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            hwnd,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        if !(*infoPtr).hwndToolTip.is_null() {
            let mut ti: TTTOOLINFOW = core::mem::zeroed();
            let mut wEmpty: u16 = 0;
            ZeroMemory!(core::ptr::addr_of_mut!(ti) as *mut _, core::mem::size_of::<TTTOOLINFOW>());
            ti.cbSize = core::mem::size_of::<TTTOOLINFOW>() as u32;
            ti.uFlags = TTF_IDISHWND | TTF_TRACK | TTF_ABSOLUTE;
            ti.hwnd = hwnd;
            ti.lpszText = core::ptr::addr_of_mut!(wEmpty);

            SendMessageW(
                (*infoPtr).hwndToolTip,
                TTM_ADDTOOLW,
                0,
                core::ptr::addr_of_mut!(ti) as LPARAM,
            );
        }
    }

    OpenThemeData(hwnd, themeClass.as_ptr());

    return 0;
}

unsafe extern "C" fn TRACKBAR_Destroy(infoPtr: *mut TRACKBAR_INFO) -> LRESULT {
    /* delete tooltip control */
    if (*infoPtr).hwndToolTip as usize != 0 {
        DestroyWindow((*infoPtr).hwndToolTip);
    }

    Free(((*infoPtr).tics).cast());
    (*infoPtr).tics = core::ptr::null_mut();

    SetWindowLongPtrW((*infoPtr).hwndSelf, 0, 0);
    CloseThemeData(GetWindowTheme((*infoPtr).hwndSelf));
    Free((infoPtr).cast());

    return 0;
}

unsafe extern "C" fn TRACKBAR_KillFocus(infoPtr: *mut TRACKBAR_INFO) -> LRESULT {
    (*infoPtr).flags &= !TB_IS_FOCUSED;
    TRACKBAR_InvalidateAll(infoPtr);

    return 0;
}

unsafe extern "C" fn TRACKBAR_LButtonDown(infoPtr: *mut TRACKBAR_INFO, x: INT, y: INT) -> LRESULT {
    let mut clickPoint: POINT = core::mem::zeroed();

    clickPoint.x = x;
    clickPoint.y = y;

    SetFocus((*infoPtr).hwndSelf);

    if PtInRect(core::ptr::addr_of_mut!((*infoPtr).rcThumb), clickPoint) != 0 {
        (*infoPtr).flags |= TB_DRAG_MODE;
        SetCapture((*infoPtr).hwndSelf);
        TRACKBAR_UpdateToolTip(infoPtr);
        TRACKBAR_ActivateToolTip(infoPtr, TRUE);
        TRACKBAR_InvalidateThumb(infoPtr, (*infoPtr).lPos);
    } else {
        let dir: LONG = TRACKBAR_GetAutoPageDirection(infoPtr, clickPoint);
        if dir == 0 {
            return 0;
        }
        (*infoPtr).flags |= if dir < 0 {
            TB_AUTO_PAGE_LEFT
        } else {
            TB_AUTO_PAGE_RIGHT
        };
        TRACKBAR_AutoPage(infoPtr, clickPoint);
        SetCapture((*infoPtr).hwndSelf);
        SetTimer((*infoPtr).hwndSelf, TB_REFRESH_TIMER as UINT_PTR, TB_REFRESH_DELAY, None);
    }

    return 0;
}

unsafe extern "C" fn TRACKBAR_LButtonUp(infoPtr: *mut TRACKBAR_INFO) -> LRESULT {
    if (*infoPtr).flags & TB_DRAG_MODE != 0 {
        notify_with_scroll(infoPtr, TB_THUMBPOSITION | (((*infoPtr).lPos << 16)) as u32);
        notify_with_scroll(infoPtr, TB_ENDTRACK);
        (*infoPtr).flags &= !TB_DRAG_MODE;
        ReleaseCapture();
        notify(infoPtr, (NM_RELEASEDCAPTURE) as i32);
        TRACKBAR_ActivateToolTip(infoPtr, FALSE);
        TRACKBAR_InvalidateThumb(infoPtr, (*infoPtr).lPos);
    }
    if (*infoPtr).flags & TB_AUTO_PAGE != 0 {
        KillTimer((*infoPtr).hwndSelf, (TB_REFRESH_TIMER) as UINT_PTR);
        (*infoPtr).flags &= !TB_AUTO_PAGE;
        notify_with_scroll(infoPtr, TB_ENDTRACK);
        ReleaseCapture();
        notify(infoPtr, (NM_RELEASEDCAPTURE) as i32);
    }

    return 0;
}

unsafe extern "C" fn TRACKBAR_CaptureChanged(infoPtr: *const TRACKBAR_INFO) -> LRESULT {
    notify_with_scroll(infoPtr, TB_ENDTRACK);
    return 0;
}

unsafe extern "C" fn TRACKBAR_Paint(infoPtr: *mut TRACKBAR_INFO, mut hdc: HDC) -> LRESULT {
    if !hdc.is_null() {
        TRACKBAR_Refresh(infoPtr, hdc);
    } else {
        let mut ps: PAINTSTRUCT = core::mem::zeroed();
        hdc = BeginPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
        TRACKBAR_Refresh(infoPtr, hdc);
        EndPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
    }

    return 0;
}

unsafe extern "C" fn TRACKBAR_SetFocus(infoPtr: *mut TRACKBAR_INFO) -> LRESULT {
    (*infoPtr).flags |= TB_IS_FOCUSED;
    TRACKBAR_InvalidateAll(infoPtr);

    return 0;
}

unsafe extern "C" fn TRACKBAR_Size(infoPtr: *mut TRACKBAR_INFO) -> LRESULT {
    if (*infoPtr).dwStyle & TBS_FIXEDLENGTH != 0 {
        TRACKBAR_CalcChannel(infoPtr);
        TRACKBAR_UpdateThumb(infoPtr);
    } else {
        TRACKBAR_InitializeThumb(infoPtr);
    }
    TRACKBAR_AlignBuddies(infoPtr);
    TRACKBAR_InvalidateAll(infoPtr);

    return 0;
}

unsafe extern "C" fn TRACKBAR_StyleChanged(infoPtr: *mut TRACKBAR_INFO, wStyleType: WPARAM, lpss: *const STYLESTRUCT) -> LRESULT {
    if wStyleType != GWL_STYLE as WPARAM { return 0 as LRESULT; }

    (*infoPtr).dwStyle = (*lpss).styleNew;

    return 0 as LRESULT;
}

unsafe extern "C" fn TRACKBAR_Timer(infoPtr: *mut TRACKBAR_INFO) -> LRESULT {
    if ((*infoPtr).flags & TB_AUTO_PAGE) != 0 {
        let mut pt: POINT = core::mem::zeroed();
        if GetCursorPos(core::ptr::addr_of_mut!(pt)) != 0 {
            if ScreenToClient((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(pt)) != 0 {
                TRACKBAR_AutoPage(infoPtr, pt);
            }
        }
    }
    return 0;
}

unsafe extern "C" fn theme_changed(infoPtr: *const TRACKBAR_INFO) -> LRESULT {
    let theme: HTHEME = GetWindowTheme((*infoPtr).hwndSelf);
    CloseThemeData(theme);
    OpenThemeData((*infoPtr).hwndSelf, themeClass.as_ptr());
    return 0;
}

unsafe extern "C" fn TRACKBAR_MouseMove(infoPtr: *mut TRACKBAR_INFO, x: INT, y: INT) -> LRESULT {
    let clickPlace: INT = if ((*infoPtr).dwStyle & TBS_VERT) != 0 { y } else { x };
    let mut dragPos: LONG;
    let oldPos: LONG = (*infoPtr).lPos;

    if ((*infoPtr).flags & TB_AUTO_PAGE) != 0 {
        let mut pt: POINT = core::mem::zeroed();
        pt.x = x;
        pt.y = y;
        TRACKBAR_AutoPage(infoPtr, pt);
        return TRUE as LRESULT;
    }

    if !(((*infoPtr).flags & TB_DRAG_MODE) != 0) {
        if !GetWindowTheme((*infoPtr).hwndSelf).is_null() {
            let oldFlags: DWORD = (*infoPtr).flags;
            let mut pt: POINT = core::mem::zeroed();
            pt.x = x;
            pt.y = y;
            if PtInRect(core::ptr::addr_of_mut!((*infoPtr).rcThumb), pt) != 0 {
                let mut tme: TRACKMOUSEEVENT = core::mem::zeroed();
                tme.cbSize = core::mem::size_of::<TRACKMOUSEEVENT>() as DWORD;
                tme.dwFlags = TME_LEAVE;
                tme.hwndTrack = (*infoPtr).hwndSelf;
                TrackMouseEvent(core::ptr::addr_of_mut!(tme));
                (*infoPtr).flags |= TB_THUMB_HOT;
            } else {
                let mut tme: TRACKMOUSEEVENT = core::mem::zeroed();
                tme.cbSize = core::mem::size_of::<TRACKMOUSEEVENT>() as DWORD;
                tme.dwFlags = TME_CANCEL;
                tme.hwndTrack = (*infoPtr).hwndSelf;
                TrackMouseEvent(core::ptr::addr_of_mut!(tme));
                (*infoPtr).flags &= !TB_THUMB_HOT;
            }
            if oldFlags != (*infoPtr).flags {
                InvalidateRect(
                    (*infoPtr).hwndSelf,
                    core::ptr::addr_of_mut!((*infoPtr).rcThumb),
                    FALSE,
                );
            }
        }
        return TRUE as LRESULT;
    }

    dragPos = TRACKBAR_ConvertPlaceToPosition(infoPtr, clickPlace);

    if dragPos == oldPos {
        return TRUE as LRESULT;
    }

    (*infoPtr).lPos = dragPos;
    TRACKBAR_UpdateThumb(infoPtr);

    notify_with_scroll(
        infoPtr,
        (TB_THUMBTRACK as UINT) | (((*infoPtr).lPos as UINT) << 16),
    );

    TRACKBAR_InvalidateThumbMove(infoPtr, oldPos, dragPos);
    UpdateWindow((*infoPtr).hwndSelf);

    return TRUE as LRESULT;
}

unsafe extern "C" fn TRACKBAR_KeyDown(infoPtr: *mut TRACKBAR_INFO, nVirtKey: INT) -> BOOL {
    let downIsLeft: BOOL = ((*infoPtr).dwStyle & TBS_DOWNISLEFT) as BOOL;
    let vert: BOOL = ((*infoPtr).dwStyle & TBS_VERT) as BOOL;
    let pos: LONG = (*infoPtr).lPos;

    match (nVirtKey) as u32 {
        VK_UP => {
            if vert == 0 && downIsLeft != 0 {
                TRACKBAR_LineDown(infoPtr);
            } else {
                TRACKBAR_LineUp(infoPtr);
            }
        }
        VK_LEFT => {
            if vert != 0 && downIsLeft != 0 {
                TRACKBAR_LineDown(infoPtr);
            } else {
                TRACKBAR_LineUp(infoPtr);
            }
        }
        VK_DOWN => {
            if vert == 0 && downIsLeft != 0 {
                TRACKBAR_LineUp(infoPtr);
            } else {
                TRACKBAR_LineDown(infoPtr);
            }
        }
        VK_RIGHT => {
            if vert != 0 && downIsLeft != 0 {
                TRACKBAR_LineUp(infoPtr);
            } else {
                TRACKBAR_LineDown(infoPtr);
            }
        }
        VK_NEXT => {
            if vert == 0 && downIsLeft != 0 {
                TRACKBAR_PageUp(infoPtr);
            } else {
                TRACKBAR_PageDown(infoPtr);
            }
        }
        VK_PRIOR => {
            if vert == 0 && downIsLeft != 0 {
                TRACKBAR_PageDown(infoPtr);
            } else {
                TRACKBAR_PageUp(infoPtr);
            }
        }
        VK_HOME => {
            if (*infoPtr).lPos == (*infoPtr).lRangeMin {
                return FALSE;
            }
            (*infoPtr).lPos = (*infoPtr).lRangeMin;
            notify_with_scroll(infoPtr, TB_TOP);
        }
        VK_END => {
            if (*infoPtr).lPos == (*infoPtr).lRangeMax {
                return FALSE;
            }
            (*infoPtr).lPos = (*infoPtr).lRangeMax;
            notify_with_scroll(infoPtr, TB_BOTTOM);
        }
        _ => {}
    }

    if pos != (*infoPtr).lPos {
        TRACKBAR_UpdateThumb(infoPtr);
        TRACKBAR_InvalidateThumbMove(infoPtr, pos, (*infoPtr).lPos);
    }

    return TRUE;
}

unsafe extern "C" fn TRACKBAR_KeyUp(infoPtr: *const TRACKBAR_INFO, nVirtKey: INT) -> BOOL {
    match (nVirtKey) as u32 {
        VK_LEFT
        | VK_UP
        | VK_RIGHT
        | VK_DOWN
        | VK_NEXT
        | VK_PRIOR
        | VK_HOME
        | VK_END => {
            notify_with_scroll(infoPtr, TB_ENDTRACK);
        }
        _ => {}
    }
    return TRUE;
}

unsafe extern "C" fn TRACKBAR_Enable(infoPtr: *mut TRACKBAR_INFO, enable: BOOL) -> LRESULT {
    if enable != 0 {
        (*infoPtr).dwStyle &= !WS_DISABLED;
    } else {
        (*infoPtr).dwStyle |= WS_DISABLED;
    }

    InvalidateRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!((*infoPtr).rcThumb), TRUE);

    return 1;
}

unsafe extern "system" fn TRACKBAR_WindowProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut infoPtr: *mut TRACKBAR_INFO = GetWindowLongPtrW(hwnd, 0) as *mut TRACKBAR_INFO;

    if infoPtr.is_null() && (uMsg != WM_CREATE) {
        return DefWindowProcW(hwnd, uMsg, wParam, lParam) as LRESULT;
    }

    match uMsg {
        TBM_CLEARSEL => {
            return TRACKBAR_ClearSel(infoPtr, wParam as BOOL) as LRESULT;
        }

        TBM_CLEARTICS => {
            return TRACKBAR_ClearTics(infoPtr, wParam as BOOL) as LRESULT;
        }

        TBM_GETBUDDY => {
            return (if wParam != 0 { (*infoPtr).hwndBuddyLA } else { (*infoPtr).hwndBuddyRB }) as LRESULT;
        }

        TBM_GETCHANNELRECT => {
            return TRACKBAR_GetChannelRect(infoPtr, lParam as LPRECT) as LRESULT;
        }

        TBM_GETLINESIZE => {
            return (*infoPtr).lLineSize as LRESULT;
        }

        TBM_GETNUMTICS => {
            return TRACKBAR_GetNumTics(infoPtr) as LRESULT;
        }

        TBM_GETPAGESIZE => {
            return (*infoPtr).lPageSize as LRESULT;
        }

        TBM_GETPOS => {
            return (*infoPtr).lPos as LRESULT;
        }

        TBM_GETPTICS => {
            return (*infoPtr).tics as LRESULT;
        }

        TBM_GETRANGEMAX => {
            return (*infoPtr).lRangeMax as LRESULT;
        }

        TBM_GETRANGEMIN => {
            return (*infoPtr).lRangeMin as LRESULT;
        }

        TBM_GETSELEND => {
            return (*infoPtr).lSelMax as LRESULT;
        }

        TBM_GETSELSTART => {
            return (*infoPtr).lSelMin as LRESULT;
        }

        TBM_GETTHUMBLENGTH => {
            return (*infoPtr).uThumbLen as LRESULT;
        }

        TBM_GETTHUMBRECT => {
            return CopyRect(lParam as LPRECT, core::ptr::addr_of_mut!((*infoPtr).rcThumb)) as LRESULT;
        }

        TBM_GETTIC => {
            return TRACKBAR_GetTic(infoPtr, wParam as INT) as LRESULT;
        }

        TBM_GETTICPOS => {
            return TRACKBAR_GetTicPos(infoPtr, wParam as INT) as LRESULT;
        }

        TBM_GETTOOLTIPS => {
            return (*infoPtr).hwndToolTip as LRESULT;
        }

        TBM_GETUNICODEFORMAT => {
            return (*infoPtr).bUnicode as LRESULT;
        }

        TBM_SETBUDDY => {
            return TRACKBAR_SetBuddy(infoPtr, wParam as BOOL, lParam as HWND) as LRESULT;
        }

        TBM_SETLINESIZE => {
            return TRACKBAR_SetLineSize(infoPtr, lParam as LONG) as LRESULT;
        }

        TBM_SETPAGESIZE => {
            return TRACKBAR_SetPageSize(infoPtr, lParam as LONG) as LRESULT;
        }

        TBM_SETPOS => {
            return TRACKBAR_SetPos(infoPtr, wParam as BOOL, lParam as LONG) as LRESULT;
        }

        TBM_SETRANGE => {
            return TRACKBAR_SetRange(infoPtr, wParam as BOOL, lParam as LONG) as LRESULT;
        }

        TBM_SETRANGEMAX => {
            return TRACKBAR_SetRangeMax(infoPtr, wParam as BOOL, lParam as LONG) as LRESULT;
        }

        TBM_SETRANGEMIN => {
            return TRACKBAR_SetRangeMin(infoPtr, wParam as BOOL, lParam as LONG) as LRESULT;
        }

        TBM_SETSEL => {
            return TRACKBAR_SetSel(infoPtr, wParam as BOOL, lParam as LONG) as LRESULT;
        }

        TBM_SETSELEND => {
            return TRACKBAR_SetSelEnd(infoPtr, wParam as BOOL, lParam as LONG) as LRESULT;
        }

        TBM_SETSELSTART => {
            return TRACKBAR_SetSelStart(infoPtr, wParam as BOOL, lParam as LONG) as LRESULT;
        }

        TBM_SETTHUMBLENGTH => {
            return TRACKBAR_SetThumbLength(infoPtr, wParam as UINT) as LRESULT;
        }

        TBM_SETTIC => {
            return TRACKBAR_SetTic(infoPtr, lParam as LONG) as LRESULT;
        }

        TBM_SETTICFREQ => {
            return TRACKBAR_SetTicFreq(infoPtr, wParam as WORD) as LRESULT;
        }

        TBM_SETTIPSIDE => {
            return TRACKBAR_SetTipSide(infoPtr, wParam as INT) as LRESULT;
        }

        TBM_SETTOOLTIPS => {
            return TRACKBAR_SetToolTips(infoPtr, wParam as HWND) as LRESULT;
        }

        TBM_SETUNICODEFORMAT => {
            return TRACKBAR_SetUnicodeFormat(infoPtr, wParam as BOOL) as LRESULT;
        }

        WM_CAPTURECHANGED => {
            if hwnd == (lParam as HWND) {
                return 0;
            }
            return TRACKBAR_CaptureChanged(infoPtr) as LRESULT;
        }

        WM_CREATE => {
            return TRACKBAR_Create(hwnd, lParam as LPCREATESTRUCTW) as LRESULT;
        }

        WM_DESTROY => {
            return TRACKBAR_Destroy(infoPtr) as LRESULT;
        }

        WM_ENABLE => {
            return TRACKBAR_Enable(infoPtr, wParam as BOOL) as LRESULT;
        }

        WM_ERASEBKGND => {
            return 0;
        }

        WM_GETDLGCODE => {
            return (DLGC_WANTARROWS) as LRESULT;
        }

        WM_KEYDOWN => {
            return TRACKBAR_KeyDown(infoPtr, wParam as INT) as LRESULT;
        }

        WM_KEYUP => {
            return TRACKBAR_KeyUp(infoPtr, wParam as INT) as LRESULT;
        }

        WM_KILLFOCUS => {
            return TRACKBAR_KillFocus(infoPtr) as LRESULT;
        }

        WM_LBUTTONDOWN => {
            return TRACKBAR_LButtonDown(infoPtr, (LOWORD!(lParam) as SHORT) as INT, (HIWORD!(lParam) as SHORT) as INT) as LRESULT;
        }

        WM_LBUTTONUP => {
            return TRACKBAR_LButtonUp(infoPtr) as LRESULT;
        }

        WM_MOUSELEAVE => {
            (*infoPtr).flags &= !TB_THUMB_HOT;
            InvalidateRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!((*infoPtr).rcThumb), FALSE);
            return 0;
        }

        WM_MOUSEMOVE => {
            return TRACKBAR_MouseMove(infoPtr, (LOWORD!(lParam) as SHORT) as INT, (HIWORD!(lParam) as SHORT) as INT) as LRESULT;
        }

        WM_PRINTCLIENT | WM_PAINT => {
            return TRACKBAR_Paint(infoPtr, wParam as HDC) as LRESULT;
        }

        WM_SETFOCUS => {
            return TRACKBAR_SetFocus(infoPtr) as LRESULT;
        }

        WM_SIZE => {
            return TRACKBAR_Size(infoPtr) as LRESULT;
        }

        WM_STYLECHANGED => {
            return TRACKBAR_StyleChanged(infoPtr, wParam, lParam as LPSTYLESTRUCT) as LRESULT;
        }

        WM_THEMECHANGED => {
            return theme_changed(infoPtr) as LRESULT;
        }

        WM_TIMER => {
            return TRACKBAR_Timer(infoPtr) as LRESULT;
        }

        WM_WININICHANGE => {
            return TRACKBAR_InitializeThumb(infoPtr) as LRESULT;
        }

        _ => {
            if (uMsg >= WM_USER) && (uMsg < WM_APP) && !(COMCTL32_IsReflectedMessage(uMsg) != 0) {
                ERR!(b"unknown msg %04x wp=%08lx lp=%08lx\n\0".as_ptr() as *const i8, uMsg, wParam, lParam);
            }
            return DefWindowProcW(hwnd, uMsg, wParam, lParam) as LRESULT;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn TRACKBAR_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(core::ptr::addr_of_mut!(wndClass) as *mut _, core::mem::size_of::<WNDCLASSW>() as usize);
    wndClass.style = CS_GLOBALCLASS as _;
    wndClass.lpfnWndProc = Some(TRACKBAR_WindowProc);
    wndClass.cbClsExtra = 0 as _;
    wndClass.cbWndExtra = core::mem::size_of::<TRACKBAR_INFO>() as _;
    wndClass.hCursor = LoadCursorW(0 as _, IDC_ARROW as usize as LPWSTR);
    wndClass.hbrBackground = (COLOR_BTNFACE + 1) as HBRUSH;
    wndClass.lpszClassName = TRACKBAR_CLASSW as _;

    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn TRACKBAR_Unregister() {
    UnregisterClassW(TRACKBAR_CLASSW.as_ptr(), core::ptr::null_mut());
}
