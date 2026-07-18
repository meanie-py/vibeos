use win32k_sys::win32k::*;

extern "C" {
    // memset is a real CRT symbol (libcntpr); keep it.
    fn memset(s: *mut core::ffi::c_void, c: i32, n: usize) -> *mut core::ffi::c_void;
}
// abs/max macros + ROP4_FROM_INDEX macro (inteng.h) — inline.
#[inline(always)]
fn abs(x: i32) -> i32 { x.abs() }
#[inline(always)]
fn max(a: i32, b: i32) -> i32 { if a > b { a } else { b } }
#[inline(always)]
fn ROP4_FROM_INDEX(index: u32) -> u32 { index | (index << 8) }

// ---- file-scope preamble (translated) ----
pub const FILL_EDGE_ALLOC_TAG: u32 = 0x45465044;

#[repr(C)]
pub struct _tagFILL_EDGE {
    pub FromX: i32,
    pub FromY: i32,
    pub ToX: i32,
    pub ToY: i32,
    pub dx: i32,
    pub dy: i32,
    pub absdx: i32,
    pub absdy: i32,
    pub x: i32,
    pub y: i32,
    pub xmajor: i32,
    pub XIntercept: [i32; 2],
    pub Error: i32,
    pub ErrorMax: i32,
    pub XDirection: i32,
    pub YDirection: i32,
    pub pNext: *mut _tagFILL_EDGE,
}

pub type FILL_EDGE = _tagFILL_EDGE;

#[repr(C)]
pub struct _FILL_EDGE_LIST {
    pub Count: i32,
    pub Edges: *mut *mut FILL_EDGE,
}

pub type FILL_EDGE_LIST = _FILL_EDGE_LIST;

