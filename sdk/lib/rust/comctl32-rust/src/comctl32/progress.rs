use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_progress);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const PROGRESS_CLASSW: [::core::ffi::c_ushort; 18] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_ushort; 18]>(
        *b"m\0s\0c\0t\0l\0s\0_\0p\0r\0o\0g\0r\0e\0s\0s\x003\x002\0\0\0",
    )
};

static mut drawProcClassic: [ProgressDrawProc; 8] = unsafe {
    [
        Some(
            draw_solid_bar_H
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
        Some(
            draw_solid_bkg_H
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
        Some(
            draw_solid_bar_V
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
        Some(
            draw_solid_bkg_V
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
        Some(
            draw_chunk_bar_H
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
        Some(
            draw_solid_bkg_H
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
        Some(
            draw_chunk_bar_V
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
        Some(
            draw_solid_bkg_V
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
    ]
};

static mut drawProcThemed: [ProgressDrawProc; 8] = unsafe {
    [
        Some(
            draw_theme_bar_H
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
        Some(
            draw_theme_bkg_H
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
        Some(
            draw_theme_bar_V
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
        Some(
            draw_theme_bkg_V
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
        Some(
            draw_theme_bar_H
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
        Some(
            draw_theme_bkg_H
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
        Some(
            draw_theme_bar_V
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
        Some(
            draw_theme_bkg_V
                as unsafe extern "C" fn(
                    *const ProgressDrawInfo,
                    ::core::ffi::c_int,
                    ::core::ffi::c_int,
                ) -> (),
        ),
    ]
};

// WINE_DEFAULT_DEBUG_CHANNEL(progress); (debug channel declaration, no Rust equivalent)

#[repr(C)]
pub struct PROGRESS_INFO {
    pub Self_: HWND, // C field `Self`; renamed because `Self` is a reserved Rust identifier
    pub CurVal: i32,
    pub MinVal: i32,
    pub MaxVal: i32,
    pub Step: i32,
    pub MarqueePos: i32,
    pub Marquee: i32,
    pub ColorBar: COLORREF,
    pub ColorBk: COLORREF,
    pub Font: HFONT,
}
pub type PPROGRESS_INFO = *mut PROGRESS_INFO;

pub const LED_GAP: i32 = 2;

pub const MARQUEE_LEDS: i32 = 5;

pub const ID_MARQUEE_TIMER: usize = 1;

pub const DEFAULT_MARQUEE_PERIOD: i32 = 30;

#[repr(C)]
pub struct tagProgressDrawInfo {
    pub hdc: HDC,
    pub rect: RECT,
    pub hbrBar: HBRUSH,
    pub hbrBk: HBRUSH,
    pub ledW: i32,
    pub ledGap: i32,
    pub theme: HTHEME,
    pub bgRect: RECT,
}
pub type ProgressDrawInfo = tagProgressDrawInfo;

pub type ProgressDrawProc = Option<unsafe extern "C" fn(di: *const ProgressDrawInfo, start: i32, end: i32)>;

unsafe extern "C" fn get_led_size(
    infoPtr: *const PROGRESS_INFO,
    style: LONG,
    rect: *const RECT,
) -> ::core::ffi::c_int {
    let mut theme: HTHEME = GetWindowTheme((*infoPtr).Self_);
    if !theme.is_null() {
        let mut chunkSize: ::core::ffi::c_int = 0;
        if GetThemeInt(
            theme,
            0,
            0,
            (TMT_PROGRESSCHUNKSIZE as i32),
            core::ptr::addr_of_mut!(chunkSize),
        ) >= 0
        {
            return chunkSize;
        }
    }
    if style & (PBS_VERTICAL as i32) != 0 {
        return MulDiv(
            (*rect).right - (*rect).left,
            2,
            3,
        );
    } else {
        return MulDiv(
            (*rect).bottom - (*rect).top,
            2,
            3,
        );
    };
}

unsafe extern "C" fn get_led_gap(infoPtr: *const PROGRESS_INFO) -> ::core::ffi::c_int {
    let theme: HTHEME = GetWindowTheme((*infoPtr).Self_);
    if !theme.is_null() {
        let mut spaceSize: ::core::ffi::c_int = 0;
        if (GetThemeInt(theme, 0, 0, (TMT_PROGRESSSPACESIZE as i32), core::ptr::addr_of_mut!(spaceSize)) >= 0) {
            return spaceSize;
        }
    }

    return LED_GAP;
}

unsafe extern "C" fn get_client_rect(hwnd: HWND, rect: *mut RECT) {
    let theme: HTHEME = GetWindowTheme(hwnd);
    GetClientRect(hwnd, rect);
    if theme.is_null() {
        InflateRect(rect, -1, -1);
    } else {
        let dwStyle: DWORD = GetWindowLongW(hwnd, GWL_STYLE) as DWORD;
        let part: ::core::ffi::c_int = if (dwStyle & PBS_VERTICAL) != 0 {
            (PP_BARVERT) as i32
        } else {
            (PP_BAR) as i32
        };
        GetThemeBackgroundContentRect(theme, core::ptr::null_mut(), part, 0, rect, rect);
    }
}

unsafe extern "C" fn get_bar_size(style: LONG, rect: *const RECT) -> i32 {
    if style & (PBS_VERTICAL as i32) != 0 {
        return (*rect).bottom - (*rect).top;
    } else {
        return (*rect).right - (*rect).left;
    }
}

unsafe extern "C" fn get_bar_position(
    infoPtr: *const PROGRESS_INFO,
    style: LONG,
    rect: *const RECT,
    value: INT,
) -> INT {
    return MulDiv(
        value - (*infoPtr).MinVal,
        get_bar_size(style, rect),
        (*infoPtr).MaxVal - (*infoPtr).MinVal,
    );
}

unsafe extern "C" fn PROGRESS_Invalidate(infoPtr: *const PROGRESS_INFO, old: INT, new: INT) {
    InvalidateRect((*infoPtr).Self_, core::ptr::null_mut(), (old > new) as BOOL);
}

unsafe extern "C" fn draw_solid_bar_H(di: *const ProgressDrawInfo, start: i32, end: i32) {
    let mut r: RECT = core::mem::zeroed();
    SetRect(
        core::ptr::addr_of_mut!(r),
        (*di).rect.left + start,
        (*di).rect.top,
        (*di).rect.left + end,
        (*di).rect.bottom,
    );
    FillRect((*di).hdc, core::ptr::addr_of!(r), (*di).hbrBar);
}

unsafe extern "C" fn draw_solid_bkg_H(di: *const ProgressDrawInfo, start: i32, end: i32) {
    let mut r: RECT = core::mem::zeroed();
    SetRect(
        core::ptr::addr_of_mut!(r),
        (*di).rect.left + start,
        (*di).rect.top,
        (*di).rect.left + end,
        (*di).rect.bottom,
    );
    FillRect((*di).hdc, core::ptr::addr_of_mut!(r), (*di).hbrBk);
}

unsafe extern "C" fn draw_solid_bar_V(di: *const ProgressDrawInfo, start: i32, end: i32) {
    let mut r: RECT = core::mem::zeroed();
    SetRect(
        core::ptr::addr_of_mut!(r),
        (*di).rect.left,
        (*di).rect.bottom - end,
        (*di).rect.right,
        (*di).rect.bottom - start,
    );
    FillRect((*di).hdc, core::ptr::addr_of_mut!(r), (*di).hbrBar);
}

unsafe extern "C" fn draw_solid_bkg_V(di: *const ProgressDrawInfo, start: i32, end: i32) {
    let mut r: RECT = core::mem::zeroed();
    SetRect(core::ptr::addr_of_mut!(r), (*di).rect.left, (*di).rect.bottom - end, (*di).rect.right, (*di).rect.bottom - start);
    FillRect((*di).hdc, core::ptr::addr_of_mut!(r), (*di).hbrBk);
}

unsafe extern "C" fn draw_chunk_bar_H(di: *const ProgressDrawInfo, start: i32, end: i32) {
    let mut r: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let right: i32 = (*di).rect.left + end;
    r.left = (*di).rect.left + start;
    r.top = (*di).rect.top;
    r.bottom = (*di).rect.bottom;
    while r.left < right {
        r.right = if r.left + (*di).ledW < right { r.left + (*di).ledW } else { right };
        FillRect((*di).hdc, core::ptr::addr_of_mut!(r), (*di).hbrBar);
        r.left = r.right;
        r.right = if r.left + (*di).ledGap < right { r.left + (*di).ledGap } else { right };
        FillRect((*di).hdc, core::ptr::addr_of_mut!(r), (*di).hbrBk);
        r.left = r.right;
    }
}

unsafe extern "C" fn draw_chunk_bar_V(di: *const ProgressDrawInfo, start: i32, end: i32) {
    let mut r: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let top: i32 = (*di).rect.bottom - end;
    r.left = (*di).rect.left;
    r.right = (*di).rect.right;
    r.bottom = (*di).rect.bottom - start;
    while r.bottom > top {
        r.top = max(r.bottom - (*di).ledW, top);
        FillRect((*di).hdc, &mut r as *mut RECT, (*di).hbrBar);
        r.bottom = r.top;
        r.top = max(r.bottom - (*di).ledGap, top);
        FillRect((*di).hdc, &mut r as *mut RECT, (*di).hbrBk);
        r.bottom = r.top;
    }
}

unsafe extern "C" fn draw_theme_bar_H(di: *const ProgressDrawInfo, start: i32, end: i32) {
    let mut r: RECT = RECT { left: 0, top: 0, bottom: 0, right: 0 };
    r.left = ((*di).rect.left + start) as LONG;
    r.top = (*di).rect.top;
    r.bottom = (*di).rect.bottom;
    r.right = ((*di).rect.left + end) as LONG;
    DrawThemeBackground((*di).theme, (*di).hdc, (PP_CHUNK) as i32, 0, &raw mut r, core::ptr::null::<RECT>());
}

unsafe extern "C" fn draw_theme_bar_V(di: *const ProgressDrawInfo, start: i32, end: i32) {
    let mut r: RECT = core::mem::zeroed();
    r.left = (*di).rect.left;
    r.right = (*di).rect.right;
    r.bottom = (*di).rect.bottom - start;
    r.top = (*di).rect.bottom - end;
    DrawThemeBackground((*di).theme, (*di).hdc, (PP_CHUNKVERT) as i32, 0, core::ptr::addr_of_mut!(r), core::ptr::null_mut());
}

unsafe extern "C" fn draw_theme_bkg_H(di: *const ProgressDrawInfo, start: i32, end: i32) {
    let mut bgrect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut r: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    SetRect(
        core::ptr::addr_of_mut!(r),
        (*di).rect.left + start,
        (*di).rect.top,
        (*di).rect.left + end,
        (*di).rect.bottom,
    );
    bgrect = (*di).bgRect;
    OffsetRect(
        core::ptr::addr_of_mut!(bgrect),
        -bgrect.left,
        -bgrect.top,
    );

    DrawThemeBackground(
        (*di).theme,
        (*di).hdc,
        (PP_BAR) as i32,
        0,
        core::ptr::addr_of_mut!(bgrect),
        core::ptr::addr_of_mut!(r),
    );
}

unsafe extern "C" fn draw_theme_bkg_V(di: *const ProgressDrawInfo, start: ::core::ffi::c_int, end: ::core::ffi::c_int) {
    let mut bgrect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut r: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    SetRect(
        &raw mut r,
        (*di).rect.left,
        (*di).rect.bottom - end,
        (*di).rect.right,
        (*di).rect.bottom - start,
    );
    bgrect = (*di).bgRect;
    OffsetRect(
        &raw mut bgrect,
        -bgrect.left,
        -bgrect.top,
    );
    DrawThemeBackground(
        (*di).theme,
        (*di).hdc,
        (PP_BARVERT) as i32,
        0,
        &raw mut bgrect,
        &raw mut r,
    );
}

unsafe extern "C" fn PROGRESS_Draw(infoPtr: *mut PROGRESS_INFO, hdc: HDC) -> LRESULT {
    let mut barSize: ::core::ffi::c_int = 0;
    let mut dwStyle: DWORD = 0;
    let mut barSmooth: BOOL = 0;
    let mut drawProcs: *const ProgressDrawProc;
    let mut pdi: ProgressDrawInfo = core::mem::zeroed();

    pdi.hdc = hdc;
    pdi.theme = GetWindowTheme((*infoPtr).Self_);

    /* get the required bar brush */
    if (*infoPtr).ColorBar == CLR_DEFAULT {
        pdi.hbrBar = GetSysColorBrush((COLOR_HIGHLIGHT) as i32);
    } else {
        pdi.hbrBar = CreateSolidBrush((*infoPtr).ColorBar);
    }

    if (*infoPtr).ColorBk == CLR_DEFAULT {
        pdi.hbrBk = GetSysColorBrush((COLOR_3DFACE) as i32);
    } else {
        pdi.hbrBk = CreateSolidBrush((*infoPtr).ColorBk);
    }

    /* get the window style */
    dwStyle = GetWindowLongW((*infoPtr).Self_, GWL_STYLE) as DWORD;

    /* get client rectangle */
    GetClientRect((*infoPtr).Self_, core::ptr::addr_of_mut!(pdi.rect));
    if pdi.theme.is_null() {
        FrameRect(hdc, core::ptr::addr_of_mut!(pdi.rect), pdi.hbrBk);
        InflateRect(
            core::ptr::addr_of_mut!(pdi.rect),
            -1,
            -1,
        );
    } else {
        let mut cntRect: RECT = core::mem::zeroed();
        let part: ::core::ffi::c_int = if (dwStyle & PBS_VERTICAL) != 0 {
            (PP_BARVERT) as i32
        } else {
            (PP_BAR) as i32
        };

        GetThemeBackgroundContentRect(
            pdi.theme,
            hdc,
            part,
            0,
            core::ptr::addr_of_mut!(pdi.rect),
            core::ptr::addr_of_mut!(cntRect),
        );

        /* Exclude content rect - content background will be drawn later */
        ExcludeClipRect(
            hdc,
            cntRect.left as ::core::ffi::c_int,
            cntRect.top as ::core::ffi::c_int,
            cntRect.right as ::core::ffi::c_int,
            cntRect.bottom as ::core::ffi::c_int,
        );
        if IsThemeBackgroundPartiallyTransparent(pdi.theme, part, 0) != 0 {
            DrawThemeParentBackground(
                (*infoPtr).Self_,
                hdc,
                core::ptr::null_mut::<RECT>(),
            );
        }
        DrawThemeBackground(
            pdi.theme,
            hdc,
            part,
            0,
            core::ptr::addr_of_mut!(pdi.rect),
            core::ptr::null::<RECT>(),
        );
        SelectClipRgn(hdc, core::ptr::null_mut::<HRGN__>());
        pdi.rect = cntRect;
    }

    /* compute some drawing parameters */
    barSmooth = ((dwStyle & PBS_SMOOTH) != 0 && pdi.theme.is_null()) as ::core::ffi::c_int as BOOL;
    drawProcs = (if !pdi.theme.is_null() {
        core::ptr::addr_of!(drawProcThemed) as *const ProgressDrawProc
    } else {
        core::ptr::addr_of!(drawProcClassic) as *const ProgressDrawProc
    })
    .offset(
        ((if barSmooth != 0 {
            0
        } else {
            4
        }) + (if (dwStyle & PBS_VERTICAL) != 0 {
            2
        } else {
            0
        })) as isize,
    );
    barSize = get_bar_size(dwStyle as LONG, core::ptr::addr_of_mut!(pdi.rect));
    if !pdi.theme.is_null() {
        GetWindowRect((*infoPtr).Self_, core::ptr::addr_of_mut!(pdi.bgRect));
        MapWindowPoints(
            (*infoPtr).Self_,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(pdi.bgRect) as *mut POINT,
            2,
        );
    }

    if barSmooth == 0 {
        pdi.ledW = get_led_size(infoPtr, dwStyle as LONG, core::ptr::addr_of_mut!(pdi.rect));
    }
    pdi.ledGap = get_led_gap(infoPtr);

    if (dwStyle & PBS_MARQUEE) != 0 {
        let ledW: ::core::ffi::c_int = if barSmooth == 0 {
            pdi.ledW + pdi.ledGap
        } else {
            1
        };
        let leds: ::core::ffi::c_int = (barSize + ledW - 1) / ledW;
        let ledMEnd: ::core::ffi::c_int = (*infoPtr).MarqueePos + MARQUEE_LEDS;

        if ledMEnd > leds {
            /* case 1: the marquee bar extends over the end and wraps around to
             * the start */
            let gapStart: ::core::ffi::c_int = max((ledMEnd - leds) * ledW, 0);
            let gapEnd: ::core::ffi::c_int = min((*infoPtr).MarqueePos * ledW, barSize);

            ((*drawProcs.offset(0 as isize)).unwrap_unchecked())(
                core::ptr::addr_of_mut!(pdi),
                0,
                gapStart,
            );
            ((*drawProcs.offset(1 as isize)).unwrap_unchecked())(
                core::ptr::addr_of_mut!(pdi),
                gapStart,
                gapEnd,
            );
            ((*drawProcs.offset(0 as isize)).unwrap_unchecked())(
                core::ptr::addr_of_mut!(pdi),
                gapEnd,
                barSize,
            );
        } else {
            /* case 2: the marquee bar is between start and end */
            let barStart: ::core::ffi::c_int = (*infoPtr).MarqueePos * ledW;
            let barEnd: ::core::ffi::c_int = min(ledMEnd * ledW, barSize);

            ((*drawProcs.offset(1 as isize)).unwrap_unchecked())(
                core::ptr::addr_of_mut!(pdi),
                0,
                barStart,
            );
            ((*drawProcs.offset(0 as isize)).unwrap_unchecked())(
                core::ptr::addr_of_mut!(pdi),
                barStart,
                barEnd,
            );
            ((*drawProcs.offset(1 as isize)).unwrap_unchecked())(
                core::ptr::addr_of_mut!(pdi),
                barEnd,
                barSize,
            );
        }
    } else {
        let mut barEnd: ::core::ffi::c_int = get_bar_position(
            infoPtr,
            dwStyle as LONG,
            core::ptr::addr_of_mut!(pdi.rect),
            (*infoPtr).CurVal,
        );
        if barSmooth == 0 {
            let ledW: ::core::ffi::c_int = pdi.ledW + pdi.ledGap;
            barEnd = min(
                ((barEnd + ledW - 1) / ledW) * ledW,
                barSize,
            );
        }
        ((*drawProcs.offset(0 as isize)).unwrap_unchecked())(
            core::ptr::addr_of_mut!(pdi),
            0,
            barEnd,
        );
        ((*drawProcs.offset(1 as isize)).unwrap_unchecked())(
            core::ptr::addr_of_mut!(pdi),
            barEnd,
            barSize,
        );
    }

    /* delete bar brush */
    if (*infoPtr).ColorBar != CLR_DEFAULT {
        DeleteObject((pdi.hbrBar).cast());
    }
    if (*infoPtr).ColorBk != CLR_DEFAULT {
        DeleteObject((pdi.hbrBk).cast());
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn PROGRESS_Paint(infoPtr: *mut PROGRESS_INFO, mut hdc: HDC) -> LRESULT {
    let mut ps: PAINTSTRUCT = core::mem::zeroed();
    if !hdc.is_null() {
        return PROGRESS_Draw(infoPtr, hdc);
    }
    hdc = BeginPaint((*infoPtr).Self_, &mut ps);
    PROGRESS_Draw(infoPtr, hdc);
    EndPaint((*infoPtr).Self_, &mut ps);
    return 0 as LRESULT;
}

unsafe extern "C" fn PROGRESS_UpdateMarquee(infoPtr: *mut PROGRESS_INFO) {
    let mut style: LONG = GetWindowLongW((*infoPtr).Self_, GWL_STYLE);
    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut ledWidth: ::core::ffi::c_int = 0;
    let mut leds: ::core::ffi::c_int = 0;
    let mut theme: HTHEME = GetWindowTheme((*infoPtr).Self_);
    let mut smooth: BOOL = (((style & (PBS_SMOOTH as i32)) != 0) && theme.is_null()) as BOOL;

    get_client_rect((*infoPtr).Self_, core::ptr::addr_of_mut!(rect));

    if smooth != 0 {
        ledWidth = 1;
    } else {
        ledWidth = get_led_size(infoPtr, style, core::ptr::addr_of_mut!(rect))
            + get_led_gap(infoPtr);
    }

    leds = (get_bar_size(style, core::ptr::addr_of_mut!(rect)) + ledWidth
        - 1)
        / ledWidth;

    /* increment the marquee progress */
    (*infoPtr).MarqueePos += 1;
    if (*infoPtr).MarqueePos >= leds {
        (*infoPtr).MarqueePos = 0;
    }

    InvalidateRect(
        (*infoPtr).Self_,
        core::ptr::addr_of_mut!(rect) as LPCRECT,
        TRUE,
    );
    UpdateWindow((*infoPtr).Self_);
}

unsafe extern "C" fn PROGRESS_CoercePos(infoPtr: *mut PROGRESS_INFO) {
    if (*infoPtr).CurVal < (*infoPtr).MinVal {
        (*infoPtr).CurVal = (*infoPtr).MinVal;
    }
    if (*infoPtr).CurVal > (*infoPtr).MaxVal {
        (*infoPtr).CurVal = (*infoPtr).MaxVal;
    }
}

unsafe extern "C" fn PROGRESS_SetFont(infoPtr: *mut PROGRESS_INFO, hFont: HFONT, bRedraw: BOOL) -> HFONT {
    let mut hOldFont: HFONT = (*infoPtr).Font;
    (*infoPtr).Font = hFont;
    /* Since infoPtr->Font is not used, there is no need for repaint */
    return hOldFont;
}

unsafe extern "C" fn PROGRESS_SetRange(mut infoPtr: *mut PROGRESS_INFO, mut low: INT, mut high: INT) -> DWORD {
    let mut res: DWORD = MAKELONG!(LOWORD!((*infoPtr).MinVal as DWORD), LOWORD!((*infoPtr).MaxVal as DWORD)) as DWORD;

    /* if nothing changes, simply return */
    if (*infoPtr).MinVal == low && (*infoPtr).MaxVal == high {
        return res;
    }

    (*infoPtr).MinVal = low;
    (*infoPtr).MaxVal = high;
    PROGRESS_CoercePos(infoPtr);
    InvalidateRect((*infoPtr).Self_, core::ptr::null_mut(), TRUE);
    return res;
}

unsafe extern "C" fn PROGRESS_SetPos(infoPtr: *mut PROGRESS_INFO, pos: INT) -> UINT {
    let mut style: DWORD = GetWindowLongW((*infoPtr).Self_, GWL_STYLE) as DWORD;
    if (style & PBS_MARQUEE as DWORD) != 0 {
        PROGRESS_UpdateMarquee(infoPtr);
        return 1 as UINT;
    } else {
        let mut oldVal: UINT = 0;
        oldVal = (*infoPtr).CurVal as UINT;
        if oldVal != pos as UINT {
            (*infoPtr).CurVal = pos;
            PROGRESS_CoercePos(infoPtr);
            if ((*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"dll/win32/comctl32/progress.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"PROGRESS_SetPos\0" as *const u8 as *const ::core::ffi::c_char,
                    517 as ::core::ffi::c_int,
                    b"PBM_SETPOS: current pos changed from %d to %d\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    oldVal,
                    (*infoPtr).CurVal,
                );
            }
            PROGRESS_Invalidate(infoPtr, oldVal as INT, (*infoPtr).CurVal);
            UpdateWindow((*infoPtr).Self_);
        }
        return oldVal;
    }
}
unsafe extern "system" fn ProgressWindowProc(
    mut hwnd: HWND,
    mut message: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: *mut PROGRESS_INFO = ::core::ptr::null_mut::<PROGRESS_INFO>();
    static themeClass: [u16; 9] = [0x0050u16, 0x0072, 0x006F, 0x0067, 0x0072, 0x0065, 0x0073, 0x0073, 0x0000];
    let mut theme: HTHEME = ::core::ptr::null_mut::<::core::ffi::c_void>();

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/progress.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ProgressWindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            535 as ::core::ffi::c_int,
            b"hwnd=%p msg=%04x wparam=%lx lParam=%lx\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hwnd,
            message,
            wParam,
            lParam,
        );
    }

    infoPtr = GetWindowLongPtrW(hwnd, 0 as ::core::ffi::c_int)
        as *mut PROGRESS_INFO;

    if infoPtr.is_null() && message != WM_CREATE as UINT {
        return DefWindowProcW(hwnd, message, wParam, lParam);
    }

    match message {
        WM_CREATE => {
            let mut dwExStyle: DWORD =
                GetWindowLongW(hwnd, GWL_EXSTYLE as ::core::ffi::c_int)
                    as DWORD;

            theme = OpenThemeData(hwnd, themeClass.as_ptr() as LPCWSTR);

            dwExStyle &= !(WS_EX_CLIENTEDGE | WS_EX_WINDOWEDGE) as DWORD;
            if theme.is_null() {
                dwExStyle |= WS_EX_STATICEDGE as DWORD;
            }
            SetWindowLongW(
                hwnd,
                GWL_EXSTYLE as ::core::ffi::c_int,
                dwExStyle as LONG,
            );
            /* Force recalculation of a non-client area */
            SetWindowPos(
                hwnd,
                0 as usize as HWND,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (SWP_FRAMECHANGED
                    | SWP_NOSIZE
                    | SWP_NOMOVE
                    | SWP_NOZORDER
                    | SWP_NOACTIVATE) as UINT,
            );

            /* allocate memory for info struct */
            infoPtr = heap_alloc_zero(::core::mem::size_of::<PROGRESS_INFO>() as SIZE_T)
                as *mut PROGRESS_INFO;
            if infoPtr.is_null() {
                return -1 as LRESULT;
            }
            SetWindowLongPtrW(
                hwnd,
                0 as ::core::ffi::c_int,
                infoPtr as DWORD_PTR as LONG_PTR,
            );

            /* initialize the info struct */
            (*infoPtr).Self_ = hwnd;
            (*infoPtr).MinVal = 0 as INT;
            (*infoPtr).MaxVal = 100 as INT;
            (*infoPtr).CurVal = 0 as INT;
            (*infoPtr).Step = 10 as INT;
            (*infoPtr).MarqueePos = 0 as INT;
            (*infoPtr).Marquee = FALSE as BOOL;
            (*infoPtr).ColorBar = CLR_DEFAULT as COLORREF;
            (*infoPtr).ColorBk = CLR_DEFAULT as COLORREF;
            (*infoPtr).Font = 0 as usize as HFONT;

            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"dll/win32/comctl32/progress.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"ProgressWindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                    573 as ::core::ffi::c_int,
                    b"Progress Ctrl creation, hwnd=%p\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    hwnd,
                );
            }
            return 0 as LRESULT;
        }

        WM_DESTROY => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"dll/win32/comctl32/progress.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"ProgressWindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                    578 as ::core::ffi::c_int,
                    b"Progress Ctrl destruction, hwnd=%p\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    hwnd,
                );
            }
            heap_free(infoPtr as *mut ::core::ffi::c_void);
            SetWindowLongPtrW(
                hwnd,
                0 as ::core::ffi::c_int,
                0 as LONG_PTR,
            );
            theme = GetWindowTheme(hwnd);
            CloseThemeData(theme);
            return 0 as LRESULT;
        }

        WM_ERASEBKGND => return 1 as LRESULT,

        WM_GETFONT => return (*infoPtr).Font as LRESULT,

        WM_SETFONT => {
            return PROGRESS_SetFont(infoPtr, wParam as HFONT, lParam as BOOL) as LRESULT;
        }

        WM_PRINTCLIENT | WM_PAINT => {
            return PROGRESS_Paint(infoPtr, wParam as HDC);
        }

        WM_TIMER => {
            if wParam == ID_MARQUEE_TIMER as WPARAM {
                PROGRESS_UpdateMarquee(infoPtr);
            }
            return 0 as LRESULT;
        }

        WM_THEMECHANGED => {
            let mut dwExStyle_0: DWORD =
                GetWindowLongW(hwnd, GWL_EXSTYLE as ::core::ffi::c_int)
                    as DWORD;

            theme = GetWindowTheme(hwnd);
            CloseThemeData(theme);
            theme = OpenThemeData(hwnd, themeClass.as_ptr() as LPCWSTR);

            /* WS_EX_STATICEDGE disappears when the control is themed */
            if !theme.is_null() {
                dwExStyle_0 &= !WS_EX_STATICEDGE as DWORD;
            } else {
                dwExStyle_0 |= WS_EX_STATICEDGE as DWORD;
            }
            SetWindowLongW(
                hwnd,
                GWL_EXSTYLE as ::core::ffi::c_int,
                dwExStyle_0 as LONG,
            );

            InvalidateRect(
                hwnd,
                ::core::ptr::null::<RECT>(),
                FALSE as BOOL,
            );
            return 0 as LRESULT;
        }

        PBM_DELTAPOS => {
            let mut oldVal: INT = 0;
            oldVal = (*infoPtr).CurVal;
            if wParam != 0 as WPARAM {
                (*infoPtr).CurVal += wParam as INT;
                PROGRESS_CoercePos(infoPtr);
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_2,
                        __dbch_2,
                        b"dll/win32/comctl32/progress.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"ProgressWindowProc\0" as *const u8
                            as *const ::core::ffi::c_char,
                        629 as ::core::ffi::c_int,
                        b"PBM_DELTAPOS: current pos changed from %d to %d\n\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        oldVal,
                        (*infoPtr).CurVal,
                    );
                }
                PROGRESS_Invalidate(infoPtr, oldVal, (*infoPtr).CurVal);
                UpdateWindow((*infoPtr).Self_);
            }
            return oldVal as LRESULT;
        }

        PBM_SETPOS => {
            return PROGRESS_SetPos(infoPtr, wParam as INT) as LRESULT;
        }

        PBM_SETRANGE => {
            return PROGRESS_SetRange(
                infoPtr,
                LOWORD!(lParam as DWORD_PTR) as ::core::ffi::c_int,
                HIWORD!(lParam as DWORD_PTR) as ::core::ffi::c_int,
            ) as LRESULT;
        }

        PBM_SETSTEP => {
            let mut oldStep: INT = 0;
            oldStep = (*infoPtr).Step;
            (*infoPtr).Step = wParam as INT;
            return oldStep as LRESULT;
        }

        PBM_GETSTEP => return (*infoPtr).Step as LRESULT,

        PBM_STEPIT => {
            let mut oldVal_0: ::core::ffi::c_int = (*infoPtr).CurVal as ::core::ffi::c_int;

            if (*infoPtr).MinVal != (*infoPtr).MaxVal {
                (*infoPtr).CurVal += (*infoPtr).Step;
                if (*infoPtr).CurVal > (*infoPtr).MaxVal {
                    (*infoPtr).CurVal = ((*infoPtr).CurVal - (*infoPtr).MinVal)
                        % ((*infoPtr).MaxVal - (*infoPtr).MinVal)
                        + (*infoPtr).MinVal;
                }
                if (*infoPtr).CurVal < (*infoPtr).MinVal {
                    (*infoPtr).CurVal = ((*infoPtr).CurVal - (*infoPtr).MinVal)
                        % ((*infoPtr).MaxVal - (*infoPtr).MinVal)
                        + (*infoPtr).MaxVal;
                }

                if oldVal_0 != (*infoPtr).CurVal {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int
                        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                        != 0
                    {
                        let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                        let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
                        ros_dbg_log(
                            __dbcl_3,
                            __dbch_3,
                            b"dll/win32/comctl32/progress.c\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            b"ProgressWindowProc\0" as *const u8
                                as *const ::core::ffi::c_char,
                            667 as ::core::ffi::c_int,
                            b"PBM_STEPIT: current pos changed from %d to %d\n\0"
                                as *const u8 as *const ::core::ffi::c_char,
                            oldVal_0,
                            (*infoPtr).CurVal,
                        );
                    }
                    PROGRESS_Invalidate(infoPtr, oldVal_0 as INT, (*infoPtr).CurVal);
                    UpdateWindow((*infoPtr).Self_);
                }
            }

            return oldVal_0 as LRESULT;
        }

        PBM_SETRANGE32 => {
            return PROGRESS_SetRange(
                infoPtr,
                wParam as ::core::ffi::c_int,
                lParam as ::core::ffi::c_int,
            ) as LRESULT;
        }

        PBM_GETRANGE => {
            if lParam != 0 {
                (*(lParam as PPBRANGE)).iLow = (*infoPtr).MinVal as ::core::ffi::c_int;
                (*(lParam as PPBRANGE)).iHigh = (*infoPtr).MaxVal as ::core::ffi::c_int;
            }
            return (if wParam != 0 {
                (*infoPtr).MinVal
            } else {
                (*infoPtr).MaxVal
            }) as LRESULT;
        }

        PBM_GETPOS => return (*infoPtr).CurVal as LRESULT,

        PBM_SETBARCOLOR => {
            let mut clr: COLORREF = (*infoPtr).ColorBar;

            (*infoPtr).ColorBar = lParam as COLORREF;
            InvalidateRect(
                hwnd,
                ::core::ptr::null::<RECT>(),
                TRUE as BOOL,
            );
            return clr as LRESULT;
        }

        PBM_GETBARCOLOR => return (*infoPtr).ColorBar as LRESULT,

        PBM_SETBKCOLOR => {
            let mut clr_0: COLORREF = (*infoPtr).ColorBk;

            (*infoPtr).ColorBk = lParam as COLORREF;
            InvalidateRect(
                hwnd,
                ::core::ptr::null::<RECT>(),
                TRUE as BOOL,
            );
            return clr_0 as LRESULT;
        }

        PBM_GETBKCOLOR => return (*infoPtr).ColorBk as LRESULT,

        PBM_SETSTATE => {
            if wParam != PBST_NORMAL as WPARAM {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_4: __wine_debug_class = __WINE_DBCL_FIXME;
                    ros_dbg_log(
                        __dbcl_4,
                        __dbch_4,
                        b"dll/win32/comctl32/progress.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"ProgressWindowProc\0" as *const u8
                            as *const ::core::ffi::c_char,
                        715 as ::core::ffi::c_int,
                        b"state %04lx not yet handled\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        wParam,
                    );
                }
            }
            return PBST_NORMAL as LRESULT;
        }

        PBM_GETSTATE => return PBST_NORMAL as LRESULT,

        PBM_SETMARQUEE => {
            if wParam != 0 as WPARAM {
                let mut period: UINT = if lParam != 0 {
                    lParam as UINT
                } else {
                    DEFAULT_MARQUEE_PERIOD as UINT
                };
                (*infoPtr).Marquee = TRUE as BOOL;
                SetTimer(
                    (*infoPtr).Self_,
                    ID_MARQUEE_TIMER as UINT_PTR,
                    period,
                    None,
                );
            } else {
                (*infoPtr).Marquee = FALSE as BOOL;
                KillTimer(
                    (*infoPtr).Self_,
                    ID_MARQUEE_TIMER as UINT_PTR,
                );
            }
            return (*infoPtr).Marquee as LRESULT;
        }

        _ => {
            if (message >= WM_USER as UINT)
                && (message < WM_APP as UINT)
                && COMCTL32_IsReflectedMessage(message) == 0
            {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_5: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_5: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_5,
                        __dbch_5,
                        b"dll/win32/comctl32/progress.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"ProgressWindowProc\0" as *const u8
                            as *const ::core::ffi::c_char,
                        737 as ::core::ffi::c_int,
                        b"unknown msg %04x wp=%04lx lp=%08lx\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        message,
                        wParam,
                        lParam,
                    );
                }
            }
            return DefWindowProcW(hwnd, message, wParam, lParam);
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn PROGRESS_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(wndClass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>(),
    );
    wndClass.style = (CS_GLOBALCLASS | CS_VREDRAW | CS_HREDRAW) as UINT;
    wndClass.lpfnWndProc = Some(ProgressWindowProc);
    wndClass.cbClsExtra = 0 as core::ffi::c_int;
    wndClass.cbWndExtra =
        core::mem::size_of::<*mut PROGRESS_INFO>() as core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as LPWSTR,
    );
    wndClass.lpszClassName = (PROGRESS_CLASSW).as_ptr();

    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn PROGRESS_Unregister() {
    UnregisterClassW((PROGRESS_CLASSW).as_ptr(), core::ptr::null_mut());
}
