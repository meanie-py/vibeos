use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----


unsafe extern "C" fn emfdrv_update_bounds(dc: *mut WINEDC, rect: *mut RECTL) {
    let bounds: *mut RECTL = core::ptr::addr_of_mut!((*dc).emf_bounds);
    let mut vport_rect: RECTL = *rect;

    //lp_to_dp( dc, (POINT *)&vport_rect, 2 );
    LPtoDP((*dc).hdc, core::ptr::addr_of_mut!(vport_rect) as *mut POINT, 2);

    /* The coordinate systems may be mirrored
       (LPtoDP handles points, not rectangles) */
    if vport_rect.left > vport_rect.right {
        let temp: LONG = vport_rect.right;
        vport_rect.right = vport_rect.left;
        vport_rect.left = temp;
    }
    if vport_rect.top > vport_rect.bottom {
        let temp: LONG = vport_rect.bottom;
        vport_rect.bottom = vport_rect.top;
        vport_rect.top = temp;
    }

    if (*bounds).left > (*bounds).right {
        /* first bounding rectangle */
        *bounds = vport_rect;
    } else {
        (*bounds).left = min((*bounds).left, vport_rect.left);
        (*bounds).top = min((*bounds).top, vport_rect.top);
        (*bounds).right = max((*bounds).right, vport_rect.right);
        (*bounds).bottom = max((*bounds).bottom, vport_rect.bottom);
    }
}
#[no_mangle]
pub unsafe extern "C" fn EMFDRV_LineTo(dc: *mut WINEDC, x: INT, y: INT) -> BOOL {
    let mut bounds: RECTL = core::mem::zeroed();
    let mut pt: POINT = core::mem::zeroed();

    //pt = dc->attr->cur_pos;
    GetCurrentPositionEx((*dc).hdc, core::ptr::addr_of_mut!(pt));

    bounds.left = min(x, pt.x);
    bounds.top = min(y, pt.y);
    bounds.right = max(x, pt.x);
    bounds.bottom = max(y, pt.y);
    emfdrv_update_bounds(dc, core::ptr::addr_of_mut!(bounds));
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDRV_RoundRect(
    dc: *mut WINEDC,
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
    ell_width: INT,
    ell_height: INT,
) -> BOOL {
    let mut bounds: RECTL = RECTL {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if left == right || top == bottom {
        return (FALSE) as BOOL;
    }
    bounds.left = min(left, right) as LONG;
    bounds.top = min(top, bottom) as LONG;
    bounds.right = max(left, right) as LONG;
    bounds.bottom = max(top, bottom) as LONG;
    if GetGraphicsMode((*dc).hdc) == (GM_COMPATIBLE as i32) {
        bounds.right -= 1;
        bounds.bottom -= 1;
    }
    emfdrv_update_bounds(dc, core::ptr::addr_of_mut!(bounds));
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDRV_ArcChordPie(
    dc: *mut WINEDC,
    mut left: INT,
    mut top: INT,
    mut right: INT,
    mut bottom: INT,
    mut xstart: INT,
    mut ystart: INT,
    mut xend: INT,
    mut yend: INT,
    type_0: DWORD,
) -> BOOL {
    let mut temp: INT = 0;
    let mut x_centre: INT = 0;
    let mut y_centre: INT = 0;
    let mut i: INT = 0;
    let mut angle_start: f64 = 0.0;
    let mut angle_end: f64 = 0.0;
    let mut xinter_start: f64 = 0.0;
    let mut yinter_start: f64 = 0.0;
    let mut xinter_end: f64 = 0.0;
    let mut yinter_end: f64 = 0.0;
    let mut emr: EMRARC = core::mem::zeroed();
    let mut bounds: RECTL = core::mem::zeroed();

    if left == right || top == bottom {
        return (FALSE) as BOOL;
    }

    if left > right {
        temp = left;
        left = right;
        right = temp;
    }
    if top > bottom {
        temp = top;
        top = bottom;
        bottom = temp;
    }

    if GetGraphicsMode((*dc).hdc) == GM_COMPATIBLE as _ {
        right -= 1;
        bottom -= 1;
    }

    emr.emr.iType = type_0;
    emr.emr.nSize = core::mem::size_of::<EMRARC>() as DWORD;
    emr.rclBox.left = left as LONG;
    emr.rclBox.top = top as LONG;
    emr.rclBox.right = right as LONG;
    emr.rclBox.bottom = bottom as LONG;
    emr.ptlStart.x = xstart as LONG;
    emr.ptlStart.y = ystart as LONG;
    emr.ptlEnd.x = xend as LONG;
    emr.ptlEnd.y = yend as LONG;

    /* Now calculate the BBox */
    x_centre = (left + right + 1) / 2;
    y_centre = (top + bottom + 1) / 2;

    xstart -= x_centre;
    ystart -= y_centre;
    xend -= x_centre;
    yend -= y_centre;

    /* invert y co-ords to get angle anti-clockwise from x-axis */
    angle_start = atan2(-(ystart as f64), xstart as f64);
    angle_end = atan2(-(yend as f64), xend as f64);

    /* These are the intercepts of the start/end lines with the arc */
    xinter_start = ((right - left + 1) / 2) as f64
        * cos(angle_start)
        + x_centre as f64;
    yinter_start = (-(bottom - top + 1) / 2) as f64
        * sin(angle_start)
        + y_centre as f64;
    xinter_end = ((right - left + 1) / 2) as f64
        * cos(angle_end)
        + x_centre as f64;
    yinter_end = (-(bottom - top + 1) / 2) as f64
        * sin(angle_end)
        + y_centre as f64;

    if angle_start < 0 as f64 {
        angle_start += 2 as f64 * M_PI;
    }
    if angle_end < 0 as f64 {
        angle_end += 2 as f64 * M_PI;
    }
    if angle_end < angle_start {
        angle_end += 2 as f64 * M_PI;
    }

    bounds.left = min(xinter_start, xinter_end) as LONG;
    bounds.top = min(yinter_start, yinter_end) as LONG;
    bounds.right = max(xinter_start, xinter_end) as LONG;
    bounds.bottom = max(yinter_start, yinter_end) as LONG;

    i = 0;
    'outer: while i <= 8 {
        'body: {
            if (i as f64 * M_PI / 2 as f64) < angle_start {
                /* loop until we're past start */
                break 'body;
            }
            if (i as f64 * M_PI / 2 as f64) > angle_end {
                /* if we're past end we're finished */
                break 'outer;
            }

            /* the arc touches the rectangle at the start of quadrant i, so adjust
               BBox to reflect this. */

            match i % 4 {
                0 => {
                    bounds.right = right as LONG;
                }
                1 => {
                    bounds.top = top as LONG;
                }
                2 => {
                    bounds.left = left as LONG;
                }
                3 => {
                    bounds.bottom = bottom as LONG;
                }
                _ => {}
            }
        }
        i += 1;
    }

    /* If we're drawing a pie then make sure we include the centre */
    if type_0 == EMR_PIE as DWORD {
        if bounds.left > x_centre as LONG {
            bounds.left = x_centre as LONG;
        } else if bounds.right < x_centre as LONG {
            bounds.right = x_centre as LONG;
        }
        if bounds.top > y_centre as LONG {
            bounds.top = y_centre as LONG;
        } else if bounds.bottom < y_centre as LONG {
            bounds.bottom = y_centre as LONG;
        }
    } else if type_0 == EMR_ARCTO as DWORD {
        let mut pt: POINT = core::mem::zeroed();
        //pt = dc->attr->cur_pos;
        GetCurrentPositionEx((*dc).hdc, core::ptr::addr_of_mut!(pt));
        bounds.left = min(bounds.left, pt.x);
        bounds.top = min(bounds.top, pt.y);
        bounds.right = max(bounds.right, pt.x);
        bounds.bottom = max(bounds.bottom, pt.y);
    }
    emfdrv_update_bounds(dc, core::ptr::addr_of_mut!(bounds));
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDRV_Arc(dc: *mut WINEDC, left: INT, top: INT, right: INT, bottom: INT, xstart: INT, ystart: INT, xend: INT, yend: INT) -> BOOL {
    return EMFDRV_ArcChordPie(dc, left, top, right, bottom, xstart, ystart, xend, yend, EMR_ARC as DWORD);
}
#[no_mangle]
pub unsafe extern "C" fn EMFDRV_ArcTo(dc: *mut WINEDC, left: INT, top: INT, right: INT, bottom: INT, xstart: INT, ystart: INT, xend: INT, yend: INT) -> BOOL {
    return EMFDRV_ArcChordPie(dc, left, top, right, bottom, xstart, ystart, xend, yend, EMR_ARCTO);
}
#[no_mangle]
pub unsafe extern "C" fn EMFDRV_Pie(
    dc: *mut WINEDC,
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
    xstart: INT,
    ystart: INT,
    xend: INT,
    yend: INT,
) -> BOOL {
    return EMFDRV_ArcChordPie(
        dc,
        left,
        top,
        right,
        bottom,
        xstart,
        ystart,
        xend,
        yend,
        EMR_PIE as DWORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn EMFDRV_Chord(
    dc: *mut WINEDC,
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
    xstart: INT,
    ystart: INT,
    xend: INT,
    yend: INT,
) -> BOOL {
    return EMFDRV_ArcChordPie(
        dc,
        left,
        top,
        right,
        bottom,
        xstart,
        ystart,
        xend,
        yend,
        EMR_CHORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn EMFDRV_Ellipse(
    dc: *mut WINEDC,
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
) -> BOOL {
    let mut bounds: RECTL = RECTL {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    if left == right || top == bottom {
        return (FALSE) as BOOL;
    }

    bounds.left = min(left, right) as LONG;
    bounds.top = min(top, bottom) as LONG;
    bounds.right = max(left, right) as LONG;
    bounds.bottom = max(top, bottom) as LONG;
    if GetGraphicsMode((*dc).hdc) == (GM_COMPATIBLE as i32) {
        bounds.right -= 1;
        bounds.bottom -= 1;
    }

    emfdrv_update_bounds(dc, core::ptr::addr_of_mut!(bounds));
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDRV_Rectangle(
    dc: *mut WINEDC,
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
) -> BOOL {
    let mut bounds: RECTL = RECTL {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    if left == right || top == bottom {
        return (FALSE) as BOOL;
    }

    bounds.left = min(left, right) as LONG;
    bounds.top = min(top, bottom) as LONG;
    bounds.right = max(left, right) as LONG;
    bounds.bottom = max(top, bottom) as LONG;
    if GetGraphicsMode((*dc).hdc) == (GM_COMPATIBLE as i32) {
        bounds.right -= 1;
        bounds.bottom -= 1;
    }

    emfdrv_update_bounds(dc, core::ptr::addr_of_mut!(bounds));
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDRV_SetPixel(dc: *mut WINEDC, x: INT, y: INT, color: COLORREF) -> COLORREF {
    let mut bounds: RECTL = RECTL {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    bounds.right = x as LONG;
    bounds.left = bounds.right;
    bounds.bottom = y as LONG;
    bounds.top = bounds.bottom;
    emfdrv_update_bounds(dc, core::ptr::addr_of_mut!(bounds));
    return CLR_INVALID;
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_PolylineTo(
    dc: *mut WINEDC,
    pt: *const POINT,
    count: INT,
) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_PolyBezier(
    dc: *mut WINEDC,
    pts: *const POINT,
    count: DWORD,
) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_PolyBezierTo(
    dc: *mut WINEDC,
    pts: *const POINT,
    count: DWORD,
) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_PolyPolyline(
    dc: *mut WINEDC,
    pt: *const POINT,
    counts: *const DWORD,
    polys: UINT,
) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_PolyPolygon(
    dc: *mut WINEDC,
    pt: *const POINT,
    counts: *const INT,
    polys: UINT,
) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_PolyDraw(
    dc: *mut WINEDC,
    pts: *const POINT,
    types: *const BYTE,
    count: DWORD,
) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_FillRgn(dc: *mut WINEDC, hrgn: HRGN, hbrush: HBRUSH) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_FrameRgn(
    dc: *mut WINEDC,
    hrgn: HRGN,
    hbrush: HBRUSH,
    width: INT,
    height: INT,
) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_InvertRgn(dc: *mut WINEDC, hrgn: HRGN) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_ExtTextOut(
    dc: *mut WINEDC,
    x: INT,
    y: INT,
    flags: UINT,
    lprect: *const RECT,
    str: LPCWSTR,
    count: UINT,
    lpDx: *const INT,
) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_GradientFill(
    dc: *mut WINEDC,
    vert_array: *mut TRIVERTEX,
    nvert: ULONG,
    grad_array: *mut core::ffi::c_void,
    ngrad: ULONG,
    mode: ULONG,
) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_FillPath(dc: *mut WINEDC) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_StrokeAndFillPath(dc: *mut WINEDC) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_StrokePath(dc: *mut WINEDC) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_AlphaBlend(
    dc_dst: *mut WINEDC,
    x_dst: INT,
    y_dst: INT,
    width_dst: INT,
    height_dst: INT,
    dc_src: HDC,
    x_src: INT,
    y_src: INT,
    width_src: INT,
    height_src: INT,
    func: BLENDFUNCTION,
) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_PatBlt(
    dc: *mut WINEDC,
    left: INT,
    top: INT,
    width: INT,
    height: INT,
    rop: DWORD,
) -> BOOL {
    (TRUE) as i32
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_StretchDIBits(
    dc: *mut WINEDC,
    x_dst: INT,
    y_dst: INT,
    width_dst: INT,
    height_dst: INT,
    x_src: INT,
    y_src: INT,
    width_src: INT,
    height_src: INT,
    bits: *const core::ffi::c_void,
    info: *mut BITMAPINFO,
    wUsage: UINT,
    dwRop: DWORD,
) -> INT {
    height_src
}
#[no_mangle]
pub unsafe extern "C" fn EMFDRV_SetDIBitsToDevice(
    dc: *mut WINEDC,
    x_dst: INT,
    y_dst: INT,
    width: DWORD,
    height: DWORD,
    x_src: INT,
    y_src: INT,
    startscan: UINT,
    lines: UINT,
    bits: *const core::ffi::c_void,
    info: *mut BITMAPINFO,
    usage: UINT,
) -> INT {
    /* FIXME: Update bound rect */
    return lines as INT;
}

#[no_mangle]
pub unsafe extern "C" fn EMFDRV_SelectBitmap(dc: *mut WINEDC, hbitmap: HBITMAP) -> HBITMAP {
    core::ptr::null_mut()
}