unsafe fn DEBUG_PRINT_ACTIVE_EDGELIST(list: *mut FILL_EDGE) {
    let mut pThis = list;
    if list.is_null() {
        return;
    }

    while !pThis.is_null() {
        pThis = (*pThis).pNext;
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn POLYGONFILL_DestroyEdgeList(list: *mut _FILL_EDGE_LIST) {
    if !list.is_null() {
        if !(*list).Edges.is_null() {
            for i in 0..(*list).Count {
                if !(*(*list).Edges.offset(i as isize)).is_null() {
                    EngFreeMem((*(*list).Edges.offset(i as isize)) as *mut core::ffi::c_void);
                }
            }
            EngFreeMem((*list).Edges as *mut core::ffi::c_void);
        }
        EngFreeMem(list as *mut core::ffi::c_void);
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn POLYGONFILL_DestroyEdgeList(list: *mut _FILL_EDGE_LIST) {
    if !list.is_null() {
        if !(*list).Edges.is_null() {
            for i in 0..(*list).Count {
                if !(*(*list).Edges.offset(i as isize)).is_null() {
                    EngFreeMem((*(*list).Edges.offset(i as isize)) as *mut core::ffi::c_void);
                }
            }
            EngFreeMem((*list).Edges as *mut core::ffi::c_void);
        }
        EngFreeMem(list as *mut core::ffi::c_void);
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn POLYGONFILL_MakeEdge(From: POINT, To: POINT) -> *mut FILL_EDGE {

    let rc = EngAllocMem(FL_ZERO_MEMORY, core::mem::size_of::<FILL_EDGE>() as u32, FILL_EDGE_ALLOC_TAG) as *mut FILL_EDGE;

    if rc.is_null() {
        return core::ptr::null_mut();
    }

    if To.y < From.y {
        (*rc).FromX = To.x;
        (*rc).FromY = To.y;
        (*rc).ToX = From.x;
        (*rc).ToY = From.y;
        (*rc).YDirection = -1;

        (*rc).Error = -1;
    } else {
        (*rc).FromX = From.x;
        (*rc).FromY = From.y;
        (*rc).ToX = To.x;
        (*rc).ToY = To.y;
        (*rc).YDirection = 1;

        (*rc).Error = 0;
    }

    (*rc).x = (*rc).FromX;
    (*rc).y = (*rc).FromY;
    (*rc).dx = (*rc).ToX - (*rc).FromX;
    (*rc).dy = (*rc).ToY - (*rc).FromY;
    (*rc).absdx = abs((*rc).dx);
    (*rc).absdy = abs((*rc).dy);

    (*rc).xmajor = if (*rc).absdx > (*rc).absdy { 1 } else { 0 };

    (*rc).ErrorMax = max((*rc).absdx, (*rc).absdy);

    (*rc).Error += (*rc).ErrorMax / 2;

    (*rc).XDirection = if (*rc).dx < 0 { -1 } else { 1 };

    (*rc).pNext = core::ptr::null_mut();

    rc
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn POLYGONFILL_MakeEdge(From: POINT, To: POINT) -> *mut FILL_EDGE {

    let rc = EngAllocMem(FL_ZERO_MEMORY, core::mem::size_of::<FILL_EDGE>() as u32, FILL_EDGE_ALLOC_TAG) as *mut FILL_EDGE;

    if rc.is_null() {
        return core::ptr::null_mut();
    }

    if To.y < From.y {
        (*rc).FromX = To.x;
        (*rc).FromY = To.y;
        (*rc).ToX = From.x;
        (*rc).ToY = From.y;
        (*rc).YDirection = -1;

        (*rc).Error = -1;
    } else {
        (*rc).FromX = From.x;
        (*rc).FromY = From.y;
        (*rc).ToX = To.x;
        (*rc).ToY = To.y;
        (*rc).YDirection = 1;

        (*rc).Error = 0;
    }

    (*rc).x = (*rc).FromX;
    (*rc).y = (*rc).FromY;
    (*rc).dx = (*rc).ToX - (*rc).FromX;
    (*rc).dy = (*rc).ToY - (*rc).FromY;
    (*rc).absdx = abs((*rc).dx);
    (*rc).absdy = abs((*rc).dy);

    (*rc).xmajor = if (*rc).absdx > (*rc).absdy { 1 } else { 0 };

    (*rc).ErrorMax = max((*rc).absdx, (*rc).absdy);

    (*rc).Error += (*rc).ErrorMax / 2;

    (*rc).XDirection = if (*rc).dx < 0 { -1 } else { 1 };

    (*rc).pNext = core::ptr::null_mut();

    rc
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn FILL_EDGE_Compare(Edge1: *mut FILL_EDGE, Edge2: *mut FILL_EDGE) -> INT {
    let e1 = (*Edge1).XIntercept[0] + (*Edge1).XIntercept[1];
    let e2 = (*Edge2).XIntercept[0] + (*Edge2).XIntercept[1];
    
    e1 - e2
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn FILL_EDGE_Compare(Edge1: *mut FILL_EDGE, Edge2: *mut FILL_EDGE) -> INT {
    let e1 = (*Edge1).XIntercept[0] + (*Edge1).XIntercept[1];
    let e2 = (*Edge2).XIntercept[0] + (*Edge2).XIntercept[1];
    
    e1 - e2
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn POLYGONFILL_ActiveListInsert(activehead: *mut *mut FILL_EDGE, NewEdge: *mut FILL_EDGE) {
    let mut pPrev: *mut FILL_EDGE;
    let mut pThis: *mut FILL_EDGE;

    if (*activehead).is_null() {
        (*NewEdge).pNext = core::ptr::null_mut();
        *activehead = NewEdge;
        return;
    }

    if FILL_EDGE_Compare(NewEdge, *activehead) <= 0 {
        (*NewEdge).pNext = *activehead;
        *activehead = NewEdge;
        return;
    }

    pThis = *activehead;
    pPrev = core::ptr::null_mut();
    while !pThis.is_null() && FILL_EDGE_Compare(pThis, NewEdge) < 0 {
        pPrev = pThis;
        pThis = (*pThis).pNext;
    }

    (*NewEdge).pNext = (*pPrev).pNext;
    (*pPrev).pNext = NewEdge;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn POLYGONFILL_ActiveListInsert(activehead: *mut *mut FILL_EDGE, NewEdge: *mut FILL_EDGE) {
    let mut pPrev: *mut FILL_EDGE;
    let mut pThis: *mut FILL_EDGE;

    if (*activehead).is_null() {
        (*NewEdge).pNext = core::ptr::null_mut();
        *activehead = NewEdge;
        return;
    }

    if FILL_EDGE_Compare(NewEdge, *activehead) <= 0 {
        (*NewEdge).pNext = *activehead;
        *activehead = NewEdge;
        return;
    }

    pThis = *activehead;
    pPrev = core::ptr::null_mut();
    while !pThis.is_null() && FILL_EDGE_Compare(pThis, NewEdge) < 0 {
        pPrev = pThis;
        pThis = (*pThis).pNext;
    }

    (*NewEdge).pNext = (*pPrev).pNext;
    (*pPrev).pNext = NewEdge;
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn POLYGONFILL_MakeEdgeList(Points: PPOINT, Count: i32) -> *mut FILL_EDGE_LIST {

    let mut CurPt: i32 = 0;
    let mut list: *mut FILL_EDGE_LIST = core::ptr::null_mut();
    let mut e: *mut FILL_EDGE = core::ptr::null_mut();

    if Points.is_null() || 2 > Count {
        return core::ptr::null_mut();
    }

    'fail: loop {
        list = EngAllocMem(FL_ZERO_MEMORY, core::mem::size_of::<FILL_EDGE_LIST>() as u32, FILL_EDGE_ALLOC_TAG) as *mut FILL_EDGE_LIST;
        if list.is_null() {
            break 'fail;
        }
        (*list).Count = 0;
        (*list).Edges = EngAllocMem(FL_ZERO_MEMORY, (Count as usize * core::mem::size_of::<*mut FILL_EDGE>()) as u32, FILL_EDGE_ALLOC_TAG) as *mut *mut FILL_EDGE;
        if (*list).Edges.is_null() {
            break 'fail;
        }

        memset((*list).Edges as *mut core::ffi::c_void, 0, Count as usize * core::mem::size_of::<*mut FILL_EDGE>());

        CurPt = 1;
        while CurPt < Count {
            e = POLYGONFILL_MakeEdge(*Points.offset((CurPt - 1) as isize), *Points.offset(CurPt as isize));
            if e.is_null() {
                break 'fail;
            }

            if (*e).absdy == 0 {
                EngFreeMem(e as *mut core::ffi::c_void);
            } else {
                *(*list).Edges.offset((*list).Count as isize) = e;
                (*list).Count += 1;
            }
            CurPt += 1;
        }
        e = POLYGONFILL_MakeEdge(*Points.offset((CurPt - 1) as isize), *Points.offset(0));
        if e.is_null() {
            break 'fail;
        }

        if (*e).absdy == 0 {
            EngFreeMem(e as *mut core::ffi::c_void);
        } else {
            *(*list).Edges.offset((*list).Count as isize) = e;
            (*list).Count += 1;
        }
        return list;
    }

    POLYGONFILL_DestroyEdgeList(list);
    return core::ptr::null_mut();
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn POLYGONFILL_MakeEdgeList(Points: PPOINT, Count: i32) -> *mut FILL_EDGE_LIST {

    let mut CurPt: i32 = 0;
    let mut list: *mut FILL_EDGE_LIST = core::ptr::null_mut();
    let mut e: *mut FILL_EDGE = core::ptr::null_mut();

    if Points.is_null() || 2 > Count {
        return core::ptr::null_mut();
    }

    'fail: loop {
        list = EngAllocMem(FL_ZERO_MEMORY, core::mem::size_of::<FILL_EDGE_LIST>() as u32, FILL_EDGE_ALLOC_TAG) as *mut FILL_EDGE_LIST;
        if list.is_null() {
            break 'fail;
        }
        (*list).Count = 0;
        (*list).Edges = EngAllocMem(FL_ZERO_MEMORY, (Count as usize * core::mem::size_of::<*mut FILL_EDGE>()) as u32, FILL_EDGE_ALLOC_TAG) as *mut *mut FILL_EDGE;
        if (*list).Edges.is_null() {
            break 'fail;
        }

        memset((*list).Edges as *mut core::ffi::c_void, 0, Count as usize * core::mem::size_of::<*mut FILL_EDGE>());

        CurPt = 1;
        while CurPt < Count {
            e = POLYGONFILL_MakeEdge(*Points.offset((CurPt - 1) as isize), *Points.offset(CurPt as isize));
            if e.is_null() {
                break 'fail;
            }

            if (*e).absdy == 0 {
                EngFreeMem(e as *mut core::ffi::c_void);
            } else {
                *(*list).Edges.offset((*list).Count as isize) = e;
                (*list).Count += 1;
            }
            CurPt += 1;
        }
        e = POLYGONFILL_MakeEdge(*Points.offset((CurPt - 1) as isize), *Points.offset(0));
        if e.is_null() {
            break 'fail;
        }

        if (*e).absdy == 0 {
            EngFreeMem(e as *mut core::ffi::c_void);
        } else {
            *(*list).Edges.offset((*list).Count as isize) = e;
            (*list).Count += 1;
        }
        return list;
    }

    POLYGONFILL_DestroyEdgeList(list);
    return core::ptr::null_mut();
}

fastcall_fn! {
unsafe fn POLYGONFILL_UpdateScanline(pEdge: *mut FILL_EDGE, Scanline: i32) {
    if (*pEdge).dy == 0 {
        return;
    }

    if (*pEdge).xmajor != 0 {
        let mut steps: i32;

        steps = ((*pEdge).ErrorMax - (*pEdge).Error - 1) / (*pEdge).absdy;
        if steps != 0 {
            let x1: i32 = (*pEdge).x;
            (*pEdge).x += steps * (*pEdge).XDirection;
            (*pEdge).Error += steps * (*pEdge).absdy;
            (*pEdge).XIntercept[0] = core::cmp::min(x1, (*pEdge).x);
            (*pEdge).XIntercept[1] = core::cmp::max(x1, (*pEdge).x);
        } else {
            (*pEdge).XIntercept[0] = (*pEdge).x;
            (*pEdge).XIntercept[1] = (*pEdge).x;
        }

        (*pEdge).x += (*pEdge).XDirection;
        (*pEdge).Error += (*pEdge).absdy;

        (*pEdge).Error -= (*pEdge).absdx;
        (*pEdge).y += 1;
    } else {
        (*pEdge).XIntercept[0] = (*pEdge).x;
        (*pEdge).XIntercept[1] = (*pEdge).x;

        (*pEdge).Error += (*pEdge).absdx;
        (*pEdge).y += 1;

        if (*pEdge).Error >= (*pEdge).ErrorMax {
            (*pEdge).Error -= (*pEdge).ErrorMax;
            (*pEdge).x += (*pEdge).XDirection;
        }
    }
}
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn POLYGONFILL_BuildActiveList(
    Scanline: i32,
    list: *mut FILL_EDGE_LIST,
    ActiveHead: *mut *mut FILL_EDGE,
) {
    let mut i: i32;

    *ActiveHead = core::ptr::null_mut();
    i = 0;
    while i < (*list).Count {
        let pEdge: *mut FILL_EDGE = *((*list).Edges).offset(i as isize);
        if (*pEdge).FromY <= Scanline && (*pEdge).ToY > Scanline {
            POLYGONFILL_UpdateScanline(pEdge, Scanline);
            POLYGONFILL_ActiveListInsert(ActiveHead, pEdge);
        }
        i += 1;
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn POLYGONFILL_BuildActiveList(
    Scanline: i32,
    list: *mut FILL_EDGE_LIST,
    ActiveHead: *mut *mut FILL_EDGE,
) {
    let mut i: i32;

    *ActiveHead = core::ptr::null_mut();
    i = 0;
    while i < (*list).Count {
        let pEdge: *mut FILL_EDGE = *((*list).Edges).offset(i as isize);
        if (*pEdge).FromY <= Scanline && (*pEdge).ToY > Scanline {
            POLYGONFILL_UpdateScanline(pEdge, Scanline);
            POLYGONFILL_ActiveListInsert(ActiveHead, pEdge);
        }
        i += 1;
    }
}

unsafe extern "system" fn POLYGONFILL_FillScanLineAlternate(
    dc: PDC,
    ScanLine: i32,
    ActiveHead: *mut FILL_EDGE,
    psurf: *mut SURFACE,
    BrushObj: *mut BRUSHOBJ,
    RopMode: MIX,
) {
    if ActiveHead.is_null() {
        return;
    }

    let mut pLeft = ActiveHead;
    let mut pRight = (*pLeft).pNext;

    while !pRight.is_null() {
        let x1 = (*pLeft).XIntercept[0];
        let x2 = (*pRight).XIntercept[1];
        if x2 > x1 {
            let mut BoundRect = RECTL {
                top: ScanLine,
                bottom: ScanLine + 1,
                left: x1,
                right: x2,
            };

            IntEngLineTo(
                core::ptr::addr_of_mut!((*psurf).SurfObj),
                core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
                BrushObj,
                x1,
                ScanLine,
                x2,
                ScanLine,
                &mut BoundRect,
                RopMode,
            );
        }
        pLeft = (*pRight).pNext;
        pRight = if !pLeft.is_null() {
            (*pLeft).pNext
        } else {
            core::ptr::null_mut()
        };
    }
}

unsafe extern "system" fn POLYGONFILL_FillScanLineWinding(
    dc: PDC,
    ScanLine: i32,
    ActiveHead: *mut FILL_EDGE,
    psurf: *mut SURFACE,
    BrushObj: *mut BRUSHOBJ,
    RopMode: MIX,
) {
    let mut pLeft: *mut FILL_EDGE;
    let mut pRight: *mut FILL_EDGE;
    let mut x1: i32;
    let mut x2: i32;
    let mut winding: i32 = 0;
    let mut BoundRect: RECTL = core::mem::zeroed();

    if ActiveHead.is_null() {
        return;
    }

    BoundRect.top = ScanLine;
    BoundRect.bottom = ScanLine + 1;

    pLeft = ActiveHead;
    winding = (*pLeft).YDirection;
    pRight = (*pLeft).pNext;

    x1 = (*pLeft).XIntercept[0];
    x2 = (*pRight).XIntercept[1];

    pLeft = pRight;
    pRight = (*pLeft).pNext;
    winding += (*pLeft).YDirection;

    while !pRight.is_null() {
        let newx1 = (*pLeft).XIntercept[0];
        let newx2 = (*pRight).XIntercept[1];
        if winding != 0 {
            if (newx1 >= x1 && newx1 <= x2) ||
               (newx2 >= x1 && newx2 <= x2) ||
               (x1 >= newx1 && x1 <= newx2) ||
               (x2 >= newx2 && x2 <= newx2)
            {
                x1 = if x1 < newx1 { x1 } else { newx1 };
                x2 = if x2 > newx2 { x2 } else { newx2 };
            } else {
                BoundRect.left = x1;
                BoundRect.right = x2;

                IntEngLineTo(
                    core::ptr::addr_of_mut!((*psurf).SurfObj),
                    core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
                    BrushObj,
                    x1,
                    ScanLine,
                    x2,
                    ScanLine,
                    &mut BoundRect,
                    RopMode,
                );

                x1 = newx1;
                x2 = newx2;
            }
        }

        pLeft = pRight;
        pRight = (*pLeft).pNext;
        winding += (*pLeft).YDirection;
    }

    BoundRect.left = x1;
    BoundRect.right = x2;

    IntEngLineTo(
        core::ptr::addr_of_mut!((*psurf).SurfObj),
        core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
        BrushObj,
        x1,
        ScanLine,
        x2,
        ScanLine,
        &mut BoundRect,
        RopMode,
    );
}

#[no_mangle]
pub unsafe extern "system" fn FillPolygon(
    dc: PDC,
    psurf: *mut SURFACE,
    BrushObj: *mut BRUSHOBJ,
    RopMode: MIX,
    Points: PPOINT,
    Count: i32,
    BoundRect: RECTL,
) -> i32 {
    let mut list: *mut FILL_EDGE_LIST = core::ptr::null_mut();
    let mut ActiveHead: *mut FILL_EDGE = core::ptr::null_mut();
    let mut ScanLine: i32;
    let pdcattr: PDC_ATTR = (*dc).pdcattr;
    let FillScanLine: unsafe extern "system" fn(
        PDC,
        i32,
        *mut FILL_EDGE,
        *mut SURFACE,
        *mut BRUSHOBJ,
        MIX,
    );

    list = POLYGONFILL_MakeEdgeList(Points, Count);
    if list.is_null() {
        return 0;
    }

    if WINDING == (*pdcattr).jFillMode as u32 {
        FillScanLine = POLYGONFILL_FillScanLineWinding;
    } else {
        FillScanLine = POLYGONFILL_FillScanLineAlternate;
    }

    ScanLine = BoundRect.top;
    while ScanLine < BoundRect.bottom {
        POLYGONFILL_BuildActiveList(ScanLine, list, core::ptr::addr_of_mut!(ActiveHead));
        FillScanLine(dc, ScanLine, ActiveHead, psurf, BrushObj, RopMode);
        ScanLine += 1;
    }

    POLYGONFILL_DestroyEdgeList(list);

    return 1;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntFillPolygon(
    dc: PDC,
    psurf: *mut SURFACE,
    BrushObj: *mut BRUSHOBJ,
    Points: PPOINT,
    Count: i32,
    DestRect: RECTL,
    BrushOrigin: *mut POINTL,
) -> u8 {

    let mut list: *mut FILL_EDGE_LIST = core::ptr::null_mut();
    let mut ActiveHead: *mut FILL_EDGE = core::ptr::null_mut();
    let mut pLeft: *mut FILL_EDGE;
    let mut pRight: *mut FILL_EDGE;
    let mut ScanLine: i32;

    list = POLYGONFILL_MakeEdgeList(Points, Count);
    if list.is_null() {
        return 0u8;
    }

    ScanLine = DestRect.top;
    while ScanLine < DestRect.bottom {
        POLYGONFILL_BuildActiveList(ScanLine, list, &mut ActiveHead);

        if ActiveHead.is_null() {
            POLYGONFILL_DestroyEdgeList(list);
            return 0u8;
        }

        pLeft = ActiveHead;
        pRight = (*pLeft).pNext;

        while !pRight.is_null() {
            let x1 = (*pLeft).XIntercept[0];
            let x2 = (*pRight).XIntercept[1];

            if x2 > x1 {
                let mut LineRect: RECTL = core::mem::zeroed();
                LineRect.top = ScanLine;
                LineRect.bottom = ScanLine + 1;
                LineRect.left = x1;
                LineRect.right = x2;

                IntEngBitBlt(
                    &mut (*psurf).SurfObj,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of!((*dc).co) as *mut CLIPOBJ,
                    core::ptr::null_mut(),
                    &mut LineRect,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    BrushObj,
                    BrushOrigin,
                    ROP4_FROM_INDEX(R3_OPINDEX_PATCOPY),
                );
            }

            pLeft = (*pRight).pNext;
            pRight = if !pLeft.is_null() { (*pLeft).pNext } else { core::ptr::null_mut() };
        }

        ScanLine += 1;
    }

    POLYGONFILL_DestroyEdgeList(list);

    1u8
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntFillPolygon(
    dc: PDC,
    psurf: *mut SURFACE,
    BrushObj: *mut BRUSHOBJ,
    Points: PPOINT,
    Count: i32,
    DestRect: RECTL,
    BrushOrigin: *mut POINTL,
) -> u8 {

    let mut list: *mut FILL_EDGE_LIST = core::ptr::null_mut();
    let mut ActiveHead: *mut FILL_EDGE = core::ptr::null_mut();
    let mut pLeft: *mut FILL_EDGE;
    let mut pRight: *mut FILL_EDGE;
    let mut ScanLine: i32;

    list = POLYGONFILL_MakeEdgeList(Points, Count);
    if list.is_null() {
        return 0u8;
    }

    ScanLine = DestRect.top;
    while ScanLine < DestRect.bottom {
        POLYGONFILL_BuildActiveList(ScanLine, list, &mut ActiveHead);

        if ActiveHead.is_null() {
            POLYGONFILL_DestroyEdgeList(list);
            return 0u8;
        }

        pLeft = ActiveHead;
        pRight = (*pLeft).pNext;

        while !pRight.is_null() {
            let x1 = (*pLeft).XIntercept[0];
            let x2 = (*pRight).XIntercept[1];

            if x2 > x1 {
                let mut LineRect: RECTL = core::mem::zeroed();
                LineRect.top = ScanLine;
                LineRect.bottom = ScanLine + 1;
                LineRect.left = x1;
                LineRect.right = x2;

                IntEngBitBlt(
                    &mut (*psurf).SurfObj,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of!((*dc).co) as *mut CLIPOBJ,
                    core::ptr::null_mut(),
                    &mut LineRect,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    BrushObj,
                    BrushOrigin,
                    ROP4_FROM_INDEX(R3_OPINDEX_PATCOPY),
                );
            }

            pLeft = (*pRight).pNext;
            pRight = if !pLeft.is_null() { (*pLeft).pNext } else { core::ptr::null_mut() };
        }

        ScanLine += 1;
    }

    POLYGONFILL_DestroyEdgeList(list);

    1u8
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS Win32k subsystem
//  * PURPOSE:           Various Polygon Filling routines for Polygon()
//  * FILE:              win32ss/gdi/ntgdi/polyfill.c
//  * PROGRAMER:         Mark Tempel
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #define FILL_EDGE_ALLOC_TAG 0x45465044
//
// /*
// ** This struct is used for book keeping during polygon filling routines.
// */
// typedef struct _tagFILL_EDGE
// {
//     /* Basic line information */
//     int FromX;
//     int FromY;
//     int ToX;
//     int ToY;
//     int dx;
//     int dy;
//     int absdx, absdy;
//     int x, y;
//     int xmajor;
//
//     /* Active Edge List information */
//     int XIntercept[2];
//     int Error;
//     int ErrorMax;
//     int XDirection, YDirection;
//
//     /* The next edge in the active Edge List */
//     struct _tagFILL_EDGE * pNext;
// } FILL_EDGE;
//
// typedef struct _FILL_EDGE_LIST
// {
//     int Count;
//     FILL_EDGE** Edges;
// } FILL_EDGE_LIST;
//
// #if 0
// static
// void
// DEBUG_PRINT_ACTIVE_EDGELIST ( FILL_EDGE* list )
// {
//     FILL_EDGE* pThis = list;
//     if (0 == list)
//     {
//         DPRINT1("List is NULL\n");
//         return;
//     }
//
//     while(0 != pThis)
//     {
//         //DPRINT1("EDGE: (%d, %d) to (%d, %d)\n", pThis->FromX, pThis->FromY, pThis->ToX, pThis->ToY);
//         DPRINT1("EDGE: [%d,%d]\n", pThis->XIntercept[0], pThis->XIntercept[1] );
//         pThis = pThis->pNext;
//     }
// }
// #else
// #define DEBUG_PRINT_ACTIVE_EDGELIST(x)
// #endif
//
// /*
// **  Hide memory clean up.
// */
// static
// void
// FASTCALL
// POLYGONFILL_DestroyEdgeList(FILL_EDGE_LIST* list)
// {
//     int i;
//
//     if (list)
//     {
//         if (list->Edges)
//         {
//             for (i = 0; i < list->Count; i++)
//             {
//                 _PRAGMA_WARNING_SUPPRESS(__WARNING_USING_UNINIT_VAR)
//                 if (list->Edges[i])
//                     EngFreeMem(list->Edges[i]);
//             }
//
//             EngFreeMem(list->Edges);
//         }
//
//         EngFreeMem(list);
//     }
// }
//
// /*
// ** This makes and initializes an Edge struct for a line between two points.
// */
// static
// FILL_EDGE*
// FASTCALL
// POLYGONFILL_MakeEdge(POINT From, POINT To)
// {
//     FILL_EDGE* rc = (FILL_EDGE*)EngAllocMem(FL_ZERO_MEMORY, sizeof(FILL_EDGE), FILL_EDGE_ALLOC_TAG);
//
//     if (0 == rc)
//         return NULL;
//
//     //DPRINT1("Making Edge: (%d, %d) to (%d, %d)\n", From.x, From.y, To.x, To.y);
//     // Now fill the struct.
//     if ( To.y < From.y )
//     {
//         rc->FromX = To.x;
//         rc->FromY = To.y;
//         rc->ToX = From.x;
//         rc->ToY = From.y;
//         rc->YDirection = -1;
//
//         // Lines that go up get walked backwards, so need to be offset
//         // by -1 in order to make the walk identically on a pixel-level
//         rc->Error = -1;
//     }
//     else
//     {
//         rc->FromX = From.x;
//         rc->FromY = From.y;
//         rc->ToX = To.x;
//         rc->ToY = To.y;
//         rc->YDirection = 1;
//
//         rc->Error = 0;
//     }
//
//     rc->x = rc->FromX;
//     rc->y = rc->FromY;
//     rc->dx   = rc->ToX - rc->FromX;
//     rc->dy   = rc->ToY - rc->FromY;
//     rc->absdx = abs(rc->dx);
//     rc->absdy = abs(rc->dy);
//
//     rc->xmajor = rc->absdx > rc->absdy;
//
//     rc->ErrorMax = max(rc->absdx,rc->absdy);
//
//     rc->Error += rc->ErrorMax / 2;
//
//     rc->XDirection = (rc->dx < 0)?(-1):(1);
//
//     rc->pNext = 0;
//
//     //DPRINT("MakeEdge (%i,%i)->(%i,%i) d=(%i,%i) dir=(%i,%i) err=%i max=%i\n",
//     //  From.x, From.y, To.x, To.y, rc->dx, rc->dy, rc->XDirection, rc->YDirection, rc->Error, rc->ErrorMax );
//
//     return rc;
// }
// /*
// ** My Edge comparison routine.
// ** This is for scan converting polygon fill.
// ** First sort by MinY, then Minx, then slope.
// **
// ** This comparison will help us determine which
// ** lines will become active first when scanning from
// ** top (min y) to bottom (max y).
// **
// ** Return Value Meaning
// ** Negative integer element1 < element2
// ** Zero element1 = element2
// ** Positive integer element1 > element2
// */
// static
// INT
// FASTCALL
// FILL_EDGE_Compare(FILL_EDGE* Edge1, FILL_EDGE* Edge2)
// {
//     int e1 = Edge1->XIntercept[0] + Edge1->XIntercept[1];
//     int e2 = Edge2->XIntercept[0] + Edge2->XIntercept[1];
//
//     return e1 - e2;
// }
//
//
// /*
// ** Insert an edge into a list keeping the list in order.
// */
// static
// void
// FASTCALL
// POLYGONFILL_ActiveListInsert(FILL_EDGE** activehead, FILL_EDGE* NewEdge )
// {
//     FILL_EDGE *pPrev, *pThis;
//     //DPRINT1("In POLYGONFILL_ActiveListInsert()\n");
//     ASSERT(activehead && NewEdge);
//
//     if (!*activehead)
//     {
//         NewEdge->pNext = NULL;
//         *activehead = NewEdge;
//         return;
//     }
//     /*
//     ** First lets check to see if we have a new smallest value.
//     */
//     if (FILL_EDGE_Compare(NewEdge, *activehead) <= 0)
//     {
//         NewEdge->pNext = *activehead;
//         *activehead = NewEdge;
//         return;
//     }
//
//     /*
//     ** Ok, now scan to the next spot to put this item.
//     */
//     pThis = *activehead;
//     pPrev = NULL;
//     while ( pThis && FILL_EDGE_Compare(pThis, NewEdge) < 0 )
//     {
//         pPrev = pThis;
//         pThis = pThis->pNext;
//     }
//
//     ASSERT(pPrev);
//     NewEdge->pNext = pPrev->pNext;
//     pPrev->pNext = NewEdge;
//     //DEBUG_PRINT_ACTIVE_EDGELIST(*activehead);
// }
//
// /*
// ** Create a list of edges for a list of points.
// */
// static
// FILL_EDGE_LIST*
// FASTCALL
// POLYGONFILL_MakeEdgeList(PPOINT Points, int Count)
// {
//     int CurPt = 0;
//     FILL_EDGE_LIST* list = 0;
//     FILL_EDGE* e = 0;
//
//     if (0 == Points || 2 > Count)
//         return 0;
//
//     list = (FILL_EDGE_LIST*)EngAllocMem(FL_ZERO_MEMORY, sizeof(FILL_EDGE_LIST), FILL_EDGE_ALLOC_TAG);
//     if ( 0 == list )
//         goto fail;
//     list->Count = 0;
//     list->Edges = (FILL_EDGE**)EngAllocMem(FL_ZERO_MEMORY, Count*sizeof(FILL_EDGE*), FILL_EDGE_ALLOC_TAG);
//     if ( !list->Edges )
//         goto fail;
//
//     memset(list->Edges, 0, Count * sizeof(FILL_EDGE*));
//
//     for (CurPt = 1; CurPt < Count; ++CurPt)
//     {
//         e = POLYGONFILL_MakeEdge ( Points[CurPt-1], Points[CurPt] );
//         if (!e)
//             goto fail;
//
//         // If a straight horizontal line - who cares?
//         if (!e->absdy)
//             EngFreeMem(e);
//         else
//             list->Edges[list->Count++] = e;
//     }
//     e = POLYGONFILL_MakeEdge ( Points[CurPt-1], Points[0] );
//     if ( !e )
//         goto fail;
//
//     if (!e->absdy)
//         EngFreeMem(e);
//     else
//         list->Edges[list->Count++] = e;
//     return list;
//
// fail:
//
//     DPRINT1("Out Of MEMORY!!\n");
//     POLYGONFILL_DestroyEdgeList ( list );
//     return 0;
// }
//
//
// /*
// ** This slow routine uses the data stored in the edge list to
// ** calculate the x intercepts for each line in the edge list
// ** for scanline Scanline.
// **TODO: Get rid of this floating point arithmetic
// */
// static
// void
// FASTCALL
// POLYGONFILL_UpdateScanline(FILL_EDGE* pEdge, int Scanline)
// {
//     if (0 == pEdge->dy)
//         return;
//
//     ASSERT(pEdge->FromY <= Scanline && pEdge->ToY > Scanline);
//
//     if (pEdge->xmajor)
//     {
//         int steps;
//
//         ASSERT(pEdge->y == Scanline);
//
//         // Now shoot to end of scanline collision
//         steps = (pEdge->ErrorMax-pEdge->Error-1)/pEdge->absdy;
//         if (steps)
//         {
//             // Record first collision with scanline
//             int x1 = pEdge->x;
//             pEdge->x += steps * pEdge->XDirection;
//             pEdge->Error += steps * pEdge->absdy;
//             ASSERT ( pEdge->Error < pEdge->ErrorMax );
//             pEdge->XIntercept[0] = min(x1,pEdge->x);
//             pEdge->XIntercept[1] = max(x1,pEdge->x);
//         }
//         else
//         {
//             pEdge->XIntercept[0] = pEdge->x;
//             pEdge->XIntercept[1] = pEdge->x;
//         }
//
//         // We should require exactly 1 step to step onto next scanline...
//         ASSERT((pEdge->ErrorMax-pEdge->Error-1) / pEdge->absdy == 0);
//         pEdge->x += pEdge->XDirection;
//         pEdge->Error += pEdge->absdy;
//         ASSERT(pEdge->Error >= pEdge->ErrorMax);
//
//         // Now step onto next scanline...
//         pEdge->Error -= pEdge->absdx;
//         pEdge->y++;
//     }
//     else // Then this is a y-major line
//     {
//         pEdge->XIntercept[0] = pEdge->x;
//         pEdge->XIntercept[1] = pEdge->x;
//
//         pEdge->Error += pEdge->absdx;
//         pEdge->y++;
//
//         if (pEdge->Error >= pEdge->ErrorMax)
//         {
//             pEdge->Error -= pEdge->ErrorMax;
//             pEdge->x += pEdge->XDirection;
//             ASSERT ( pEdge->Error < pEdge->ErrorMax );
//         }
//     }
//
//     //DPRINT("Line (%d, %d) to (%d, %d) intersects scanline %d at (%d,%d)\n",
//     //        pEdge->FromX, pEdge->FromY, pEdge->ToX, pEdge->ToY, Scanline, pEdge->XIntercept[0], pEdge->XIntercept[1] );
// }
//
// /*
//  * This method updates the Active edge collection for the scanline Scanline.
//  */
// static
// void
// APIENTRY
// POLYGONFILL_BuildActiveList(
//     int Scanline,
//     FILL_EDGE_LIST* list,
//     FILL_EDGE** ActiveHead)
// {
//     int i;
//
//     ASSERT(list && ActiveHead);
//     *ActiveHead = 0;
//     for (i = 0; i < list->Count; i++)
//     {
//         FILL_EDGE* pEdge = list->Edges[i];
//         ASSERT(pEdge);
//         if (pEdge->FromY <= Scanline && pEdge->ToY > Scanline)
//         {
//             POLYGONFILL_UpdateScanline(pEdge, Scanline);
//             POLYGONFILL_ActiveListInsert(ActiveHead, pEdge);
//         }
//     }
// }
//
// /*
// ** This method fills the portion of the polygon that intersects with the scanline
// ** Scanline.
// */
// static
// void
// APIENTRY
// POLYGONFILL_FillScanLineAlternate(
//     PDC dc,
//     int ScanLine,
//     FILL_EDGE* ActiveHead,
//     SURFACE *psurf,
//     BRUSHOBJ *BrushObj,
//     MIX RopMode )
// {
//     FILL_EDGE *pLeft, *pRight;
//
//     if (!ActiveHead)
//         return;
//
//     pLeft = ActiveHead;
//     pRight = pLeft->pNext;
//     ASSERT(pRight);
//
//     while (NULL != pRight)
//     {
//         int x1 = pLeft->XIntercept[0];
//         int x2 = pRight->XIntercept[1];
//         if (x2 > x1)
//         {
//             RECTL BoundRect;
//             BoundRect.top = ScanLine;
//             BoundRect.bottom = ScanLine + 1;
//             BoundRect.left = x1;
//             BoundRect.right = x2;
//
//             //DPRINT("Fill Line (%d, %d) to (%d, %d)\n",x1, ScanLine, x2, ScanLine);
//             IntEngLineTo(&psurf->SurfObj,
//                          (CLIPOBJ *)&dc->co,
//                          BrushObj,
//                          x1,
//                          ScanLine,
//                          x2,
//                          ScanLine,
//                          &BoundRect,  // Bounding rectangle
//                          RopMode);    // MIX
//         }
//         pLeft = pRight->pNext;
//         pRight = pLeft ? pLeft->pNext : NULL;
//     }
// }
//
// static
// void
// APIENTRY
// POLYGONFILL_FillScanLineWinding(
//     PDC dc,
//     int ScanLine,
//     FILL_EDGE* ActiveHead,
//     SURFACE *psurf,
//     BRUSHOBJ *BrushObj,
//     MIX RopMode )
// {
//     FILL_EDGE *pLeft, *pRight;
//     int x1, x2, winding = 0;
//     RECTL BoundRect;
//
//     if (!ActiveHead)
//         return;
//
//     BoundRect.top = ScanLine;
//     BoundRect.bottom = ScanLine + 1;
//
//     pLeft = ActiveHead;
//     winding = pLeft->YDirection;
//     pRight = pLeft->pNext;
//     ASSERT(pRight);
//
//     // Setup first line...
//     x1 = pLeft->XIntercept[0];
//     x2 = pRight->XIntercept[1];
//
//     pLeft = pRight;
//     pRight = pLeft->pNext;
//     winding += pLeft->YDirection;
//
//     while ( NULL != pRight )
//     {
//         int newx1 = pLeft->XIntercept[0];
//         int newx2 = pRight->XIntercept[1];
//         if ( winding )
//         {
//             // Check and see if this new line touches the previous...
//             if ((newx1 >= x1 && newx1 <= x2) ||
//                 (newx2 >= x1 && newx2 <= x2) ||
//                 (x1 >= newx1 && x1 <= newx2) ||
//                 (x2 >= newx2 && x2 <= newx2))
//             {
//                 // Yup, just tack it on to our existing line
//                 x1 = min(x1,newx1);
//                 x2 = max(x2,newx2);
//             }
//             else
//             {
//                 // Nope - render the old line..
//                 BoundRect.left = x1;
//                 BoundRect.right = x2;
//
//                 //DPRINT("Fill Line (%d, %d) to (%d, %d)\n",x1, ScanLine, x2, ScanLine);
//                 IntEngLineTo(&psurf->SurfObj,
//                              (CLIPOBJ *)&dc->co,
//                              BrushObj,
//                              x1,
//                              ScanLine,
//                              x2,
//                              ScanLine,
//                              &BoundRect,  // Bounding rectangle
//                              RopMode);    // MIX
//
//                 x1 = newx1;
//                 x2 = newx2;
//             }
//         }
//
//         pLeft = pRight;
//         pRight = pLeft->pNext;
//         winding += pLeft->YDirection;
//     }
//
//     // There will always be a line left-over, render it now...
//     BoundRect.left = x1;
//     BoundRect.right = x2;
//
//     //DPRINT("Fill Line (%d, %d) to (%d, %d)\n",x1, ScanLine, x2, ScanLine);
//     IntEngLineTo(&psurf->SurfObj,
//                  (CLIPOBJ *)&dc->co,
//                  BrushObj,
//                  x1,
//                  ScanLine,
//                  x2,
//                  ScanLine,
//                  &BoundRect,  // Bounding rectangle
//                  RopMode);    // MIX
// }
//
// // When the fill mode is ALTERNATE, GDI fills the area between odd-numbered and
// // even-numbered polygon sides on each scan line. That is, GDI fills the area between the
// // first and second side, between the third and fourth side, and so on.
//
// // WINDING Selects winding mode (fills any region with a nonzero winding value).
// // When the fill mode is WINDING, GDI fills any region that has a nonzero winding value.
// // This value is defined as the number of times a pen used to draw the polygon would go around the region.
// // The direction of each edge of the polygon is important.
//
// BOOL
// APIENTRY
// FillPolygon(
//     PDC dc,
//     SURFACE *psurf,
//     BRUSHOBJ *BrushObj,
//     MIX RopMode,
//     CONST PPOINT Points,
//     int Count,
//     RECTL BoundRect )
// {
//     FILL_EDGE_LIST *list = 0;
//     FILL_EDGE *ActiveHead = 0;
//     int ScanLine;
//     PDC_ATTR pdcattr = dc->pdcattr;
//     void
//     (APIENTRY *FillScanLine)(
//         PDC dc,
//         int ScanLine,
//         FILL_EDGE* ActiveHead,
//         SURFACE *psurf,
//         BRUSHOBJ *BrushObj,
//         MIX RopMode);
//
//     //DPRINT("FillPolygon\n");
//
//     /* Create Edge List. */
//     list = POLYGONFILL_MakeEdgeList(Points, Count);
//     /* DEBUG_PRINT_EDGELIST(list); */
//     if (NULL == list)
//         return FALSE;
//
//     if (WINDING == pdcattr->jFillMode)
//         FillScanLine = POLYGONFILL_FillScanLineWinding;
//     else /* Default */
//         FillScanLine = POLYGONFILL_FillScanLineAlternate;
//
//     /* For each Scanline from BoundRect.bottom to BoundRect.top,
//      * determine line segments to draw
//      */
//     for (ScanLine = BoundRect.top; ScanLine < BoundRect.bottom; ++ScanLine)
//     {
//         POLYGONFILL_BuildActiveList(ScanLine, list, &ActiveHead);
//         //DEBUG_PRINT_ACTIVE_EDGELIST(ActiveHead);
//         FillScanLine(dc, ScanLine, ActiveHead, psurf, BrushObj, RopMode);
//     }
//
//     /* Free Edge List. If any are left. */
//     POLYGONFILL_DestroyEdgeList(list);
//
//     return TRUE;
// }
//
// BOOL FASTCALL
// IntFillPolygon(
//     PDC dc,
//     SURFACE *psurf,
//     BRUSHOBJ *BrushObj,
//     CONST PPOINT Points,
//     int Count,
//     RECTL DestRect,
//     POINTL *BrushOrigin)
// {
//     FILL_EDGE_LIST *list = 0;
//     FILL_EDGE *ActiveHead = 0;
//     FILL_EDGE *pLeft, *pRight;
//     int ScanLine;
//
//     //DPRINT("IntFillPolygon\n");
//
//     /* Create Edge List. */
//     list = POLYGONFILL_MakeEdgeList(Points, Count);
//     /* DEBUG_PRINT_EDGELIST(list); */
//     if (NULL == list)
//         return FALSE;
//
//     /* For each Scanline from DestRect.top to DestRect.bottom, determine line segments to draw */
//     for (ScanLine = DestRect.top; ScanLine < DestRect.bottom; ++ScanLine)
//     {
//         POLYGONFILL_BuildActiveList(ScanLine, list, &ActiveHead);
//         //DEBUG_PRINT_ACTIVE_EDGELIST(ActiveHead);
//
//         if (!ActiveHead)
//         {
//             POLYGONFILL_DestroyEdgeList(list);
//             return FALSE;
//         }
//
//         pLeft = ActiveHead;
//         pRight = pLeft->pNext;
//         ASSERT(pRight);
//
//         while (NULL != pRight)
//         {
//             int x1 = pLeft->XIntercept[0];
//             int x2 = pRight->XIntercept[1];
//
//             if (x2 > x1)
//             {
//                 RECTL LineRect;
//                 LineRect.top = ScanLine;
//                 LineRect.bottom = ScanLine + 1;
//                 LineRect.left = x1;
//                 LineRect.right = x2;
//
//                 IntEngBitBlt(&psurf->SurfObj,
//                              NULL,
//                              NULL,
//                              (CLIPOBJ *)&dc->co,
//                              NULL,
//                              &LineRect,
//                              NULL,
//                              NULL,
//                              BrushObj,
//                              BrushOrigin,
//                              ROP4_FROM_INDEX(R3_OPINDEX_PATCOPY));
//             }
//
//             pLeft = pRight->pNext;
//             pRight = pLeft ? pLeft->pNext : NULL;
//         }
//     }
//
//     /* Free Edge List. If any are left. */
//     POLYGONFILL_DestroyEdgeList(list);
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
