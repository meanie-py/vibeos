use win32k_sys::win32k::*;
use crate::brushbody::pbrushbody;

// CRT math (cdecl).
extern "C" {
    fn tan(x: f64) -> f64;
    fn ceil(x: f64) -> f64;
    fn abs(x: i32) -> i32;
}

// BRUSH_ShareLockBrush / BRUSH_ShareUnlockBrush are macros (brush.h) over the
// real GDIOBJ_ShareLockObj (NTAPI) / GDIOBJ_vDereferenceObject — inline them.
#[inline(always)]
unsafe fn BRUSH_ShareLockBrush(hbrush: HBRUSH) -> PBRUSH {
    GDIOBJ_ShareLockObj(hbrush as HGDIOBJ, GDI_OBJECT_TYPE_BRUSH as DWORD) as PBRUSH
}
#[inline(always)]
unsafe fn BRUSH_ShareUnlockBrush(pbrush: PBRUSH) {
    GDIOBJ_vDereferenceObject(pbrush as POBJ);
}
// pt(x,y) = app_new_point(x,y)  [drawing.c macro]
#[inline(always)]
unsafe fn pt(x: i32, y: i32) -> POINT { app_new_point(x, y) }
// ROP4_FROM_INDEX(i) = (i) | ((i) << 8)  [inteng.h]
#[inline(always)]
fn ROP4_FROM_INDEX(index: u32) -> u32 { index | (index << 8) }
// DEGREES_TO_RADIANS(deg) = (deg)*2*M_PI/360  [drawing.c macro]
const M_PI: f64 = core::f64::consts::PI;
// Macro (not fn): a Rust fn returning f64 by value hits the SIMD-f64 ABI error on no-SSE i386.
#[allow(unused_macros)]
macro_rules! DEGREES_TO_RADIANS { ($deg:expr) => { ($deg as f64) * 2.0 * M_PI / 360.0 }; }
// ASSERT_DC_PREPARED — debug assert; no-op.
#[inline(always)]
unsafe fn ASSERT_DC_PREPARED(_dc: *mut DC) {}

// ---- file-scope preamble (translated) ----
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub type Rect = _Rect;
pub type PRect = *mut _Rect;

unsafe fn app_new_point(x: i32, y: i32) -> POINT {
    let mut p: POINT = core::mem::zeroed();
    p.x = x;
    p.y = y;
    p
}

unsafe fn rect(x: i32, y: i32, width: i32, height: i32) -> Rect {
    let mut r: Rect = core::mem::zeroed();
    r.x = x;
    r.y = y;
    r.width = width;
    r.height = height;
    r
}

