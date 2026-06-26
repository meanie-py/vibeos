//! objects/linedda.c → Rust. The LineDDA digital differential analyzer (Bresenham line
//! walk), adapted from Wine. Pure client-side algorithm — invokes the caller's LINEDDAPROC
//! callback at each integer point along the line; no NtGdi* syscall, no GDI handle. Wave 0
//! leaf: proves the cohab staticlib link + i386 `_LineDDA@24` stdcall decoration + the
//! CALLBACK (stdcall) callback ABI.

use crate::types::*;

/// LINEDDAPROC — `VOID CALLBACK proc(int X, int Y, LPARAM lParam)`. CALLBACK is WINAPI
/// (stdcall on i386), so `extern "system"`.
pub type LINEDDAPROC = unsafe extern "system" fn(INT, INT, LPARAM);

/// LineDDA (GDI32.@) — walk the line (nXStart,nYStart)→(nXEnd,nYEnd) one integer point at
/// a time, calling `callback(x, y, lParam)` for each. i386 export: `_LineDDA@24`.
///
/// C: `BOOL WINAPI LineDDA(INT nXStart, INT nYStart, INT nXEnd, INT nYEnd,
///                         LINEDDAPROC callback, LPARAM lParam)`
#[no_mangle]
pub unsafe extern "system" fn LineDDA(
    mut nXStart: INT,
    mut nYStart: INT,
    nXEnd: INT,
    nYEnd: INT,
    callback: LINEDDAPROC,
    lParam: LPARAM,
) -> BOOL {
    let mut xadd: INT = 1;
    let mut yadd: INT = 1;
    let mut err: INT;
    let erradd: INT;
    let mut cnt: INT;
    let mut dx: INT = nXEnd - nXStart;
    let mut dy: INT = nYEnd - nYStart;

    if dx < 0 {
        dx = -dx;
        xadd = -1;
    }
    if dy < 0 {
        dy = -dy;
        yadd = -1;
    }

    if dx > dy {
        /* line is "more horizontal" */
        err = 2 * dy - dx;
        erradd = 2 * dy - 2 * dx;
        cnt = 0;
        while cnt < dx {
            callback(nXStart, nYStart, lParam);
            if err > 0 {
                nYStart += yadd;
                err += erradd;
            } else {
                err += 2 * dy;
            }
            nXStart += xadd;
            cnt += 1;
        }
    } else {
        /* line is "more vertical" */
        err = 2 * dx - dy;
        erradd = 2 * dx - 2 * dy;
        cnt = 0;
        while cnt < dy {
            callback(nXStart, nYStart, lParam);
            if err > 0 {
                nXStart += xadd;
                err += erradd;
            } else {
                err += 2 * dx;
            }
            nYStart += yadd;
            cnt += 1;
        }
    }

    TRUE
}