unsafe fn app_draw_ellipse(g: *mut DC, r: Rect, pbrush: PBRUSH) -> i32 {
    let mut a = r.width / 2;
    let mut b = r.height / 2;
    let mut x = 0;
    let mut y = b;
    let mut a2 = a as i64 * a as i64;
    let mut b2 = b as i64 * b as i64;
    let mut xcrit = (3 * a2 / 4) + 1;
    let mut ycrit = (3 * b2 / 4) + 1;
    let mut t = b2 + a2 - 2 * a2 * b as i64;
    let mut dxt = b2 * (3 + x + x) as i64;
    let mut dyt = a2 * (3 - y - y) as i64;
    let mut d2xt = b2 + b2;
    let mut d2yt = a2 + a2;

    let w = (*pbrushbody(pbrush)).lWidth;

    let mut A = if a - w > 0 { a - w } else { 0 };
    let mut B = if b - w > 0 { b - w } else { 0 };
    let mut X = 0;
    let mut Y = B;
    let mut A2 = A as i64 * A as i64;
    let mut B2 = B as i64 * B as i64;
    let mut XCRIT = (3 * A2 / 4) + 1;
    let mut YCRIT = (3 * B2 / 4) + 1;
    let mut T = B2 + A2 - 2 * A2 * B as i64;
    let mut DXT = B2 * (3 + X + X) as i64;
    let mut DYT = A2 * (3 - Y - Y) as i64;
    let mut D2XT = B2 + B2;
    let mut D2YT = A2 + A2;

    let mut movedown: i32;
    let mut moveout: i32;
    let mut innerX = 0;
    let mut prevx: i32;
    let mut prevy: i32;
    let mut W: i32;
    let mut r1: Rect = core::mem::zeroed();
    let mut r2: Rect;
    let mut result = 1;

    if (r.width <= 2) || (r.height <= 2) {
        return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, 1);
    }

    r1.x = r.x + a;
    r1.y = r.y;
    r1.width = r.width & 1;
    r1.height = 1;

    r2 = r1;
    r2.y = r.y + r.height - 1;

    prevx = r1.x;
    prevy = r1.y;

    while y > 0 {
        while Y == y {
            innerX = X;

            if T + A2 * (Y as i64) < XCRIT {
                X += 1;
                T += DXT;
                DXT += D2XT;
            } else if T - B2 * X as i64 >= YCRIT {
                Y -= 1;
                T += DYT;
                DYT += D2YT;
            } else {
                X += 1;
                Y -= 1;
                T += DXT + DYT;
                DXT += D2XT;
                DYT += D2YT;
            }
        }

        movedown = 0;
        moveout = 0;

        W = x - innerX;
        if r1.x + W < prevx {
            W = prevx - r1.x;
        }
        if W < w {
            W = w;
        }

        if t + a2 * (y as i64) < xcrit {
            x += 1;
            t += dxt;
            dxt += d2xt;

            moveout = 1;
        } else if t - b2 * x as i64 >= ycrit {
            y -= 1;
            t += dyt;
            dyt += d2yt;

            movedown = 1;
        } else {
            x += 1;
            y -= 1;
            t += dxt + dyt;
            dxt += d2xt;
            dyt += d2yt;

            movedown = 1;
            moveout = 1;
        }

        if movedown != 0 {
            if r1.width == 0 {
                r1.x -= 1;
                r1.width += 2;
                r2.x -= 1;
                r2.width += 2;
                moveout = 0;
            }

            if r1.x < r.x {
                r1.x = r.x;
                r2.x = r.x;
            }
            if r1.width > r.width {
                r1.width = r.width;
                r2.width = r.width;
            }
            if r1.y == r2.y - 1 {
                r1.x = r.x;
                r2.x = r.x;
                r1.width = r.width;
                r2.width = r.width;
            }

            if (r1.y < r.y + w) || (r1.x + W >= r1.x + r1.width - W) {
                result &= IntFillRect(g, r1.x, r1.y, r1.width, r1.height, pbrush, 1);
                result &= IntFillRect(g, r2.x, r2.y, r2.width, r2.height, pbrush, 1);

                prevx = r1.x;
                prevy = r1.y;
            } else if r1.y + r1.height < r2.y {
                result &= IntFillRect(g, r1.x, r1.y, W, 1, pbrush, 1);
                result &= IntFillRect(g, r1.x + r1.width - W, r1.y, W, 1, pbrush, 1);
                result &= IntFillRect(g, r2.x, r2.y, W, 1, pbrush, 1);
                result &= IntFillRect(g, r2.x + r2.width - W, r2.y, W, 1, pbrush, 1);

                prevx = r1.x;
                prevy = r1.y;
            }

            r1.y += 1;
            r2.y -= 1;
        }

        if moveout != 0 {
            r1.x -= 1;
            r1.width += 2;
            r2.x -= 1;
            r2.width += 2;
        }
    }

    if (x <= a) && (prevy < r2.y) {
        r1.height = r1.y + r1.height - r2.y;
        r1.y = r2.y;

        W = w;
        if r.x + W != prevx {
            W = prevx - r.x;
        }
        if W < w {
            W = w;
        }

        if W + W >= r.width {
            result &= IntFillRect(g, r.x, r1.y, r.width, r1.height, pbrush, 1);
            return result;
        }

        result &= IntFillRect(g, r.x, r1.y, W, r1.height, pbrush, 1);
        result &= IntFillRect(g, r.x + r.width - W, r1.y, W, r1.height, pbrush, 1);
    }

    result
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn app_fill_arc_rect(
    g: *mut DC,
    r: Rect,
    p0: POINT,
    p1: POINT,
    p2: POINT,
    start_angle: i32,
    end_angle: i32,
    pbrush: PBRUSH,
    Pen: i32,
) -> i32 {
    let mut r = r;
    let x1: i32;
    let x2: i32;
    let start_above: i32;
    let end_above: i32;
    let rise1: i32;
    let run1: i32;
    let rise2: i32;
    let run2: i32;

    rise1 = p1.y - p0.y;
    run1 = p1.x - p0.x;
    rise2 = p2.y - p0.y;
    run2 = p2.x - p0.x;

    if r.y <= p0.y {
        if p1.y <= r.y {
            if rise1 == 0 {
                x1 = p1.x;
            } else {
                x1 = p0.x + (r.y - p0.y) * run1 / rise1;
            }
            start_above = 1;
        } else if (start_angle >= 0) && (start_angle <= 180) {
            x1 = p1.x;
            start_above = 1;
        } else {
            x1 = r.x + r.width;
            start_above = 0;
        }
        let mut x1 = x1;
        if x1 < r.x {
            x1 = r.x;
        }
        if x1 > r.x + r.width {
            x1 = r.x + r.width;
        }

        if p2.y <= r.y {
            if rise2 == 0 {
                x2 = p2.x;
            } else {
                x2 = p0.x + (r.y - p0.y) * run2 / rise2;
            }
            end_above = 1;
        } else if (end_angle >= 0) && (end_angle <= 180) {
            x2 = p2.x;
            end_above = 1;
        } else {
            x2 = r.x;
            end_above = 0;
        }
        let mut x2 = x2;

        if x2 < r.x {
            x2 = r.x;
        }

        if x2 > r.x + r.width {
            x2 = r.x + r.width;
        }

        if (start_above != 0) && (end_above != 0) {
            if start_angle > end_angle {
                if IntFillRect(g, r.x, r.y, x1 - r.x, r.height, pbrush, Pen) == 0 {
                    return 0;
                }
                return IntFillRect(g, x2, r.y, r.x + r.width - x2, r.height, pbrush, Pen);
            } else {
                r.width = x1 - x2;
                r.x = x2;
                return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
            }
        } else if start_above != 0 {
            r.width = x1 - r.x;
            return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
        } else if end_above != 0 {
            r.width = r.x + r.width - x2;
            r.x = x2;
            return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
        } else {
            if start_angle > end_angle {
                return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
            } else {
                return 1;
            }
        }
    } else {
        if p1.y >= r.y {
            if rise1 == 0 {
                x1 = p1.x;
            } else {
                x1 = p0.x + (r.y - p0.y) * run1 / rise1;
            }
            start_above = 0;
        } else if (start_angle >= 180) && (start_angle <= 360) {
            x1 = p1.x;
            start_above = 0;
        } else {
            x1 = r.x;
            start_above = 1;
        }
        let mut x1 = x1;
        if x1 < r.x {
            x1 = r.x;
        }
        if x1 > r.x + r.width {
            x1 = r.x + r.width;
        }

        if p2.y >= r.y {
            if rise2 == 0 {
                x2 = p2.x;
            } else {
                x2 = p0.x + (r.y - p0.y) * run2 / rise2;
            }
            end_above = 0;
        } else if (end_angle >= 180) && (end_angle <= 360) {
            x2 = p2.x;
            end_above = 0;
        } else {
            x2 = r.x + r.width;
            end_above = 1;
        }
        let mut x2 = x2;
        if x2 < r.x {
            x2 = r.x;
        }
        if x2 > r.x + r.width {
            x2 = r.x + r.width;
        }

        if (start_above != 0) && (end_above != 0) {
            if start_angle > end_angle {
                return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
            } else {
                return 1;
            }
        } else if start_above != 0 {
            r.width = x2 - r.x;
            return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
        } else if end_above != 0 {
            r.width = r.x + r.width - x1;
            r.x = x1;
            return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
        } else {
            if start_angle > end_angle {
                if IntFillRect(g, r.x, r.y, x2 - r.x, r.height, pbrush, Pen) == 0 {
                    return 0;
                }
                return IntFillRect(g, x1, r.y, r.x + r.width - x1, r.height, pbrush, Pen);
            } else {
                r.width = x2 - x1;
                r.x = x1;
                return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
            }
        }
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn app_fill_arc_rect(
    g: *mut DC,
    r: Rect,
    p0: POINT,
    p1: POINT,
    p2: POINT,
    start_angle: i32,
    end_angle: i32,
    pbrush: PBRUSH,
    Pen: i32,
) -> i32 {
    let mut r = r;
    let x1: i32;
    let x2: i32;
    let start_above: i32;
    let end_above: i32;
    let rise1: i32;
    let run1: i32;
    let rise2: i32;
    let run2: i32;

    rise1 = p1.y - p0.y;
    run1 = p1.x - p0.x;
    rise2 = p2.y - p0.y;
    run2 = p2.x - p0.x;

    if r.y <= p0.y {
        if p1.y <= r.y {
            if rise1 == 0 {
                x1 = p1.x;
            } else {
                x1 = p0.x + (r.y - p0.y) * run1 / rise1;
            }
            start_above = 1;
        } else if (start_angle >= 0) && (start_angle <= 180) {
            x1 = p1.x;
            start_above = 1;
        } else {
            x1 = r.x + r.width;
            start_above = 0;
        }
        let mut x1 = x1;
        if x1 < r.x {
            x1 = r.x;
        }
        if x1 > r.x + r.width {
            x1 = r.x + r.width;
        }

        if p2.y <= r.y {
            if rise2 == 0 {
                x2 = p2.x;
            } else {
                x2 = p0.x + (r.y - p0.y) * run2 / rise2;
            }
            end_above = 1;
        } else if (end_angle >= 0) && (end_angle <= 180) {
            x2 = p2.x;
            end_above = 1;
        } else {
            x2 = r.x;
            end_above = 0;
        }
        let mut x2 = x2;

        if x2 < r.x {
            x2 = r.x;
        }

        if x2 > r.x + r.width {
            x2 = r.x + r.width;
        }

        if (start_above != 0) && (end_above != 0) {
            if start_angle > end_angle {
                if IntFillRect(g, r.x, r.y, x1 - r.x, r.height, pbrush, Pen) == 0 {
                    return 0;
                }
                return IntFillRect(g, x2, r.y, r.x + r.width - x2, r.height, pbrush, Pen);
            } else {
                r.width = x1 - x2;
                r.x = x2;
                return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
            }
        } else if start_above != 0 {
            r.width = x1 - r.x;
            return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
        } else if end_above != 0 {
            r.width = r.x + r.width - x2;
            r.x = x2;
            return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
        } else {
            if start_angle > end_angle {
                return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
            } else {
                return 1;
            }
        }
    } else {
        if p1.y >= r.y {
            if rise1 == 0 {
                x1 = p1.x;
            } else {
                x1 = p0.x + (r.y - p0.y) * run1 / rise1;
            }
            start_above = 0;
        } else if (start_angle >= 180) && (start_angle <= 360) {
            x1 = p1.x;
            start_above = 0;
        } else {
            x1 = r.x;
            start_above = 1;
        }
        let mut x1 = x1;
        if x1 < r.x {
            x1 = r.x;
        }
        if x1 > r.x + r.width {
            x1 = r.x + r.width;
        }

        if p2.y >= r.y {
            if rise2 == 0 {
                x2 = p2.x;
            } else {
                x2 = p0.x + (r.y - p0.y) * run2 / rise2;
            }
            end_above = 0;
        } else if (end_angle >= 180) && (end_angle <= 360) {
            x2 = p2.x;
            end_above = 0;
        } else {
            x2 = r.x + r.width;
            end_above = 1;
        }
        let mut x2 = x2;
        if x2 < r.x {
            x2 = r.x;
        }
        if x2 > r.x + r.width {
            x2 = r.x + r.width;
        }

        if (start_above != 0) && (end_above != 0) {
            if start_angle > end_angle {
                return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
            } else {
                return 1;
            }
        } else if start_above != 0 {
            r.width = x2 - r.x;
            return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
        } else if end_above != 0 {
            r.width = r.x + r.width - x1;
            r.x = x1;
            return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
        } else {
            if start_angle > end_angle {
                if IntFillRect(g, r.x, r.y, x2 - r.x, r.height, pbrush, Pen) == 0 {
                    return 0;
                }
                return IntFillRect(g, x1, r.y, r.x + r.width - x1, r.height, pbrush, Pen);
            } else {
                r.width = x2 - x1;
                r.x = x1;
                return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, Pen);
            }
        }
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn app_fill_ellipse(g: *mut DC, r: Rect, pbrush: PBRUSH) -> i32 {
    let a = r.width / 2;
    let b = r.height / 2;
    let mut x = 0;
    let mut y = b;
    let a2 = (a * a) as i64;
    let b2 = (b * b) as i64;
    let xcrit = (3 * a2 / 4) + 1;
    let ycrit = (3 * b2 / 4) + 1;
    let mut t = b2 + a2 - 2 * a2 * b as i64;
    let mut dxt = b2 * (3 + x + x) as i64;
    let mut dyt = a2 * (3 - y - y) as i64;
    let d2xt = (b2 + b2) as i32;
    let d2yt = (a2 + a2) as i32;
    let mut r1: Rect;
    let mut r2: Rect;
    let mut result = 1i32;

    if (r.width <= 2) || (r.height <= 2) {
        return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, 0);
    }

    r1 = Rect {
        x: r.x + a,
        y: r.y,
        width: r.width & 1,
        height: 1,
    };

    r2 = r1;
    r2.y = r.y + r.height - 1;

    while y > 0 {
        if t + a2 * (y as i64) < xcrit {
            x += 1;
            t += dxt;
            dxt += d2xt as i64;

            r1.x -= 1;
            r1.width += 2;
            r2.x -= 1;
            r2.width += 2;
        } else if t - b2 * x as i64 >= ycrit {
            y -= 1;
            t += dyt;
            dyt += d2yt as i64;

            r1.height += 1;
            r2.height += 1;
            r2.y -= 1;
        } else {
            x += 1;
            y -= 1;
            t += dxt + dyt;
            dxt += d2xt as i64;
            dyt += d2yt as i64;

            if (r1.width > 0) && (r1.height > 0) {
                if r1.y + r1.height < r2.y {
                    result &= IntFillRect(g, r1.x, r1.y, r1.width, r1.height, pbrush, 0);
                    result &= IntFillRect(g, r2.x, r2.y, r2.width, r2.height, pbrush, 0);
                }

                r1.y += r1.height;
                r1.height = 1;
                r2.y -= 1;
                r2.height = 1;
            } else {
                r1.height += 1;
                r2.height += 1;
                r2.y -= 1;
            }

            r1.x -= 1;
            r1.width += 2;
            r2.x -= 1;
            r2.width += 2;
        }
    }
    if r1.y < r2.y {
        r1.x = r.x;
        r1.width = r.width;
        r1.height = r2.y + r2.height - r1.y;
        result &= IntFillRect(g, r1.x, r1.y, r1.width, r1.height, pbrush, 0);
    } else if x <= a {
        r1.x = r.x;
        r1.width = r.width;
        r1.height = r1.y + r1.height - r2.y;
        r1.y = r2.y;
        result &= IntFillRect(g, r1.x, r1.y, r1.width, r1.height, pbrush, 0);
    }
    result
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn app_fill_ellipse(g: *mut DC, r: Rect, pbrush: PBRUSH) -> i32 {
    let a = r.width / 2;
    let b = r.height / 2;
    let mut x = 0;
    let mut y = b;
    let a2 = (a * a) as i64;
    let b2 = (b * b) as i64;
    let xcrit = (3 * a2 / 4) + 1;
    let ycrit = (3 * b2 / 4) + 1;
    let mut t = b2 + a2 - 2 * a2 * b as i64;
    let mut dxt = b2 * (3 + x + x) as i64;
    let mut dyt = a2 * (3 - y - y) as i64;
    let d2xt = (b2 + b2) as i32;
    let d2yt = (a2 + a2) as i32;
    let mut r1: Rect;
    let mut r2: Rect;
    let mut result = 1i32;

    if (r.width <= 2) || (r.height <= 2) {
        return IntFillRect(g, r.x, r.y, r.width, r.height, pbrush, 0);
    }

    r1 = Rect {
        x: r.x + a,
        y: r.y,
        width: r.width & 1,
        height: 1,
    };

    r2 = r1;
    r2.y = r.y + r.height - 1;

    while y > 0 {
        if t + a2 * (y as i64) < xcrit {
            x += 1;
            t += dxt;
            dxt += d2xt as i64;

            r1.x -= 1;
            r1.width += 2;
            r2.x -= 1;
            r2.width += 2;
        } else if t - b2 * x as i64 >= ycrit {
            y -= 1;
            t += dyt;
            dyt += d2yt as i64;

            r1.height += 1;
            r2.height += 1;
            r2.y -= 1;
        } else {
            x += 1;
            y -= 1;
            t += dxt + dyt;
            dxt += d2xt as i64;
            dyt += d2yt as i64;

            if (r1.width > 0) && (r1.height > 0) {
                if r1.y + r1.height < r2.y {
                    result &= IntFillRect(g, r1.x, r1.y, r1.width, r1.height, pbrush, 0);
                    result &= IntFillRect(g, r2.x, r2.y, r2.width, r2.height, pbrush, 0);
                }

                r1.y += r1.height;
                r1.height = 1;
                r2.y -= 1;
                r2.height = 1;
            } else {
                r1.height += 1;
                r2.height += 1;
                r2.y -= 1;
            }

            r1.x -= 1;
            r1.width += 2;
            r2.x -= 1;
            r2.width += 2;
        }
    }
    if r1.y < r2.y {
        r1.x = r.x;
        r1.width = r.width;
        r1.height = r2.y + r2.height - r1.y;
        result &= IntFillRect(g, r1.x, r1.y, r1.width, r1.height, pbrush, 0);
    } else if x <= a {
        r1.x = r.x;
        r1.width = r.width;
        r1.height = r1.y + r1.height - r2.y;
        r1.y = r2.y;
        result &= IntFillRect(g, r1.x, r1.y, r1.width, r1.height, pbrush, 0);
    }
    result
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn app_boundary_point(r: Rect, angle: i32) -> POINT {
    let mut cx: i32;
    let mut cy: i32;
    let tangent: f64;

    cx = r.width;
    cx /= 2;
    cx += r.x;

    cy = r.height;
    cy /= 2;
    cy += r.y;

    if angle == 0 {
        return pt(r.x + r.width, cy);
    } else if angle == 45 {
        return pt(r.x + r.width, r.y);
    } else if angle == 90 {
        return pt(cx, r.y);
    } else if angle == 135 {
        return pt(r.x, r.y);
    } else if angle == 180 {
        return pt(r.x, cy);
    } else if angle == 225 {
        return pt(r.x, r.y + r.height);
    } else if angle == 270 {
        return pt(cx, r.y + r.height);
    } else if angle == 315 {
        return pt(r.x + r.width, r.y + r.height);
    }

    tangent = tan(DEGREES_TO_RADIANS!(angle));

    if (angle > 45) && (angle < 135) {
        return pt((cx as f64 + r.height as f64 / tangent / 2.0) as i32, r.y);
    } else if (angle > 225) && (angle < 315) {
        return pt((cx as f64 - r.height as f64 / tangent / 2.0) as i32, r.y + r.height);
    } else if (angle > 135) && (angle < 225) {
        return pt(r.x, (cy as f64 + r.width as f64 * tangent / 2.0) as i32);
    } else {
        return pt(r.x + r.width, (cy as f64 - r.width as f64 * tangent / 2.0) as i32);
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn app_boundary_point(r: Rect, angle: i32) -> POINT {
    let mut cx: i32;
    let mut cy: i32;
    let tangent: f64;

    cx = r.width;
    cx /= 2;
    cx += r.x;

    cy = r.height;
    cy /= 2;
    cy += r.y;

    if angle == 0 {
        return pt(r.x + r.width, cy);
    } else if angle == 45 {
        return pt(r.x + r.width, r.y);
    } else if angle == 90 {
        return pt(cx, r.y);
    } else if angle == 135 {
        return pt(r.x, r.y);
    } else if angle == 180 {
        return pt(r.x, cy);
    } else if angle == 225 {
        return pt(r.x, r.y + r.height);
    } else if angle == 270 {
        return pt(cx, r.y + r.height);
    } else if angle == 315 {
        return pt(r.x + r.width, r.y + r.height);
    }

    tangent = tan(DEGREES_TO_RADIANS!(angle));

    if (angle > 45) && (angle < 135) {
        return pt((cx as f64 + r.height as f64 / tangent / 2.0) as i32, r.y);
    } else if (angle > 225) && (angle < 315) {
        return pt((cx as f64 - r.height as f64 / tangent / 2.0) as i32, r.y + r.height);
    } else if (angle > 135) && (angle < 225) {
        return pt(r.x, (cy as f64 + r.width as f64 * tangent / 2.0) as i32);
    } else {
        return pt(r.x + r.width, (cy as f64 - r.width as f64 * tangent / 2.0) as i32);
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn app_fill_arc(
    g: *mut DC,
    r: Rect,
    start_angle: i32,
    end_angle: i32,
    pbrush: PBRUSH,
    Chord: i32,
) -> i32 {
    let mut start_angle = start_angle;
    let mut end_angle = end_angle;

    let a = r.width / 2;
    let b = r.height / 2;
    let mut x = 0;
    let mut y = b;
    let a2 = (a * a) as i64;
    let b2 = (b * b) as i64;
    let xcrit = (3 * a2 / 4) + 1;
    let ycrit = (3 * b2 / 4) + 1;
    let mut t = b2 + a2 - 2 * a2 * b as i64;
    let mut dxt = b2 * (3 + x + x) as i64;
    let mut dyt = a2 * (3 - y - y) as i64;
    let d2xt = b2 + b2;
    let d2yt = a2 + a2;
    let mut r1: Rect;
    let mut r2: Rect;
    let movedown: i32;
    let moveout: i32;
    let mut result = 1;

    let p0: POINT;
    let p1: POINT;
    let p2: POINT;

    if (start_angle + 360 <= end_angle) || (start_angle - 360 >= end_angle) {
        return app_fill_ellipse(g, r, pbrush);
    }

    while start_angle < 0 {
        start_angle += 360;
    }
    start_angle %= 360;

    while end_angle < 0 {
        end_angle += 360;
    }
    end_angle %= 360;

    if start_angle == end_angle {
        return 1;
    }

    p1 = app_boundary_point(r, start_angle);
    p2 = app_boundary_point(r, end_angle);
    if Chord != 0 {
        p0 = pt((p1.x + p2.x) / 2, (p1.y + p2.y) / 2);
    } else {
        p0 = pt(r.x + r.width / 2, r.y + r.height / 2);
    }

    r1 = Rect {
        x: r.x + a,
        y: r.y,
        width: r.width & 1,
        height: 1,
    };

    r2 = r1;
    r2.y = r.y + r.height - 1;

    while y > 0 {
        let mut moveout = 0;
        let mut movedown = 0;

        if t + a2 * (y as i64) < xcrit {
            x += 1;
            t += dxt;
            dxt += d2xt;

            moveout = 1;
        } else if t - b2 * (x as i64) >= ycrit {
            y -= 1;
            t += dyt;
            dyt += d2yt;

            movedown = 1;
        } else {
            x += 1;
            y -= 1;
            t += dxt + dyt;
            dxt += d2xt;
            dyt += d2yt;

            moveout = 1;
            movedown = 1;
        }

        if movedown != 0 {
            if r1.width == 0 {
                r1.x -= 1;
                r1.width += 2;
                r2.x -= 1;
                r2.width += 2;
                moveout = 0;
            }

            if r1.x < r.x {
                r1.x = r.x;
                r2.x = r.x;
            }
            if r1.width > r.width {
                r1.width = r.width;
                r2.width = r.width;
            }
            if r1.y == r2.y - 1 {
                r1.x = r.x;
                r2.x = r.x;
                r1.width = r.width;
                r2.width = r.width;
            }

            if (r1.width > 0) && (r1.y + r1.height < r2.y) {
                result &= app_fill_arc_rect(
                    g,
                    r1,
                    p0,
                    p1,
                    p2,
                    start_angle,
                    end_angle,
                    pbrush,
                    0,
                );
                result &= app_fill_arc_rect(
                    g,
                    r2,
                    p0,
                    p1,
                    p2,
                    start_angle,
                    end_angle,
                    pbrush,
                    0,
                );
            }

            r1.y += 1;
            r2.y -= 1;
        }

        if moveout != 0 {
            r1.x -= 1;
            r1.width += 2;
            r2.x -= 1;
            r2.width += 2;
        }
    }

    if r1.y < r2.y {
        r1.x = r.x;
        r1.width = r.width;
        r1.height = r2.y + r2.height - r1.y;
        while r1.height > 0 {
            result &= app_fill_arc_rect(
                g,
                rect(r1.x, r1.y, r1.width, 1),
                p0,
                p1,
                p2,
                start_angle,
                end_angle,
                pbrush,
                0,
            );
            r1.y += 1;
            r1.height -= 1;
        }
    } else if x <= a {
        r1.x = r.x;
        r1.width = r.width;
        r1.height = r1.y + r1.height - r2.y;
        r1.y = r2.y;
        while r1.height > 0 {
            result &= app_fill_arc_rect(
                g,
                rect(r1.x, r1.y, r1.width, 1),
                p0,
                p1,
                p2,
                start_angle,
                end_angle,
                pbrush,
                0,
            );
            r1.y += 1;
            r1.height -= 1;
        }
    }
    result
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn app_fill_arc(
    g: *mut DC,
    r: Rect,
    start_angle: i32,
    end_angle: i32,
    pbrush: PBRUSH,
    Chord: i32,
) -> i32 {
    let mut start_angle = start_angle;
    let mut end_angle = end_angle;

    let a = r.width / 2;
    let b = r.height / 2;
    let mut x = 0;
    let mut y = b;
    let a2 = (a * a) as i64;
    let b2 = (b * b) as i64;
    let xcrit = (3 * a2 / 4) + 1;
    let ycrit = (3 * b2 / 4) + 1;
    let mut t = b2 + a2 - 2 * a2 * b as i64;
    let mut dxt = b2 * (3 + x + x) as i64;
    let mut dyt = a2 * (3 - y - y) as i64;
    let d2xt = b2 + b2;
    let d2yt = a2 + a2;
    let mut r1: Rect;
    let mut r2: Rect;
    let movedown: i32;
    let moveout: i32;
    let mut result = 1;

    let p0: POINT;
    let p1: POINT;
    let p2: POINT;

    if (start_angle + 360 <= end_angle) || (start_angle - 360 >= end_angle) {
        return app_fill_ellipse(g, r, pbrush);
    }

    while start_angle < 0 {
        start_angle += 360;
    }
    start_angle %= 360;

    while end_angle < 0 {
        end_angle += 360;
    }
    end_angle %= 360;

    if start_angle == end_angle {
        return 1;
    }

    p1 = app_boundary_point(r, start_angle);
    p2 = app_boundary_point(r, end_angle);
    if Chord != 0 {
        p0 = pt((p1.x + p2.x) / 2, (p1.y + p2.y) / 2);
    } else {
        p0 = pt(r.x + r.width / 2, r.y + r.height / 2);
    }

    r1 = Rect {
        x: r.x + a,
        y: r.y,
        width: r.width & 1,
        height: 1,
    };

    r2 = r1;
    r2.y = r.y + r.height - 1;

    while y > 0 {
        let mut moveout = 0;
        let mut movedown = 0;

        if t + a2 * (y as i64) < xcrit {
            x += 1;
            t += dxt;
            dxt += d2xt;

            moveout = 1;
        } else if t - b2 * (x as i64) >= ycrit {
            y -= 1;
            t += dyt;
            dyt += d2yt;

            movedown = 1;
        } else {
            x += 1;
            y -= 1;
            t += dxt + dyt;
            dxt += d2xt;
            dyt += d2yt;

            moveout = 1;
            movedown = 1;
        }

        if movedown != 0 {
            if r1.width == 0 {
                r1.x -= 1;
                r1.width += 2;
                r2.x -= 1;
                r2.width += 2;
                moveout = 0;
            }

            if r1.x < r.x {
                r1.x = r.x;
                r2.x = r.x;
            }
            if r1.width > r.width {
                r1.width = r.width;
                r2.width = r.width;
            }
            if r1.y == r2.y - 1 {
                r1.x = r.x;
                r2.x = r.x;
                r1.width = r.width;
                r2.width = r.width;
            }

            if (r1.width > 0) && (r1.y + r1.height < r2.y) {
                result &= app_fill_arc_rect(
                    g,
                    r1,
                    p0,
                    p1,
                    p2,
                    start_angle,
                    end_angle,
                    pbrush,
                    0,
                );
                result &= app_fill_arc_rect(
                    g,
                    r2,
                    p0,
                    p1,
                    p2,
                    start_angle,
                    end_angle,
                    pbrush,
                    0,
                );
            }

            r1.y += 1;
            r2.y -= 1;
        }

        if moveout != 0 {
            r1.x -= 1;
            r1.width += 2;
            r2.x -= 1;
            r2.width += 2;
        }
    }

    if r1.y < r2.y {
        r1.x = r.x;
        r1.width = r.width;
        r1.height = r2.y + r2.height - r1.y;
        while r1.height > 0 {
            result &= app_fill_arc_rect(
                g,
                rect(r1.x, r1.y, r1.width, 1),
                p0,
                p1,
                p2,
                start_angle,
                end_angle,
                pbrush,
                0,
            );
            r1.y += 1;
            r1.height -= 1;
        }
    } else if x <= a {
        r1.x = r.x;
        r1.width = r.width;
        r1.height = r1.y + r1.height - r2.y;
        r1.y = r2.y;
        while r1.height > 0 {
            result &= app_fill_arc_rect(
                g,
                rect(r1.x, r1.y, r1.width, 1),
                p0,
                p1,
                p2,
                start_angle,
                end_angle,
                pbrush,
                0,
            );
            r1.y += 1;
            r1.height -= 1;
        }
    }
    result
}

unsafe fn app_draw_arc(
    g: *mut DC,
    r: Rect,
    start_angle: i32,
    end_angle: i32,
    pbrushPen: PBRUSH,
    Chord: i32,
) -> i32 {
    let mut start_angle = start_angle;
    let mut end_angle = end_angle;

    let a = r.width / 2;
    let b = r.height / 2;
    let mut x = 0;
    let mut y = b;
    let a2 = (a * a) as i64;
    let b2 = (b * b) as i64;
    let xcrit = (3 * a2 / 4) + 1;
    let ycrit = (3 * b2 / 4) + 1;
    let mut t = b2 + a2 - 2 * a2 * b as i64;
    let mut dxt = b2 * (3 + x + x) as i64;
    let mut dyt = a2 * (3 - y - y) as i64;
    let d2xt = (b2 + b2) as i32;
    let d2yt = (a2 + a2) as i32;

    let w = (*pbrushbody(pbrushPen)).lWidth;

    let A = if a - w > 0 { a - w } else { 0 };
    let B = if b - w > 0 { b - w } else { 0 };
    let mut X = 0;
    let mut Y = B;
    let A2 = (A * A) as i64;
    let B2 = (B * B) as i64;
    let XCRIT = (3 * A2 / 4) + 1;
    let YCRIT = (3 * B2 / 4) + 1;
    let mut T = B2 + A2 - 2 * A2 * B as i64;
    let mut DXT = B2 * (3 + X + X) as i64 as i64;
    let mut DYT = A2 * (3 - Y - Y) as i64 as i64;
    let D2XT = (B2 + B2) as i32;
    let D2YT = (A2 + A2) as i32;

    let mut innerX = 0;
    let mut prevx;
    let mut prevy;
    let mut W;
    let mut r1: Rect;
    let mut r2: Rect;
    let mut result = 1;

    let p0: POINT;
    let p1: POINT;
    let p2: POINT;

    if (start_angle + 360 <= end_angle) || (start_angle - 360 >= end_angle) {
        return app_draw_ellipse(g, r, pbrushPen);
    }

    while start_angle < 0 {
        start_angle += 360;
    }
    start_angle %= 360;

    while end_angle < 0 {
        end_angle += 360;
    }
    end_angle %= 360;

    if start_angle == end_angle {
        return 1;
    }

    p1 = app_boundary_point(r, start_angle);
    p2 = app_boundary_point(r, end_angle);
    if Chord != 0 {
        p0 = pt((p1.x + p2.x) / 2, (p1.y + p2.y) / 2);
    } else {
        p0 = pt(r.x + r.width / 2, r.y + r.height / 2);
    }

    r1 = Rect {
        x: r.x + a,
        y: r.y,
        width: r.width & 1,
        height: 1,
    };

    r2 = r1;
    r2.y = r.y + r.height - 1;

    prevx = r1.x;
    prevy = r1.y;

    while y > 0 {
        while Y == y {
            innerX = X;

            if T + A2 * (Y as i64) < XCRIT {
                X += 1;
                T += DXT;
                DXT += D2XT as i64;
            } else if T - B2 * X as i64 >= YCRIT {
                Y -= 1;
                T += DYT;
                DYT += D2YT as i64;
            } else {
                X += 1;
                Y -= 1;
                T += DXT + DYT;
                DXT += D2XT as i64;
                DYT += D2YT as i64;
            }
        }

        let mut movedown = 0;
        let mut moveout = 0;

        W = x - innerX;
        if r1.x + W < prevx {
            W = prevx - r1.x;
        }
        if W < w {
            W = w;
        }

        if t + a2 * (y as i64) < xcrit {
            x += 1;
            t += dxt;
            dxt += d2xt as i64;

            moveout = 1;
        } else if t - b2 * x as i64 >= ycrit {
            y -= 1;
            t += dyt;
            dyt += d2yt as i64;

            movedown = 1;
        } else {
            x += 1;
            y -= 1;
            t += dxt + dyt;
            dxt += d2xt as i64;
            dyt += d2yt as i64;

            movedown = 1;
            moveout = 1;
        }

        if movedown != 0 {
            if r1.width == 0 {
                r1.x -= 1;
                r1.width += 2;
                r2.x -= 1;
                r2.width += 2;
                moveout = 0;
            }

            if r1.x < r.x {
                r1.x = r.x;
                r2.x = r.x;
            }
            if r1.width > r.width {
                r1.width = r.width;
                r2.width = r.width;
            }
            if r1.y == r2.y - 1 {
                r1.x = r.x;
                r2.x = r.x;
                r1.width = r.width;
                r2.width = r.width;
            }

            if (r1.y < r.y + w) || (r1.x + W >= r1.x + r1.width - W) {
                result &= app_fill_arc_rect(
                    g,
                    r1,
                    p0,
                    p1,
                    p2,
                    start_angle,
                    end_angle,
                    pbrushPen,
                    1,
                );
                result &= app_fill_arc_rect(
                    g,
                    r2,
                    p0,
                    p1,
                    p2,
                    start_angle,
                    end_angle,
                    pbrushPen,
                    1,
                );

                prevx = r1.x;
                prevy = r1.y;
            } else if r1.y + r1.height < r2.y {
                result &= app_fill_arc_rect(
                    g,
                    rect(r1.x, r1.y, W, 1),
                    p0,
                    p1,
                    p2,
                    start_angle,
                    end_angle,
                    pbrushPen,
                    1,
                );
                result &= app_fill_arc_rect(
                    g,
                    rect(r1.x + r1.width - W, r1.y, W, 1),
                    p0,
                    p1,
                    p2,
                    start_angle,
                    end_angle,
                    pbrushPen,
                    1,
                );
                result &= app_fill_arc_rect(
                    g,
                    rect(r2.x, r2.y, W, 1),
                    p0,
                    p1,
                    p2,
                    start_angle,
                    end_angle,
                    pbrushPen,
                    1,
                );
                result &= app_fill_arc_rect(
                    g,
                    rect(r2.x + r2.width - W, r2.y, W, 1),
                    p0,
                    p1,
                    p2,
                    start_angle,
                    end_angle,
                    pbrushPen,
                    1,
                );

                prevx = r1.x;
                prevy = r1.y;
            }

            r1.y += 1;
            r2.y -= 1;
        }

        if moveout != 0 {
            r1.x -= 1;
            r1.width += 2;
            r2.x -= 1;
            r2.width += 2;
        }
    }

    if (x <= a) && (prevy < r2.y) {
        r1.height = r1.y + r1.height - r2.y;
        r1.y = r2.y;

        W = w;
        if r.x + W != prevx {
            W = prevx - r.x;
        }
        if W < w {
            W = w;
        }

        if W + W >= r.width {
            while r1.height > 0 {
                result &= app_fill_arc_rect(
                    g,
                    rect(r.x, r1.y, r.width, 1),
                    p0,
                    p1,
                    p2,
                    start_angle,
                    end_angle,
                    pbrushPen,
                    1,
                );
                r1.y += 1;
                r1.height -= 1;
            }
            return result;
        }

        while r1.height > 0 {
            result &= app_fill_arc_rect(
                g,
                rect(r.x, r1.y, W, 1),
                p0,
                p1,
                p2,
                start_angle,
                end_angle,
                pbrushPen,
                1,
            );
            result &= app_fill_arc_rect(
                g,
                rect(r.x + r.width - W, r1.y, W, 1),
                p0,
                p1,
                p2,
                start_angle,
                end_angle,
                pbrushPen,
                1,
            );
            r1.y += 1;
            r1.height -= 1;
        }
    }

    result
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntFillRect(
    dc: *mut DC,
    XLeft: i32,
    YLeft: i32,
    Width: i32,
    Height: i32,
    pbrush: PBRUSH,
    Pen: i32,
) -> i32 {
    let mut ROP: u32 = ROP4_FROM_INDEX(R3_OPINDEX_PATCOPY);
    let mut DestRect: RECTL = core::mem::zeroed();
    let psurf: *mut SURFACE;
    let mut BrushOrigin: POINTL = core::mem::zeroed();
    let mut Ret: i32 = 1;
    let pdcattr: PDC_ATTR;
    let mut XLeft = XLeft;
    let mut YLeft = YLeft;
    let mut Width = Width;
    let mut Height = Height;

    ASSERT_DC_PREPARED(dc);

    psurf = (*dc).dclevel.pSurface;
    if psurf.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if ((*pbrushbody(pbrush)).flAttrs & BR_IS_NULL) == 0 {
        pdcattr = (*dc).pdcattr;

        if Width < 0 {
            XLeft += Width;
            Width = 0 - Width;
        }
        if Height < 0 {
            YLeft += Height;
            Height = 0 - Height;
        }

        DestRect.left = XLeft;
        DestRect.right = XLeft + Width;

        DestRect.top = YLeft;
        DestRect.bottom = YLeft + Height;

        BrushOrigin.x = (*pbrushbody(pbrush)).ptOrigin.x;
        BrushOrigin.y = (*pbrushbody(pbrush)).ptOrigin.y;

        if (*pdcattr).jROP2 == R2_XORPEN as u8 {
            ROP = ROP4_FROM_INDEX(R3_OPINDEX_PATINVERT);
        }

        Ret = IntEngBitBlt(
            core::ptr::addr_of_mut!((*psurf).SurfObj),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(DestRect),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            if Pen != 0 { core::ptr::addr_of_mut!((*dc).eboLine.BrushObject) } else { core::ptr::addr_of_mut!((*dc).eboFill.BrushObject) },
            core::ptr::addr_of_mut!(BrushOrigin),
            ROP,
        );
    }

    Ret as i32
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntFillRect(
    dc: *mut DC,
    XLeft: i32,
    YLeft: i32,
    Width: i32,
    Height: i32,
    pbrush: PBRUSH,
    Pen: i32,
) -> i32 {
    let mut ROP: u32 = ROP4_FROM_INDEX(R3_OPINDEX_PATCOPY);
    let mut DestRect: RECTL = core::mem::zeroed();
    let psurf: *mut SURFACE;
    let mut BrushOrigin: POINTL = core::mem::zeroed();
    let mut Ret: i32 = 1;
    let pdcattr: PDC_ATTR;
    let mut XLeft = XLeft;
    let mut YLeft = YLeft;
    let mut Width = Width;
    let mut Height = Height;

    ASSERT_DC_PREPARED(dc);

    psurf = (*dc).dclevel.pSurface;
    if psurf.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if ((*pbrushbody(pbrush)).flAttrs & BR_IS_NULL) == 0 {
        pdcattr = (*dc).pdcattr;

        if Width < 0 {
            XLeft += Width;
            Width = 0 - Width;
        }
        if Height < 0 {
            YLeft += Height;
            Height = 0 - Height;
        }

        DestRect.left = XLeft;
        DestRect.right = XLeft + Width;

        DestRect.top = YLeft;
        DestRect.bottom = YLeft + Height;

        BrushOrigin.x = (*pbrushbody(pbrush)).ptOrigin.x;
        BrushOrigin.y = (*pbrushbody(pbrush)).ptOrigin.y;

        if (*pdcattr).jROP2 == R2_XORPEN as u8 {
            ROP = ROP4_FROM_INDEX(R3_OPINDEX_PATINVERT);
        }

        Ret = IntEngBitBlt(
            core::ptr::addr_of_mut!((*psurf).SurfObj),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(DestRect),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            if Pen != 0 { core::ptr::addr_of_mut!((*dc).eboLine.BrushObject) } else { core::ptr::addr_of_mut!((*dc).eboFill.BrushObject) },
            core::ptr::addr_of_mut!(BrushOrigin),
            ROP,
        );
    }

    Ret as i32
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntFillArc(
    dc: PDC,
    XLeft: i32,
    YLeft: i32,
    Width: i32,
    Height: i32,
    StartArc: f64,
    EndArc: f64,
    arctype: ARCTYPE,
) -> BOOL {
    let pdcattr: PDC_ATTR;
    let pbrush: PBRUSH;
    let Start: i32 = ceil(StartArc) as i32;
    let End: i32 = ceil(EndArc) as i32;
    let Chord: BOOL = if arctype == GdiTypeChord { 1 } else { 0 };
    let ret: BOOL;

    pdcattr = (*dc).pdcattr;

    pbrush = BRUSH_ShareLockBrush((*pdcattr).hbrush as HBRUSH);
    if pbrush.is_null() {
        EngSetLastError(ERROR_INTERNAL_ERROR);
        return 0;
    }

    ret = app_fill_arc(
        dc,
        rect(XLeft, YLeft, Width, Height),
        if (*dc).dclevel.flPath & DCPATH_CLOCKWISE != 0 { -End } else { -Start },
        if (*dc).dclevel.flPath & DCPATH_CLOCKWISE != 0 { -Start } else { -End },
        pbrush,
        Chord,
    );

    BRUSH_ShareUnlockBrush(pbrush);
    ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntFillArc(
    dc: PDC,
    XLeft: i32,
    YLeft: i32,
    Width: i32,
    Height: i32,
    StartArc: f64,
    EndArc: f64,
    arctype: ARCTYPE,
) -> BOOL {
    let pdcattr: PDC_ATTR;
    let pbrush: PBRUSH;
    let Start: i32 = ceil(StartArc) as i32;
    let End: i32 = ceil(EndArc) as i32;
    let Chord: BOOL = if arctype == GdiTypeChord { 1 } else { 0 };
    let ret: BOOL;

    pdcattr = (*dc).pdcattr;

    pbrush = BRUSH_ShareLockBrush((*pdcattr).hbrush as HBRUSH);
    if pbrush.is_null() {
        EngSetLastError(ERROR_INTERNAL_ERROR);
        return 0;
    }

    ret = app_fill_arc(
        dc,
        rect(XLeft, YLeft, Width, Height),
        if (*dc).dclevel.flPath & DCPATH_CLOCKWISE != 0 { -End } else { -Start },
        if (*dc).dclevel.flPath & DCPATH_CLOCKWISE != 0 { -Start } else { -End },
        pbrush,
        Chord,
    );

    BRUSH_ShareUnlockBrush(pbrush);
    ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntDrawArc(
    dc: PDC,
    XLeft: i32,
    YLeft: i32,
    Width: i32,
    Height: i32,
    StartArc: f64,
    EndArc: f64,
    arctype: u32,
    pbrush: PBRUSH,
) -> i32 {
    
    let Start = ceil(StartArc) as i32;
    let End = ceil(EndArc) as i32;
    let Chord = if arctype == GdiTypeChord { 1 } else { 0 };
    
    app_draw_arc(
        dc,
        rect(XLeft, YLeft, Width, Height),
        if (*dc).dclevel.flPath & DCPATH_CLOCKWISE != 0 { -End } else { -Start },
        if (*dc).dclevel.flPath & DCPATH_CLOCKWISE != 0 { -Start } else { -End },
        pbrush,
        Chord,
    )
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntDrawArc(
    dc: PDC,
    XLeft: i32,
    YLeft: i32,
    Width: i32,
    Height: i32,
    StartArc: f64,
    EndArc: f64,
    arctype: u32,
    pbrush: PBRUSH,
) -> i32 {
    
    let Start = ceil(StartArc) as i32;
    let End = ceil(EndArc) as i32;
    let Chord = if arctype == GdiTypeChord { 1 } else { 0 };
    
    app_draw_arc(
        dc,
        rect(XLeft, YLeft, Width, Height),
        if (*dc).dclevel.flPath & DCPATH_CLOCKWISE != 0 { -End } else { -Start },
        if (*dc).dclevel.flPath & DCPATH_CLOCKWISE != 0 { -Start } else { -End },
        pbrush,
        Chord,
    )
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntDrawEllipse(
    dc: PDC,
    XLeft: i32,
    YLeft: i32,
    Width: i32,
    Height: i32,
    pbrush: PBRUSH,
) -> i32 {
    app_draw_ellipse(dc, rect(XLeft, YLeft, Width, Height), pbrush)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntDrawEllipse(
    dc: PDC,
    XLeft: i32,
    YLeft: i32,
    Width: i32,
    Height: i32,
    pbrush: PBRUSH,
) -> i32 {
    app_draw_ellipse(dc, rect(XLeft, YLeft, Width, Height), pbrush)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntFillEllipse(
    dc: PDC,
    XLeft: i32,
    YLeft: i32,
    Width: i32,
    Height: i32,
    pbrush: PBRUSH,
) -> i32 {
    app_fill_ellipse(dc, rect(XLeft, YLeft, Width, Height), pbrush)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntFillEllipse(
    dc: PDC,
    XLeft: i32,
    YLeft: i32,
    Width: i32,
    Height: i32,
    pbrush: PBRUSH,
) -> i32 {
    app_fill_ellipse(dc, rect(XLeft, YLeft, Width, Height), pbrush)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntFillRoundRect(
    dc: PDC,
    Left: i32,
    Top: i32,
    Right: i32,
    Bottom: i32,
    Wellipse: i32,
    Hellipse: i32,
    pbrush: PBRUSH,
) -> u8 {

    let mut r: Rect;
    let rx: i32;
    let ry: i32;

    r = rect(Left, Top, abs(Right - Left), abs(Bottom - Top));
    rx = Wellipse / 2;
    ry = Hellipse / 2;

    if Wellipse > r.width {
        if Hellipse > r.height {
            app_fill_ellipse(dc, r, pbrush);
        } else {
            app_fill_arc(
                dc,
                rect(r.x, r.y, r.width - 1, Hellipse),
                0,
                180,
                pbrush,
                0,
            );
            app_fill_arc(
                dc,
                rect(r.x, Bottom - Hellipse - 1, r.width - 1, Hellipse),
                180,
                360,
                pbrush,
                0,
            );
        }
    } else if Hellipse > r.height {
        app_fill_arc(
            dc,
            rect(r.x, r.y, Wellipse, r.height - 1),
            90,
            270,
            pbrush,
            0,
        );
        app_fill_arc(
            dc,
            rect(Right - Wellipse - 1, r.y, Wellipse, r.height - 1),
            270,
            90,
            pbrush,
            0,
        );
    } else {
        app_fill_arc(
            dc,
            rect(r.x, r.y, rx + rx, ry + ry),
            90,
            180,
            pbrush,
            0,
        );

        app_fill_arc(
            dc,
            rect(r.x, r.y + r.height - ry - ry, rx + rx, ry + ry),
            180,
            270,
            pbrush,
            0,
        );

        app_fill_arc(
            dc,
            rect(r.x + r.width - rx - rx, r.y + r.height - ry - ry, rx + rx, ry + ry),
            270,
            360,
            pbrush,
            0,
        );

        app_fill_arc(
            dc,
            rect(r.x + r.width - rx - rx, r.y, rx + rx, ry + ry),
            0,
            90,
            pbrush,
            0,
        );
    }

    if Wellipse < r.width {
        IntFillRect(dc, r.x + rx, r.y, r.width - rx - rx, ry + 1, pbrush, 0);
        IntFillRect(
            dc,
            r.x + rx,
            r.y + r.height - ry + 1,
            r.width - rx - rx,
            ry - 1,
            pbrush,
            0,
        );
    }

    if Hellipse < r.height {
        IntFillRect(dc, r.x, r.y + ry + 1, r.width, r.height - ry - ry, pbrush, 0);
    }

    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntFillRoundRect(
    dc: PDC,
    Left: i32,
    Top: i32,
    Right: i32,
    Bottom: i32,
    Wellipse: i32,
    Hellipse: i32,
    pbrush: PBRUSH,
) -> u8 {

    let mut r: Rect;
    let rx: i32;
    let ry: i32;

    r = rect(Left, Top, abs(Right - Left), abs(Bottom - Top));
    rx = Wellipse / 2;
    ry = Hellipse / 2;

    if Wellipse > r.width {
        if Hellipse > r.height {
            app_fill_ellipse(dc, r, pbrush);
        } else {
            app_fill_arc(
                dc,
                rect(r.x, r.y, r.width - 1, Hellipse),
                0,
                180,
                pbrush,
                0,
            );
            app_fill_arc(
                dc,
                rect(r.x, Bottom - Hellipse - 1, r.width - 1, Hellipse),
                180,
                360,
                pbrush,
                0,
            );
        }
    } else if Hellipse > r.height {
        app_fill_arc(
            dc,
            rect(r.x, r.y, Wellipse, r.height - 1),
            90,
            270,
            pbrush,
            0,
        );
        app_fill_arc(
            dc,
            rect(Right - Wellipse - 1, r.y, Wellipse, r.height - 1),
            270,
            90,
            pbrush,
            0,
        );
    } else {
        app_fill_arc(
            dc,
            rect(r.x, r.y, rx + rx, ry + ry),
            90,
            180,
            pbrush,
            0,
        );

        app_fill_arc(
            dc,
            rect(r.x, r.y + r.height - ry - ry, rx + rx, ry + ry),
            180,
            270,
            pbrush,
            0,
        );

        app_fill_arc(
            dc,
            rect(r.x + r.width - rx - rx, r.y + r.height - ry - ry, rx + rx, ry + ry),
            270,
            360,
            pbrush,
            0,
        );

        app_fill_arc(
            dc,
            rect(r.x + r.width - rx - rx, r.y, rx + rx, ry + ry),
            0,
            90,
            pbrush,
            0,
        );
    }

    if Wellipse < r.width {
        IntFillRect(dc, r.x + rx, r.y, r.width - rx - rx, ry + 1, pbrush, 0);
        IntFillRect(
            dc,
            r.x + rx,
            r.y + r.height - ry + 1,
            r.width - rx - rx,
            ry - 1,
            pbrush,
            0,
        );
    }

    if Hellipse < r.height {
        IntFillRect(dc, r.x, r.y + ry + 1, r.width, r.height - ry - ry, pbrush, 0);
    }

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntDrawRoundRect(
    dc: PDC,
    Left: i32,
    Top: i32,
    Right: i32,
    Bottom: i32,
    Wellipse: i32,
    Hellipse: i32,
    pbrushPen: PBRUSH,
) -> u8 {
    let mut r: Rect;
    let mut rx: i32;
    let mut ry: i32;
    let w: i32 = (*pbrushbody(pbrushPen)).lWidth;

    r = rect(Left, Top, abs(Right - Left), abs(Bottom - Top));
    rx = Wellipse / 2;
    ry = Hellipse / 2;

    if Wellipse > r.width {
        if Hellipse > r.height {
            app_draw_ellipse(dc, r, pbrushPen);
        } else {
            app_draw_arc(
                dc,
                rect(r.x, r.y, r.width - 1, Hellipse - 1),
                0,
                180,
                pbrushPen,
                0,
            );
            app_draw_arc(
                dc,
                rect(r.x, Bottom - Hellipse, r.width - 1, Hellipse - 1),
                180,
                360,
                pbrushPen,
                0,
            );
        }
    } else if Hellipse > r.height {
        app_draw_arc(
            dc,
            rect(r.x, r.y, Wellipse - 1, r.height - 1),
            90,
            270,
            pbrushPen,
            0,
        );
        app_draw_arc(
            dc,
            rect(Right - Wellipse, r.y, Wellipse - 1, r.height - 1),
            270,
            90,
            pbrushPen,
            0,
        );
    } else {
        app_draw_arc(dc, rect(r.x, r.y, rx + rx, ry + ry), 90, 180, pbrushPen, 0);

        app_draw_arc(
            dc,
            rect(r.x, r.y + r.height - ry - ry, rx + rx, ry + ry),
            180,
            270,
            pbrushPen,
            0,
        );

        app_draw_arc(
            dc,
            rect(r.x + r.width - rx - rx, r.y + r.height - ry - ry, rx + rx, ry + ry),
            270,
            360,
            pbrushPen,
            0,
        );

        app_draw_arc(
            dc,
            rect(r.x + r.width - rx - rx, r.y, rx + rx, ry + ry),
            0,
            90,
            pbrushPen,
            0,
        );
    }
    if Hellipse < r.height {
        IntFillRect(
            dc,
            r.x,
            r.y + ry + 1,
            w,
            r.height - ry - ry,
            pbrushPen,
            1,
        );

        IntFillRect(
            dc,
            r.x + r.width - w,
            r.y + ry + 1,
            w,
            r.height - ry - ry,
            pbrushPen,
            1,
        );
    }
    if Wellipse < r.width {
        IntFillRect(
            dc,
            r.x + rx,
            r.y + r.height - w,
            r.width - rx - rx,
            w,
            pbrushPen,
            1,
        );

        IntFillRect(dc, r.x + rx, r.y, r.width - rx - rx, w, pbrushPen, 1);
    }
    1
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntDrawRoundRect(
    dc: PDC,
    Left: i32,
    Top: i32,
    Right: i32,
    Bottom: i32,
    Wellipse: i32,
    Hellipse: i32,
    pbrushPen: PBRUSH,
) -> u8 {
    let mut r: Rect;
    let mut rx: i32;
    let mut ry: i32;
    let w: i32 = (*pbrushbody(pbrushPen)).lWidth;

    r = rect(Left, Top, abs(Right - Left), abs(Bottom - Top));
    rx = Wellipse / 2;
    ry = Hellipse / 2;

    if Wellipse > r.width {
        if Hellipse > r.height {
            app_draw_ellipse(dc, r, pbrushPen);
        } else {
            app_draw_arc(
                dc,
                rect(r.x, r.y, r.width - 1, Hellipse - 1),
                0,
                180,
                pbrushPen,
                0,
            );
            app_draw_arc(
                dc,
                rect(r.x, Bottom - Hellipse, r.width - 1, Hellipse - 1),
                180,
                360,
                pbrushPen,
                0,
            );
        }
    } else if Hellipse > r.height {
        app_draw_arc(
            dc,
            rect(r.x, r.y, Wellipse - 1, r.height - 1),
            90,
            270,
            pbrushPen,
            0,
        );
        app_draw_arc(
            dc,
            rect(Right - Wellipse, r.y, Wellipse - 1, r.height - 1),
            270,
            90,
            pbrushPen,
            0,
        );
    } else {
        app_draw_arc(dc, rect(r.x, r.y, rx + rx, ry + ry), 90, 180, pbrushPen, 0);

        app_draw_arc(
            dc,
            rect(r.x, r.y + r.height - ry - ry, rx + rx, ry + ry),
            180,
            270,
            pbrushPen,
            0,
        );

        app_draw_arc(
            dc,
            rect(r.x + r.width - rx - rx, r.y + r.height - ry - ry, rx + rx, ry + ry),
            270,
            360,
            pbrushPen,
            0,
        );

        app_draw_arc(
            dc,
            rect(r.x + r.width - rx - rx, r.y, rx + rx, ry + ry),
            0,
            90,
            pbrushPen,
            0,
        );
    }
    if Hellipse < r.height {
        IntFillRect(
            dc,
            r.x,
            r.y + ry + 1,
            w,
            r.height - ry - ry,
            pbrushPen,
            1,
        );

        IntFillRect(
            dc,
            r.x + r.width - w,
            r.y + ry + 1,
            w,
            r.height - ry - ry,
            pbrushPen,
            1,
        );
    }
    if Wellipse < r.width {
        IntFillRect(
            dc,
            r.x + rx,
            r.y + r.height - w,
            r.width - rx - rx,
            w,
            pbrushPen,
            1,
        );

        IntFillRect(dc, r.x + rx, r.y, r.width - rx - rx, w, pbrushPen, 1);
    }
    1
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
// App Software Licence
// --------------------
// This package includes software which is copyright (c) L. Patrick.
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the distribution.
// 3. You may not sell this software package.
// 4. You may include this software in a distribution of other software,
//    and you may charge a nominal fee for the media used.
// 5. You may sell derivative programs, providing that such programs
//    simply use this software in a compiled form.
// 6. You may sell derivative programs which use a compiled, modified
//    version of this software, provided that you have attempted as
//    best as you can to propagate all modifications made to the source
//    code files of this software package back to the original author(s)
//    of this package.
//
// THIS SOFTWARE IS PROVIDED BY THE AUTHORS AND CONTRIBUTORS AS IS, AND
// ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHORS OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
// OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
// HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT
// LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY
// OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
// SUCH DAMAGE.
// */
// /* Copyright (c) L. Patrick
//
//    This file is part of the App cross-platform programming package.
//    You may redistribute it and/or modify it under the terms of the
//    App Software License. See the file LICENSE.TXT for details.
//
//    http://enchantia.com/software/graphapp/
//    http://www.it.usyd.edu.au/~graphapp/
// */
// /*
//     Modified for ReactOS
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
//
// #define DEGREES_TO_RADIANS(deg) ((deg)*2*M_PI/360)
//
// typedef struct _Rect
// {
//     int x, y;		/* Top-left point inside rect */
//     int width, height;	/* Width and height of rect */
// } Rect, *PRect;
//
// int FASTCALL IntFillRect(DC *dc, INT XLeft, INT YLeft, INT Width, INT Height, PBRUSH pbrush, BOOL Pen);
// //int FASTCALL app_fill_rect(DC *dc, Rect r, PBRUSH pbrush, BOOL Pen);
//
// static
// POINT
// NTAPI
// app_new_point(int x, int y)
// {
//     POINT p;
//     p.x = x;
//     p.y = y;
//     return p;
// }
// #define pt(x,y)       app_new_point((x),(y))
//
// static
// Rect
// NTAPI
// rect(int x, int y, int width, int height)
// {
//     Rect r;
//     r.x = x;
//     r.y = y;
//     r.width = width;
//     r.height = height;
//     return r;
// }
//
//
// /*
//  *  app_window_fill_rect:
//  *
//  *  Fill a rectangle with colour, in a window.
//  *
//  *  This function implements client-side clipping, so that
//  *  we never rely on the GDI system to do clipping, except if
//  *  the destination is a window which is partially obscured.
//  *  In that situation we must rely on the GDI system because there
//  *  is no way for the program to know which portions of the
//  *  window are currently obscured.
//  */
// #define app_fill_rect( dc, r, BrushObj, Pen) \
//         IntFillRect(dc, r.x, r.y, r.width, r.height, BrushObj, Pen)
//
// /*
//  *  Drawing an ellipse with a certain line thickness.
//  *  Use an inner and and outer ellipse and fill the spaces between.
//  *  The inner ellipse uses all UPPERCASE letters, the outer lowercase.
//  *
//  *  This algorithm is based on the fill_ellipse algorithm presented
//  *  above, but uses two ellipse calculations, and some fix-up code
//  *  to avoid pathological cases where the inner ellipse is almost
//  *  the same size as the outer (in which case the border of the
//  *  elliptical curve might otherwise have appeared broken).
//  */
// static
// int
// NTAPI
// app_draw_ellipse(DC *g, Rect r, PBRUSH pbrush)
// {
//     /* Outer ellipse: e(x,y) = b*b*x*x + a*a*y*y - a*a*b*b */
//
//     int a = r.width / 2;
//     int b = r.height / 2;
//     int x = 0;
//     int y = b;
//     long a2 = a*a;
//     long b2 = b*b;
//     long xcrit = (3 * a2 / 4) + 1;
//     long ycrit = (3 * b2 / 4) + 1;
//     long t = b2 + a2 - 2*a2*b;	/* t = e(x+1,y-1) */
//     long dxt = b2*(3+x+x);
//     long dyt = a2*(3-y-y);
//     int d2xt = b2+b2;
//     int d2yt = a2+a2;
//
//     int w = pbrush->lWidth;
//
//     /* Inner ellipse: E(X,Y) = B*B*X*X + A*A*Y*Y - A*A*B*B */
//
//     int A = a-w > 0 ? a-w : 0;
//     int B = b-w > 0 ? b-w : 0;
//     int X = 0;
//     int Y = B;
//     long A2 = A*A;
//     long B2 = B*B;
//     long XCRIT = (3 * A2 / 4) + 1;
//     long YCRIT = (3 * B2 / 4) + 1;
//     long T = B2 + A2 - 2*A2*B;	/* T = E(X+1,Y-1) */
//     long DXT = B2*(3+X+X);
//     long DYT = A2*(3-Y-Y);
//     int D2XT = B2+B2;
//     int D2YT = A2+A2;
//
//     int movedown, moveout;
//     int innerX = 0, prevx, prevy, W;
//     Rect r1, r2;
//     int result = 1;
//
//     if ((r.width <= 2) || (r.height <= 2))
//         return app_fill_rect(g, r, pbrush, TRUE);
//
//     r1.x = r.x + a;
//     r1.y = r.y;
//     r1.width = r.width & 1; /* i.e. if width is odd */
//     r1.height = 1;
//
//     r2 = r1;
//     r2.y = r.y + r.height - 1;
//
//     prevx = r1.x;
//     prevy = r1.y;
//
//     while (y > 0)
//     {
//         while (Y == y)
//         {
//             innerX = X;
//
//             if (T + A2*Y < XCRIT) /* E(X+1,Y-1/2) <= 0 */
//             {
//                 /* Move outwards to encounter edge */
//                 X += 1;
//                 T += DXT;
//                 DXT += D2XT;
//             }
//             else if (T - B2*X >= YCRIT) /* e(x+1/2,y-1) > 0 */
//             {
//                 /* Drop down one line */
//                 Y -= 1;
//                 T += DYT;
//                 DYT += D2YT;
//             }
//             else
//             {
//                 /* Drop diagonally down and out */
//                 X += 1;
//                 Y -= 1;
//                 T += DXT + DYT;
//                 DXT += D2XT;
//                 DYT += D2YT;
//             }
//         }
//
//         movedown = moveout = 0;
//
//         W = x - innerX;
//         if (r1.x + W < prevx)
//             W = prevx - r1.x;
//         if (W < w)
//             W = w;
//
//         if (t + a2*y < xcrit) /* e(x+1,y-1/2) <= 0 */
//         {
//             /* Move outwards to encounter edge */
//             x += 1;
//             t += dxt;
//             dxt += d2xt;
//
//             moveout = 1;
//         }
//         else if (t - b2*x >= ycrit) /* e(x+1/2,y-1) > 0 */
//         {
//             /* Drop down one line */
//             y -= 1;
//             t += dyt;
//             dyt += d2yt;
//
//             movedown = 1;
//         }
//         else
//         {
//             /* Drop diagonally down and out */
//             x += 1;
//             y -= 1;
//             t += dxt + dyt;
//             dxt += d2xt;
//             dyt += d2yt;
//
//             movedown = 1;
//             moveout = 1;
//         }
//
//         if (movedown)
//         {
//             if (r1.width == 0)
//             {
//                 r1.x -= 1;
//                 r1.width += 2;
//                 r2.x -= 1;
//                 r2.width += 2;
//                 moveout = 0;
//             }
//
//             if (r1.x < r.x)
//                 r1.x = r2.x = r.x;
//             if (r1.width > r.width)
//                 r1.width = r2.width = r.width;
//             if (r1.y == r2.y-1)
//             {
//                 r1.x = r2.x = r.x;
//                 r1.width = r2.width = r.width;
//             }
//
//             if ((r1.y < r.y+w) || (r1.x+W >= r1.x+r1.width-W))
//             {
//                 result &= app_fill_rect(g, r1, pbrush, TRUE);
//                 result &= app_fill_rect(g, r2, pbrush, TRUE);
//
//                 prevx = r1.x;
//                 prevy = r1.y;
//             }
//             else if (r1.y+r1.height < r2.y)
//             {
//                 /* Draw distinct rectangles */
//                 result &= app_fill_rect(g, rect(r1.x,r1.y,
//                                                 W,1), pbrush, TRUE);
//                 result &= app_fill_rect(g, rect(
//                                             r1.x+r1.width-W,r1.y,W,1), pbrush, TRUE);
//                 result &= app_fill_rect(g, rect(r2.x,
//                                                 r2.y,W,1), pbrush, TRUE);
//                 result &= app_fill_rect(g, rect(
//                                             r2.x+r2.width-W,r2.y,W,1), pbrush, TRUE);
//
//                 prevx = r1.x;
//                 prevy = r1.y;
//             }
//
//             /* Move down */
//             r1.y += 1;
//             r2.y -= 1;
//         }
//
//         if (moveout)
//         {
//             /* Move outwards */
//             r1.x -= 1;
//             r1.width += 2;
//             r2.x -= 1;
//             r2.width += 2;
//         }
//     }
//     if ((x <= a) && (prevy < r2.y))
//     {
//         /* Draw final line */
//         r1.height = r1.y+r1.height-r2.y;
//         r1.y = r2.y;
//
//         W = w;
//         if (r.x + W != prevx)
//             W = prevx - r.x;
//         if (W < w)
//             W = w;
//
//         if (W+W >= r.width)
//         {
//             result &= app_fill_rect(g, rect(r.x, r1.y,
//                                             r.width, r1.height), pbrush, TRUE);
//             return result;
//         }
//
//         result &= app_fill_rect(g, rect(r.x, r1.y, W, r1.height), pbrush, TRUE);
//         result &= app_fill_rect(g, rect(r.x+r.width-W, r1.y,
//                                         W, r1.height), pbrush, TRUE);
//     }
//     return result;
// }
//
// /*
//  *  Draw an arc of an ellipse from start_angle anti-clockwise to
//  *  end_angle. If the angles coincide, draw nothing; if they
//  *  differ by 360 degrees or more, draw a full ellipse.
//  *  The shape is drawn with the current line thickness,
//  *  completely within the bounding rectangle. The shape is also
//  *  axis-aligned, so that the ellipse would be horizontally and
//  *  vertically symmetric is it was complete.
//  *
//  *  The draw_arc algorithm is based on draw_ellipse, but unlike
//  *  that algorithm is not symmetric in the general case, since
//  *  an angular portion is clipped from the shape.
//  *  This clipping is performed by keeping track of two hypothetical
//  *  lines joining the centre point to the enclosing rectangle,
//  *  at the angles start_angle and end_angle, using a line-intersection
//  *  algorithm. Essentially the algorithm just fills the spaces
//  *  which are within the arc and also between the angles, going
//  *  in an anti-clockwise direction from start_angle to end_angle.
//  *  In the top half of the ellipse, this amounts to drawing
//  *  to the left of the start_angle line and to the right of
//  *  the end_angle line, while in the bottom half of the ellipse,
//  *  it involves drawing to the right of the start_angle and to
//  *  the left of the end_angle.
//  */
//
// /*
//  *  Fill a rectangle within an arc, given the centre point p0,
//  *  and the two end points of the lines corresponding to the
//  *  start_angle and the end_angle. This function takes care of
//  *  the logic needed to swap the fill direction below
//  *  the central point, and also performs the calculations
//  *  needed to intersect the current Y value with each line.
//  */
// static
// int
// FASTCALL
// app_fill_arc_rect(DC *g,
//                   Rect r,   // top, left, width, height
//                   POINT p0, // Center
//                   POINT p1, // Start
//                   POINT p2, // End
//                   int start_angle,
//                   int end_angle,
//                   PBRUSH pbrush,
//                   BOOL Pen)
// {
//     int x1, x2;
//     int start_above, end_above;
//     long rise1, run1, rise2, run2;
//
//     rise1 = p1.y - p0.y;
//     run1  = p1.x - p0.x;
//     rise2 = p2.y - p0.y;
//     run2  = p2.x - p0.x;
//
//     if (r.y <= p0.y) //
//     {
//         /* In top half of arc ellipse */
//
//         if (p1.y <= r.y)
//         {
//             /* Start_line is in the top half and is */
//             /* intersected by the current Y scan line */
//             if (rise1 == 0)
//                 x1 = p1.x;
//             else
//                 x1 = p0.x + (r.y-p0.y)*run1/rise1;
//             start_above = 1;
//         }
//         else if ((start_angle >= 0) && (start_angle <= 180))
//         {
//             /* Start_line is above middle */
//             x1 = p1.x;
//             start_above = 1;
//         }
//         else
//         {
//             /* Start_line is below middle */
//             x1 = r.x + r.width;
//             start_above = 0;
//         }
//         if (x1 < r.x)
//             x1 = r.x;
//         if (x1 > r.x+r.width)
//             x1 = r.x+r.width;
//
//         if (p2.y <= r.y)
//         {
//             /* end_line is in the top half and is */
//             /* intersected by the current Y scan line */
//             if (rise2 == 0)
//                 x2 = p2.x;
//             else
//                 x2 = p0.x + (r.y-p0.y)*run2/rise2;
//             end_above = 1;
//         }
//         else if ((end_angle >= 0) && (end_angle <= 180))
//         {
//             /* end_line is above middle */
//             x2 = p2.x;
//             end_above = 1;
//         }
//         else
//         {
//             /* end_line is below middle */
//             x2 = r.x;
//             end_above = 0;
//         }
//
//         if (x2 < r.x) x2 = r.x;
//
//         if (x2 > r.x+r.width) x2 = r.x+r.width;
//
//         if (start_above && end_above)
//         {
//             if (start_angle > end_angle)
//             {
//                 /* Fill outsides of wedge */
//                 if (! app_fill_rect(g, rect(r.x, r.y,
//                                             x1-r.x, r.height), pbrush, Pen))
//                     return 0;
//                 return app_fill_rect(g, rect(x2, r.y,
//                                              r.x+r.width-x2, r.height), pbrush, Pen);
//             }
//             else
//             {
//                 /* Fill inside of wedge */
//                 r.width = x1-x2;
//                 r.x = x2;
//                 return app_fill_rect(g, r, pbrush, Pen);
//             }
//         }
//         else if (start_above)
//         {
//             /* Fill to the left of the start_line */
//             r.width = x1-r.x;
//             return app_fill_rect(g, r, pbrush, Pen);
//         }
//         else if (end_above)
//         {
//             /* Fill right of end_line */
//             r.width = r.x+r.width-x2;
//             r.x = x2;
//             return app_fill_rect(g, r, pbrush, Pen);
//         }
//         else
//         {
//             if (start_angle > end_angle)
//                 return app_fill_rect(g,r, pbrush, Pen);
//             else
//                 return 1;
//         }
//     }
//     else
//     {
//         /* In lower half of arc ellipse */
//
//         if (p1.y >= r.y)
//         {
//             /* start_line is in the lower half and is */
//             /* intersected by the current Y scan line */
//             if (rise1 == 0)
//                 x1 = p1.x;
//             else
//                 x1 = p0.x + (r.y-p0.y)*run1/rise1;
//             start_above = 0;
//         }
//         else if ((start_angle >= 180) && (start_angle <= 360))
//         {
//             /* start_line is below middle */
//             x1 = p1.x;
//             start_above = 0;
//         }
//         else
//         {
//             /* start_line is above middle */
//             x1 = r.x;
//             start_above = 1;
//         }
//         if (x1 < r.x)
//             x1 = r.x;
//         if (x1 > r.x+r.width)
//             x1 = r.x+r.width;
//
//         if (p2.y >= r.y)
//         {
//             /* end_line is in the lower half and is */
//             /* intersected by the current Y scan line */
//             if (rise2 == 0)
//                 x2 = p2.x;
//             else
//                 x2 = p0.x + (r.y-p0.y)*run2/rise2;
//             end_above = 0;
//         }
//         else if ((end_angle >= 180) && (end_angle <= 360))
//         {
//             /* end_line is below middle */
//             x2 = p2.x;
//             end_above = 0;
//         }
//         else
//         {
//             /* end_line is above middle */
//             x2 = r.x + r.width;
//             end_above = 1;
//         }
//         if (x2 < r.x)
//             x2 = r.x;
//         if (x2 > r.x+r.width)
//             x2 = r.x+r.width;
//
//         if (start_above && end_above)
//         {
//             if (start_angle > end_angle)
//                 return app_fill_rect(g,r, pbrush, Pen);
//             else
//                 return 1;
//         }
//         else if (start_above)
//         {
//             /* Fill to the left of end_line */
//             r.width = x2-r.x;
//             return app_fill_rect(g,r, pbrush, Pen);
//         }
//         else if (end_above)
//         {
//             /* Fill right of start_line */
//             r.width = r.x+r.width-x1;
//             r.x = x1;
//             return app_fill_rect(g,r, pbrush, Pen);
//         }
//         else
//         {
//             if (start_angle > end_angle)
//             {
//                 /* Fill outsides of wedge */
//                 if (! app_fill_rect(g, rect(r.x, r.y,
//                                             x2-r.x, r.height), pbrush, Pen))
//                     return 0;
//                 return app_fill_rect(g, rect(x1, r.y,
//                                              r.x+r.width-x1, r.height), pbrush, Pen);
//             }
//             else
//             {
//                 /* Fill inside of wedge */
//                 r.width = x2-x1;
//                 r.x = x1;
//                 return app_fill_rect(g, r, pbrush, Pen);
//             }
//         }
//     }
// }
//
// /*
//  *  To fill an axis-aligned ellipse, we use a scan-line algorithm.
//  *  We walk downwards from the top Y co-ordinate, calculating
//  *  the width of the ellipse using incremental integer arithmetic.
//  *  To save calculation, we observe that the top and bottom halves
//  *  of the ellipsoid are mirror-images, therefore we can draw the
//  *  top and bottom halves by reflection. As a result, this algorithm
//  *  draws rectangles inwards from the top and bottom edges of the
//  *  bounding rectangle.
//  *
//  *  To save rendering time, draw as few rectangles as possible.
//  *  Other ellipse-drawing algorithms assume we want to draw each
//  *  line, using a draw_pixel operation, or a draw_horizontal_line
//  *  operation. This approach is slower than it needs to be in
//  *  circumstances where a fill_rect operation is more efficient
//  *  (such as in X-Windows, where there is a communication overhead
//  *  to the X-Server). For this reason, the algorithm accumulates
//  *  rectangles on adjacent lines which have the same width into a
//  *  single larger rectangle.
//  *
//  *  This algorithm forms the basis of the later, more complex,
//  *  draw_ellipse algorithm, which renders the rectangular spaces
//  *  between an outer and inner ellipse, and also the draw_arc and
//  *  fill_arc operations which additionally clip drawing between
//  *  a start_angle and an end_angle.
//  *
//  */
// static
// int
// FASTCALL
// app_fill_ellipse(DC *g, Rect r, PBRUSH pbrush)
// {
//     /* e(x,y) = b*b*x*x + a*a*y*y - a*a*b*b */
//
//     int a = r.width / 2;
//     int b = r.height / 2;
//     int x = 0;
//     int y = b;
//     long a2 = a*a;
//     long b2 = b*b;
//     long xcrit = (3 * a2 / 4) + 1;
//     long ycrit = (3 * b2 / 4) + 1;
//     long t = b2 + a2 - 2*a2*b;	/* t = e(x+1,y-1) */
//     long dxt = b2*(3+x+x);
//     long dyt = a2*(3-y-y);
//     int d2xt = b2+b2;
//     int d2yt = a2+a2;
//     Rect r1, r2;
//     int result = 1;
//
//     if ((r.width <= 2) || (r.height <= 2))
//         return app_fill_rect(g, r, pbrush, FALSE);
//
//     r1.x = r.x + a;
//     r1.y = r.y;
//     r1.width = r.width & 1; /* i.e. if width is odd */
//     r1.height = 1;
//
//     r2 = r1;
//     r2.y = r.y + r.height - 1;
//
//     while (y > 0)
//     {
//         if (t + a2*y < xcrit)   /* e(x+1,y-1/2) <= 0 */
//         {
//             /* Move outwards to encounter edge */
//             x += 1;
//             t += dxt;
//             dxt += d2xt;
//
//             /* Move outwards */
//             r1.x -= 1;
//             r1.width += 2;
//             r2.x -= 1;
//             r2.width += 2;
//         }
//         else if (t - b2*x >= ycrit)   /* e(x+1/2,y-1) > 0 */
//         {
//             /* Drop down one line */
//             y -= 1;
//             t += dyt;
//             dyt += d2yt;
//
//             /* Enlarge rectangles */
//             r1.height += 1;
//             r2.height += 1;
//             r2.y -= 1;
//         }
//         else
//         {
//             /* Drop diagonally down and out */
//             x += 1;
//             y -= 1;
//             t += dxt + dyt;
//             dxt += d2xt;
//             dyt += d2yt;
//
//             if ((r1.width > 0) && (r1.height > 0))
//             {
//                 /* Draw rectangles first */
//
//                 if (r1.y+r1.height < r2.y)
//                 {
//                     /* Distinct rectangles */
//                     result &= app_fill_rect(g, r1, pbrush, FALSE);
//                     result &= app_fill_rect(g, r2, pbrush, FALSE);
//                 }
//
//                 /* Move down */
//                 r1.y += r1.height;
//                 r1.height = 1;
//                 r2.y -= 1;
//                 r2.height = 1;
//             }
//             else
//             {
//                 /* Skipped pixels on initial diagonal */
//
//                 /* Enlarge, rather than moving down */
//                 r1.height += 1;
//                 r2.height += 1;
//                 r2.y -= 1;
//             }
//
//             /* Move outwards */
//             r1.x -= 1;
//             r1.width += 2;
//             r2.x -= 1;
//             r2.width += 2;
//         }
//     }
//     if (r1.y < r2.y)
//     {
//         /* Overlap */
//         r1.x = r.x;
//         r1.width = r.width;
//         r1.height = r2.y+r2.height-r1.y;
//         result &= app_fill_rect(g, r1, pbrush, FALSE);
//     }
//     else if (x <= a)
//     {
//         /* Crossover, draw final line */
//         r1.x = r.x;
//         r1.width = r.width;
//         r1.height = r1.y+r1.height-r2.y;
//         r1.y = r2.y;
//         result &= app_fill_rect(g, r1, pbrush, FALSE);
//     }
//     return result;
// }
//
// static
// POINT
// FASTCALL
// app_boundary_point(Rect r, int angle)
// {
//     int cx, cy;
//     double tangent;
//
//     cx = r.width;
//     cx /= 2;
//     cx += r.x;
//
//     cy = r.height;
//     cy /= 2;
//     cy += r.y;
//
//     if (angle == 0)
//         return pt(r.x+r.width, cy);
//     else if (angle == 45)
//         return pt(r.x+r.width, r.y);
//     else if (angle == 90)
//         return pt(cx, r.y);
//     else if (angle == 135)
//         return pt(r.x, r.y);
//     else if (angle == 180)
//         return pt(r.x, cy);
//     else if (angle == 225)
//         return pt(r.x, r.y+r.height);
//     else if (angle == 270)
//         return pt(cx, r.y+r.height);
//     else if (angle == 315)
//         return pt(r.x+r.width, r.y+r.height);
//
//     tangent = tan(DEGREES_TO_RADIANS!(angle));
//
//     if ((angle > 45) && (angle < 135))
//         return pt((int)(cx+r.height/tangent/2), r.y);
//     else if ((angle > 225) && (angle < 315))
//         return pt((int)(cx-r.height/tangent/2), r.y+r.height);
//     else if ((angle > 135) && (angle < 225))
//         return pt(r.x, (int)(cy+r.width*tangent/2));
//     else
//         return pt(r.x+r.width, (int)(cy-r.width*tangent/2));
// }
//
// int
// FASTCALL
// app_fill_arc(DC *g, Rect r, int start_angle, int end_angle, PBRUSH pbrush, BOOL Chord)
// {
//     /* e(x,y) = b*b*x*x + a*a*y*y - a*a*b*b */
//
//     int a = r.width / 2;
//     int b = r.height / 2;
//     int x = 0;
//     int y = b;
//     long a2 = a*a;
//     long b2 = b*b;
//     long xcrit = (3 * a2 / 4) + 1;
//     long ycrit = (3 * b2 / 4) + 1;
//     long t = b2 + a2 - 2*a2*b;	/* t = e(x+1,y-1) */
//     long dxt = b2*(3+x+x);
//     long dyt = a2*(3-y-y);
//     int d2xt = b2+b2;
//     int d2yt = a2+a2;
//     Rect r1, r2;
//     int movedown, moveout;
//     int result = 1;
//
//     /* Line descriptions */
//     POINT p0, p1, p2;
//
//     /* If angles differ by 360 degrees or more, close the shape */
//     if ((start_angle + 360 <= end_angle) ||
//             (start_angle - 360 >= end_angle))
//     {
//         return app_fill_ellipse(g, r, pbrush);
//     }
//
//     /* Make start_angle >= 0 and <= 360 */
//     while (start_angle < 0)
//         start_angle += 360;
//     start_angle %= 360;
//
//     /* Make end_angle >= 0 and <= 360 */
//     while (end_angle < 0)
//         end_angle += 360;
//     end_angle %= 360;
//
//     /* Draw nothing if the angles are equal */
//     if (start_angle == end_angle)
//         return 1;
//
//     /* Find arc wedge line end points */
//     p1 = app_boundary_point(r, start_angle);
//     p2 = app_boundary_point(r, end_angle);
//     if (Chord)
//         p0 = pt((p1.x+p2.x)/2,(p1.y+p2.y)/2);
//     else
//         p0 = pt(r.x + r.width/2, r.y + r.height/2);
//
//     /* Initialise rectangles to be drawn */
//     r1.x = r.x + a;
//     r1.y = r.y;
//     r1.width = r.width & 1; /* i.e. if width is odd */
//     r1.height = 1;
//
//     r2 = r1;
//     r2.y = r.y + r.height - 1;
//
//     while (y > 0)
//     {
//         moveout = movedown = 0;
//
//         if (t + a2*y < xcrit)   /* e(x+1,y-1/2) <= 0 */
//         {
//             /* Move outwards to encounter edge */
//             x += 1;
//             t += dxt;
//             dxt += d2xt;
//
//             moveout = 1;
//         }
//         else if (t - b2*x >= ycrit)   /* e(x+1/2,y-1) > 0 */
//         {
//             /* Drop down one line */
//             y -= 1;
//             t += dyt;
//             dyt += d2yt;
//
//             movedown = 1;
//         }
//         else
//         {
//             /* Drop diagonally down and out */
//             x += 1;
//             y -= 1;
//             t += dxt + dyt;
//             dxt += d2xt;
//             dyt += d2yt;
//
//             moveout = 1;
//             movedown = 1;
//         }
//
//         if (movedown)
//         {
//             if (r1.width == 0)
//             {
//                 r1.x -= 1;
//                 r1.width += 2;
//                 r2.x -= 1;
//                 r2.width += 2;
//                 moveout = 0;
//             }
//
//             if (r1.x < r.x)
//                 r1.x = r2.x = r.x;
//             if (r1.width > r.width)
//                 r1.width = r2.width = r.width;
//             if (r1.y == r2.y-1)
//             {
//                 r1.x = r2.x = r.x;
//                 r1.width = r2.width = r.width;
//             }
//
//             if ((r1.width > 0) && (r1.y+r1.height < r2.y))
//             {
//                 /* Distinct rectangles */
//                 result &= app_fill_arc_rect(g, r1,
//                                             p0, p1, p2,
//                                             start_angle, end_angle, pbrush, FALSE);
//                 result &= app_fill_arc_rect(g, r2,
//                                             p0, p1, p2,
//                                             start_angle, end_angle, pbrush, FALSE);
//             }
//
//             /* Move down */
//             r1.y += 1;
//             r2.y -= 1;
//         }
//
//         if (moveout)
//         {
//             /* Move outwards */
//             r1.x -= 1;
//             r1.width += 2;
//             r2.x -= 1;
//             r2.width += 2;
//         }
//     }
//     if (r1.y < r2.y)
//     {
//         /* Overlap */
//         r1.x = r.x;
//         r1.width = r.width;
//         r1.height = r2.y+r2.height-r1.y;
//         while (r1.height > 0)
//         {
//             result &= app_fill_arc_rect(g,
//                                         rect(r1.x, r1.y, r1.width, 1),
//                                         p0, p1, p2, start_angle, end_angle, pbrush, FALSE);
//             r1.y += 1;
//             r1.height -= 1;
//         }
//     }
//     else if (x <= a)
//     {
//         /* Crossover, draw final line */
//         r1.x = r.x;
//         r1.width = r.width;
//         r1.height = r1.y+r1.height-r2.y;
//         r1.y = r2.y;
//         while (r1.height > 0)
//         {
//             result &= app_fill_arc_rect(g,
//                                         rect(r1.x, r1.y, r1.width, 1),
//                                         p0, p1, p2, start_angle, end_angle, pbrush, FALSE);
//             r1.y += 1;
//             r1.height -= 1;
//         }
//     }
//     return result;
// }
//
// int app_draw_arc(DC *g, Rect r, int start_angle, int end_angle, PBRUSH pbrushPen, BOOL Chord)
// {
//     /* Outer ellipse: e(x,y) = b*b*x*x + a*a*y*y - a*a*b*b */
//
//     int a = r.width / 2;
//     int b = r.height / 2;
//     int x = 0;
//     int y = b;
//     long a2 = a*a;
//     long b2 = b*b;
//     long xcrit = (3 * a2 / 4) + 1;
//     long ycrit = (3 * b2 / 4) + 1;
//     long t = b2 + a2 - 2*a2*b;	/* t = e(x+1,y-1) */
//     long dxt = b2*(3+x+x);
//     long dyt = a2*(3-y-y);
//     int d2xt = b2+b2;
//     int d2yt = a2+a2;
//
//     int w = pbrushPen->lWidth;
//
//     /* Inner ellipse: E(X,Y) = B*B*X*X + A*A*Y*Y - A*A*B*B */
//
//     int A = a-w > 0 ? a-w : 0;
//     int B = b-w > 0 ? b-w : 0;
//     int X = 0;
//     int Y = B;
//     long A2 = A*A;
//     long B2 = B*B;
//     long XCRIT = (3 * A2 / 4) + 1;
//     long YCRIT = (3 * B2 / 4) + 1;
//     long T = B2 + A2 - 2*A2*B;	/* T = E(X+1,Y-1) */
//     long DXT = B2*(3+X+X);
//     long DYT = A2*(3-Y-Y);
//     int D2XT = B2+B2;
//     int D2YT = A2+A2;
//
//     /* Arc rectangle calculations */
//     int movedown, moveout;
//     int innerX = 0, prevx, prevy, W;
//     Rect r1, r2;
//     int result = 1;
//
//     /* Line descriptions */
//     POINT p0, p1, p2;
//
//     /* If angles differ by 360 degrees or more, close the shape */
//     if ((start_angle + 360 <= end_angle) ||
//             (start_angle - 360 >= end_angle))
//     {
//         return app_draw_ellipse(g, r, pbrushPen);
//     }
//
//     /* Make start_angle >= 0 and <= 360 */
//     while (start_angle < 0)
//         start_angle += 360;
//     start_angle %= 360;
//
//     /* Make end_angle >= 0 and <= 360 */
//     while (end_angle < 0)
//         end_angle += 360;
//     end_angle %= 360;
//
//     /* Draw nothing if the angles are equal */
//     if (start_angle == end_angle)
//         return 1;
//
//     /* Find arc wedge line end points */
//     p1 = app_boundary_point(r, start_angle);
//     p2 = app_boundary_point(r, end_angle);
//     if (Chord)
//         p0 = pt((p1.x+p2.x)/2,(p1.y+p2.y)/2);
//     else
//         p0 = pt(r.x + r.width/2, r.y + r.height/2);
//
//     /* Determine ellipse rectangles */
//     r1.x = r.x + a;
//     r1.y = r.y;
//     r1.width = r.width & 1; /* i.e. if width is odd */
//     r1.height = 1;
//
//     r2 = r1;
//     r2.y = r.y + r.height - 1;
//
//     prevx = r1.x;
//     prevy = r1.y;
//
//     while (y > 0)
//     {
//         while (Y == y)
//         {
//             innerX = X;
//
//             if (T + A2*Y < XCRIT) /* E(X+1,Y-1/2) <= 0 */
//             {
//                 /* Move outwards to encounter edge */
//                 X += 1;
//                 T += DXT;
//                 DXT += D2XT;
//             }
//             else if (T - B2*X >= YCRIT) /* e(x+1/2,y-1) > 0 */
//             {
//                 /* Drop down one line */
//                 Y -= 1;
//                 T += DYT;
//                 DYT += D2YT;
//             }
//             else
//             {
//                 /* Drop diagonally down and out */
//                 X += 1;
//                 Y -= 1;
//                 T += DXT + DYT;
//                 DXT += D2XT;
//                 DYT += D2YT;
//             }
//         }
//
//         movedown = moveout = 0;
//
//         W = x - innerX;
//         if (r1.x + W < prevx)
//             W = prevx - r1.x;
//         if (W < w)
//             W = w;
//
//         if (t + a2*y < xcrit) /* e(x+1,y-1/2) <= 0 */
//         {
//             /* Move outwards to encounter edge */
//             x += 1;
//             t += dxt;
//             dxt += d2xt;
//
//             moveout = 1;
//         }
//         else if (t - b2*x >= ycrit) /* e(x+1/2,y-1) > 0 */
//         {
//             /* Drop down one line */
//             y -= 1;
//             t += dyt;
//             dyt += d2yt;
//
//             movedown = 1;
//         }
//         else
//         {
//             /* Drop diagonally down and out */
//             x += 1;
//             y -= 1;
//             t += dxt + dyt;
//             dxt += d2xt;
//             dyt += d2yt;
//
//             movedown = 1;
//             moveout = 1;
//         }
//
//         if (movedown)
//         {
//             if (r1.width == 0)
//             {
//                 r1.x -= 1;
//                 r1.width += 2;
//                 r2.x -= 1;
//                 r2.width += 2;
//                 moveout = 0;
//             }
//
//             if (r1.x < r.x)
//                 r1.x = r2.x = r.x;
//             if (r1.width > r.width)
//                 r1.width = r2.width = r.width;
//             if (r1.y == r2.y-1)
//             {
//                 r1.x = r2.x = r.x;
//                 r1.width = r2.width = r.width;
//             }
//
//             if ((r1.y < r.y+w) || (r1.x+W >= r1.x+r1.width-W))
//             {
//                 result &= app_fill_arc_rect(g, r1,
//                                             p0, p1, p2,
//                                             start_angle, end_angle, pbrushPen, TRUE);
//                 result &= app_fill_arc_rect(g, r2,
//                                             p0, p1, p2,
//                                             start_angle, end_angle, pbrushPen, TRUE);
//
//                 prevx = r1.x;
//                 prevy = r1.y;
//             }
//             else if (r1.y+r1.height < r2.y)
//             {
//                 /* Draw distinct rectangles */
//                 result &= app_fill_arc_rect(g, rect(
//                                                 r1.x,r1.y,W,1),
//                                             p0, p1, p2,
//                                             start_angle, end_angle, pbrushPen, TRUE);
//                 result &= app_fill_arc_rect(g, rect(
//                                                 r1.x+r1.width-W,r1.y,W,1),
//                                             p0, p1, p2,
//                                             start_angle, end_angle, pbrushPen, TRUE);
//                 result &= app_fill_arc_rect(g, rect(
//                                                 r2.x,r2.y,W,1),
//                                             p0, p1, p2,
//                                             start_angle, end_angle, pbrushPen, TRUE);
//                 result &= app_fill_arc_rect(g, rect(
//                                                 r2.x+r2.width-W,r2.y,W,1),
//                                             p0, p1, p2,
//                                             start_angle, end_angle, pbrushPen, TRUE);
//
//                 prevx = r1.x;
//                 prevy = r1.y;
//             }
//
//             /* Move down */
//             r1.y += 1;
//             r2.y -= 1;
//         }
//
//         if (moveout)
//         {
//             /* Move outwards */
//             r1.x -= 1;
//             r1.width += 2;
//             r2.x -= 1;
//             r2.width += 2;
//         }
//     }
//     if ((x <= a) && (prevy < r2.y))
//     {
//         /* Draw final lines */
//         r1.height = r1.y+r1.height-r2.y;
//         r1.y = r2.y;
//
//         W = w;
//         if (r.x + W != prevx)
//             W = prevx - r.x;
//         if (W < w)
//             W = w;
//
//         if (W+W >= r.width)
//         {
//             while (r1.height > 0)
//             {
//                 result &= app_fill_arc_rect(g, rect(r.x,
//                                                     r1.y, r.width, 1), p0, p1, p2,
//                                             start_angle, end_angle, pbrushPen, TRUE);
//                 r1.y += 1;
//                 r1.height -= 1;
//             }
//             return result;
//         }
//
//         while (r1.height > 0)
//         {
//             result &= app_fill_arc_rect(g, rect(r.x, r1.y,
//                                                 W, 1), p0, p1, p2,
//                                         start_angle, end_angle, pbrushPen, TRUE);
//             result &= app_fill_arc_rect(g, rect(r.x+r.width-W,
//                                                 r1.y, W, 1), p0, p1, p2,
//                                         start_angle, end_angle, pbrushPen, TRUE);
//             r1.y += 1;
//             r1.height -= 1;
//         }
//     }
//
//     return result;
// }
//
// /* ReactOS Interface *********************************************************/
//
// int
// FASTCALL
// IntFillRect( DC *dc,
//              INT XLeft,
//              INT YLeft,
//              INT Width,
//              INT Height,
//              PBRUSH pbrush,
//              BOOL Pen)
// {
//     DWORD ROP = ROP4_FROM_INDEX(R3_OPINDEX_PATCOPY);
//     RECTL DestRect;
//     SURFACE *psurf;
//     POINTL BrushOrigin;
//     BOOL Ret = TRUE;
//     PDC_ATTR pdcattr;
//
//     ASSERT(pbrush);
//     ASSERT_DC_PREPARED(dc);
//
//     psurf = dc->dclevel.pSurface;
//     if (psurf == NULL)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return 0;
//     }
//
//     if (!(pbrush->flAttrs & BR_IS_NULL))
//     {
//         pdcattr = dc->pdcattr;
//
//         /* Fix negative spaces */
//         if (Width < 0)
//         {
//             XLeft += Width;
//             Width = 0 - Width;
//         }
//         if (Height < 0)
//         {
//             YLeft += Height;
//             Height = 0 - Height;
//         }
//
//         DestRect.left = XLeft;
//         DestRect.right = XLeft + Width;
//
//         DestRect.top = YLeft;
//         DestRect.bottom = YLeft + Height;
//
//         BrushOrigin.x = pbrush->ptOrigin.x;
//         BrushOrigin.y = pbrush->ptOrigin.y;
//
//         if (pdcattr->jROP2 == R2_XORPEN)
//             ROP = ROP4_FROM_INDEX(R3_OPINDEX_PATINVERT);
//
//         Ret = IntEngBitBlt(
//                   &psurf->SurfObj,
//                   NULL,
//                   NULL,
//                   (CLIPOBJ *)&dc->co,
//                   NULL,
//                   &DestRect,
//                   NULL,
//                   NULL,
//                   Pen ? &dc->eboLine.BrushObject : &dc->eboFill.BrushObject,
//                   &BrushOrigin,
//                   ROP);
//     }
//
//     return (int)Ret;
// }
//
// BOOL
// FASTCALL
// IntFillArc( PDC dc,
//             INT XLeft,
//             INT YLeft,
//             INT Width,
//             INT Height,
//             double StartArc, // FIXME: don't use floating point!
//             double EndArc,
//             ARCTYPE arctype)
// {
//     PDC_ATTR pdcattr;
//     PBRUSH pbrush;
//     int Start = (int)ceil(StartArc);
//     int End   = (int)ceil(EndArc);
//     BOOL Chord = (arctype == GdiTypeChord), ret;
//
//     pdcattr = dc->pdcattr;
//
//     pbrush = BRUSH_ShareLockBrush(pdcattr->hbrush);
//     if (!pbrush)
//     {
//         DPRINT1("FillArc Fail\n");
//         EngSetLastError(ERROR_INTERNAL_ERROR);
//         return FALSE;
//     }
//     // Sort out alignment here.
//     ret = app_fill_arc(dc, rect( XLeft, YLeft, Width, Height),
//                        (dc->dclevel.flPath & DCPATH_CLOCKWISE) ? -End : -Start,
//                        (dc->dclevel.flPath & DCPATH_CLOCKWISE) ? -Start : -End,
//                        pbrush, Chord);
//
//     BRUSH_ShareUnlockBrush(pbrush);
//     return ret;
// }
//
// BOOL
// FASTCALL
// IntDrawArc( PDC dc,
//             INT XLeft,
//             INT YLeft,
//             INT Width,
//             INT Height,
//             double StartArc, // FIXME: don't use floating point!
//             double EndArc,
//             ARCTYPE arctype,
//             PBRUSH pbrush)
// {
//     int Start = (int)ceil(StartArc);
//     int End   = (int)ceil(EndArc);
//     BOOL Chord = (arctype == GdiTypeChord);
//     // Sort out alignment here.
//     return app_draw_arc(dc, rect( XLeft, YLeft, Width, Height),
//                         (dc->dclevel.flPath & DCPATH_CLOCKWISE) ? -End : -Start,
//                         (dc->dclevel.flPath & DCPATH_CLOCKWISE) ? -Start : -End,
//                         pbrush, Chord);
// }
//
// BOOL
// FASTCALL
// IntDrawEllipse( PDC dc,
//                 INT XLeft,
//                 INT YLeft,
//                 INT Width,
//                 INT Height,
//                 PBRUSH pbrush)
// {
//     return (BOOL)app_draw_ellipse(dc, rect( XLeft, YLeft, Width, Height), pbrush);
// }
//
// BOOL
// FASTCALL
// IntFillEllipse( PDC dc,
//                 INT XLeft,
//                 INT YLeft,
//                 INT Width,
//                 INT Height,
//                 PBRUSH pbrush)
// {
//     return (BOOL)app_fill_ellipse(dc, rect( XLeft, YLeft, Width, Height), pbrush);
// }
//
// BOOL
// FASTCALL
// IntFillRoundRect( PDC dc,
//                   INT Left,
//                   INT Top,
//                   INT Right,
//                   INT Bottom,
//                   INT Wellipse,
//                   INT Hellipse,
//                   PBRUSH pbrush)
// {
//     Rect r;
//     int rx, ry; /* Radius in x and y directions */
//
//     //           x    y          Width          Height
//     r = rect( Left, Top, abs(Right-Left), abs(Bottom-Top));
//     rx = Wellipse/2;
//     ry = Hellipse/2;
//
//     if (Wellipse > r.width)
//     {
//         if (Hellipse > r.height) // > W > H
//             app_fill_ellipse(dc, r, pbrush);
//         else // > W < H
//         {
//             app_fill_arc(dc, rect( r.x, r.y, r.width - 1, Hellipse),
//                          0, 180, pbrush,FALSE);
//             app_fill_arc(dc, rect(r.x, Bottom - Hellipse - 1, r.width - 1, Hellipse),
//                          180, 360, pbrush, FALSE);
//         }
//     }
//     else if(Hellipse > r.height) // < W > H
//     {
//         app_fill_arc(dc, rect(r.x, r.y, Wellipse, r.height - 1),
//                      90, 270, pbrush, FALSE);
//         app_fill_arc(dc, rect(Right - Wellipse - 1, r.y, Wellipse, r.height - 1),
//                      270, 90, pbrush,FALSE);
//     }
//     else // < W < H
//     {
//         app_fill_arc(dc, rect(r.x, r.y, rx+rx, ry+ry),
//                      90, 180, pbrush, FALSE);
//
//         app_fill_arc(dc, rect(r.x, r.y+r.height-ry-ry, rx+rx, ry+ry),
//                      180, 270, pbrush, FALSE);
//
//         app_fill_arc(dc, rect(r.x+r.width-rx-rx, r.y+r.height-ry-ry, rx+rx, ry+ry),
//                      270, 360, pbrush,FALSE);
//
//         app_fill_arc(dc, rect(r.x+r.width-rx-rx, r.y, rx+rx, ry+ry),
//                      0, 90, pbrush,FALSE);
//     }
//     if (Wellipse < r.width)
//     {
//         app_fill_rect(dc, rect(r.x+rx, r.y, r.width-rx-rx, ry+1), pbrush, FALSE);
//         app_fill_rect(dc, rect(r.x+rx, r.y+r.height-ry+1, r.width-rx-rx, ry-1), pbrush, FALSE);
//     }
//     if (Hellipse < r.height)
//     {
//         app_fill_rect(dc, rect(r.x, r.y+ry+1, r.width, r.height-ry-ry), pbrush, FALSE);
//     }
//
//     return TRUE;
// }
//
//
// BOOL
// FASTCALL
// IntDrawRoundRect( PDC dc,
//                   INT Left,
//                   INT Top,
//                   INT Right,
//                   INT Bottom,
//                   INT Wellipse,
//                   INT Hellipse,
//                   PBRUSH pbrushPen)
// {
//     Rect r;
//     int rx, ry; /* Radius in x and y directions */
//     int w = pbrushPen->lWidth;
//
//     r = rect( Left, Top, abs(Right-Left), abs(Bottom-Top));
//     rx = Wellipse/2;
//     ry = Hellipse/2;
//
//     if (Wellipse > r.width)
//     {
//         if (Hellipse > r.height) // > W > H
//             app_draw_ellipse(dc, r, pbrushPen);
//         else // > W < H
//         {
//             app_draw_arc(dc, rect( r.x, r.y, r.width - 1, Hellipse - 1),
//                          0, 180, pbrushPen, FALSE);
//             app_draw_arc(dc, rect(r.x, Bottom - Hellipse, r.width - 1, Hellipse - 1),
//                          180, 360, pbrushPen, FALSE);
//         }
//     }
//     else if(Hellipse > r.height) // < W > H
//     {
//         app_draw_arc(dc, rect(r.x, r.y, Wellipse - 1, r.height - 1),
//                      90, 270, pbrushPen, FALSE);
//         app_draw_arc(dc, rect(Right - Wellipse, r.y, Wellipse - 1, r.height - 1),
//                      270, 90, pbrushPen, FALSE);
//     }
//     else // < W < H
//     {
//         app_draw_arc(dc, rect(r.x, r.y, rx+rx, ry+ry),
//                      90, 180, pbrushPen, FALSE);
//
//         app_draw_arc(dc, rect(r.x,r.y+r.height-ry-ry,rx+rx,ry+ry),
//                      180, 270, pbrushPen, FALSE);
//
//         app_draw_arc(dc, rect(r.x+r.width-rx-rx, r.y+r.height-ry-ry, rx+rx, ry+ry),
//                      270, 360, pbrushPen, FALSE);
//
//         app_draw_arc(dc, rect(r.x+r.width-rx-rx,r.y,rx+rx,ry+ry),
//                      0, 90, pbrushPen, FALSE);
//     }
//     if ( Hellipse < r.height)
//     {
//         app_fill_rect(dc, rect(r.x, r.y+ry+1, w, r.height-ry-ry), pbrushPen, TRUE);
//
//
//         app_fill_rect(dc, rect(r.x+r.width-w, r.y+ry+1, w, r.height-ry-ry),
//                       pbrushPen, TRUE);
//     }
//     if ( Wellipse < r.width)
//     {
//         app_fill_rect(dc, rect(r.x+rx, r.y+r.height-w, r.width-rx-rx, w),
//                       pbrushPen, TRUE);
//
//         app_fill_rect(dc, rect(r.x+rx, r.y, r.width-rx-rx, w), pbrushPen, TRUE);
//     }
//     return TRUE;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
