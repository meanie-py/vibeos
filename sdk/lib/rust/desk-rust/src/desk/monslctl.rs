// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static szMonitorSelWndClass: [TCHAR; 18] = [
    'M' as TCHAR, 'O' as TCHAR, 'N' as TCHAR, 'I' as TCHAR, 'T' as TCHAR, 'O' as TCHAR, 'R' as TCHAR, 'S' as TCHAR, 'E' as TCHAR, 'L' as TCHAR, 'W' as TCHAR, 'N' as TCHAR, 'D' as TCHAR, 'C' as TCHAR, 'L' as TCHAR, 'A' as TCHAR, 'S' as TCHAR, 'S' as TCHAR,
];

#[repr(C)]
pub struct MONSL_MON {
    pub rc: RECT,
    pub hFont: HFONT,
    pub szCaption: [TCHAR; 12],
}

pub type PMONSL_MON = *mut MONSL_MON;

#[repr(C)]
pub struct MONITORSELWND {
    pub hSelf: HWND,
    pub hNotify: HWND,
    pub hFont: HFONT,
    pub ClientSize: SIZE,
    pub UIState: DWORD,
    pub __bindgen_anon_1: MONITORSELWND__bindgen_ty_1,
    pub ControlExStyle: DWORD,
    pub MonitorsCount: DWORD,
    pub SelectedMonitor: INT,
    pub DraggingMonitor: INT,
    pub rcDragging: RECT,
    pub ptDrag: POINT,
    pub ptDragBegin: POINT,
    pub DraggingMargin: SIZE,
    pub MonitorInfo: PMONSL_MONINFO,
    pub Monitors: PMONSL_MON,
    pub rcExtent: RECT,
    pub rcMonitors: RECT,
    pub rcOldMonitors: RECT,
    pub ScrollPos: POINT,
    pub Margin: SIZE,
    pub SelectionFrame: SIZE,
    pub hbmDisabledPattern: HBITMAP,
    pub hbrDisabled: HBRUSH,
}

#[repr(C)]
pub union MONITORSELWND__bindgen_ty_1 {
    pub dwInternalFlags: DWORD,
    pub __bindgen_anon_1: MONITORSELWND__bindgen_ty_1__bindgen_ty_1,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct MONITORSELWND__bindgen_ty_1__bindgen_ty_1 {
    pub _bitfield_1: u32,
}

// The C's anonymous union/struct members, which C injects into MONITORSELWND's own name
// scope. Bit 0..4 in declaration order — LSB first, as every compiler
// targeting these three ABIs allocates them.
impl MONITORSELWND {
    #[inline(always)]
    pub unsafe fn Enabled(&self) -> UINT {
        (self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1 >> 0) & 1
    }
    #[inline(always)]
    pub unsafe fn set_Enabled(&mut self, value: UINT) {
        let bits = &mut self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1;
        *bits = (*bits & !(1 << 0)) | (((value != 0) as UINT) << 0);
    }
    #[inline(always)]
    pub unsafe fn HasFocus(&self) -> UINT {
        (self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1 >> 1) & 1
    }
    #[inline(always)]
    pub unsafe fn set_HasFocus(&mut self, value: UINT) {
        let bits = &mut self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1;
        *bits = (*bits & !(1 << 1)) | (((value != 0) as UINT) << 1);
    }
    #[inline(always)]
    pub unsafe fn CanDisplay(&self) -> UINT {
        (self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1 >> 2) & 1
    }
    #[inline(always)]
    pub unsafe fn set_CanDisplay(&mut self, value: UINT) {
        let bits = &mut self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1;
        *bits = (*bits & !(1 << 2)) | (((value != 0) as UINT) << 2);
    }
    #[inline(always)]
    pub unsafe fn LeftBtnDown(&self) -> UINT {
        (self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1 >> 3) & 1
    }
    #[inline(always)]
    pub unsafe fn set_LeftBtnDown(&mut self, value: UINT) {
        let bits = &mut self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1;
        *bits = (*bits & !(1 << 3)) | (((value != 0) as UINT) << 3);
    }
    #[inline(always)]
    pub unsafe fn IsDraggingMonitor(&self) -> UINT {
        (self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1 >> 4) & 1
    }
    #[inline(always)]
    pub unsafe fn set_IsDraggingMonitor(&mut self, value: UINT) {
        let bits = &mut self.__bindgen_anon_1.__bindgen_anon_1._bitfield_1;
        *bits = (*bits & !(1 << 4)) | (((value != 0) as UINT) << 4);
    }
}

pub type PMONITORSELWND = *mut MONITORSELWND;
unsafe extern "C" fn MonSelNotify(
    infoPtr: *mut MONITORSELWND,
    code: UINT,
    data: PVOID,
) -> LRESULT {
    let mut Ret: LRESULT = 0;

    if (*infoPtr).hNotify != core::ptr::null_mut() {
        let pnmh = data as LPNMHDR;

        (*pnmh).hwndFrom = (*infoPtr).hSelf;
        (*pnmh).idFrom = GetWindowLongPtr((*infoPtr).hSelf, GWLP_ID) as UINT_PTR;
        (*pnmh).code = code;

        Ret = SendMessage(
            (*infoPtr).hNotify,
            WM_NOTIFY,
            (*pnmh).idFrom as WPARAM,
            pnmh as LPARAM,
        );
    }

    Ret
}
unsafe extern "C" fn MonSelNotifyMonitor(
    infoPtr: *mut MONITORSELWND,
    code: UINT,
    Index: INT,
    pmonnmh: *mut MONSL_MONNMHDR,
) -> LRESULT {
    (*pmonnmh).Index = Index;

    if Index >= 0 {
        (*pmonnmh).MonitorInfo = (*infoPtr).MonitorInfo.offset(Index as isize).read();
    } else {
        ZeroMemory!(
            core::ptr::addr_of_mut!((*pmonnmh).MonitorInfo) as *mut _,
            core::mem::size_of_val(&(*pmonnmh).MonitorInfo),
        );
    }

    MonSelNotify(infoPtr, code, pmonnmh as PVOID)
}
unsafe extern "C" fn MonSelChangeFont(
    infoPtr: *mut MONITORSELWND,
    hFont: HFONT,
    Redraw: BOOL,
) -> HFONT {
    let hOldFont = (*infoPtr).hFont;
    (*infoPtr).hFont = hFont;

    if Redraw != 0 {
        InvalidateRect(
            (*infoPtr).hSelf,
            core::ptr::null_mut(),
            TRUE,
        );
    }

    hOldFont
}
unsafe extern "C" fn MonSelRectToScreen(
    infoPtr: *mut MONITORSELWND,
    prc: *const RECT,
    prcOnScreen: PRECT,
) {
    *prcOnScreen = *prc;
    OffsetRect(
        prcOnScreen,
        -(*infoPtr).ScrollPos.x,
        -(*infoPtr).ScrollPos.y,
    );
}
unsafe extern "C" fn MonSelScreenToPt(
    infoPtr: PMONITORSELWND,
    pptOnScreen: *const POINT,
    ppt: PPOINT,
) {
    (*ppt).x = (*pptOnScreen).x + (*infoPtr).ScrollPos.x;
    (*ppt).y = (*pptOnScreen).y + (*infoPtr).ScrollPos.y;
}
unsafe extern "C" fn MonSelMonInfoToRect(
    pMonInfo: *const MONSL_MONINFO,
    prc: PRECT,
) {
    (*prc).left = (*pMonInfo).Position.x;
    (*prc).top = (*pMonInfo).Position.y;
    (*prc).right = (*pMonInfo).Position.x + (*pMonInfo).Size.cx;
    (*prc).bottom = (*pMonInfo).Position.y + (*pMonInfo).Size.cy;
}
unsafe extern "C" fn MonSelHitTest(
    infoPtr: PMONITORSELWND,
    ppt: *const POINT,
) -> INT {
    let mut pt: POINT = core::mem::zeroed();
    let mut Index: INT;
    let mut Ret: INT = -1;

    if (*infoPtr).CanDisplay() != 0 {
        MonSelScreenToPt(
            infoPtr,
            ppt,
            &mut pt,
        );

        Index = 0;
        while Index < (*infoPtr).MonitorsCount as INT {
            if PtInRect(
                &(*(*infoPtr).Monitors.offset(Index as isize)).rc,
                pt,
            ) != 0
            {
                Ret = Index;
                break;
            }
            Index += 1;
        }
    }

    return Ret;
}
unsafe extern "C" fn MonSelUpdateExtent(infoPtr: *mut MONITORSELWND) {
    let mut Index: DWORD;
    let mut rcMonitor: RECT = core::mem::zeroed();

    if (*infoPtr).MonitorsCount > 0 {
        MonSelMonInfoToRect(
            &(*infoPtr).MonitorInfo.offset(0 as isize),
            &mut (*infoPtr).rcExtent,
        );

        Index = 1;
        while Index < (*infoPtr).MonitorsCount {
            MonSelMonInfoToRect(
                &(*infoPtr).MonitorInfo.offset(Index as isize),
                &mut rcMonitor,
            );

            UnionRect(
                &mut (*infoPtr).rcExtent,
                &(*infoPtr).rcExtent,
                &rcMonitor,
            );

            Index += 1;
        }
    } else {
        ZeroMemory!(
            &mut (*infoPtr).rcExtent as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of_val(&(*infoPtr).rcExtent),
        );
    }
}
unsafe extern "C" fn MonSelScaleRectRelative(
    prcBaseFrom: *const RECT,
    prcFrom: *const RECT,
    prcBaseTo: *const RECT,
    prcTo: PRECT,
) {
    let mut BaseFrom: SIZE = core::mem::zeroed();
    let mut BaseTo: SIZE = core::mem::zeroed();
    let mut From: SIZE = core::mem::zeroed();

    BaseFrom.cx = (*prcBaseFrom).right - (*prcBaseFrom).left;
    BaseFrom.cy = (*prcBaseFrom).bottom - (*prcBaseFrom).top;
    BaseTo.cx = (*prcBaseTo).right - (*prcBaseTo).left;
    BaseTo.cy = (*prcBaseTo).bottom - (*prcBaseTo).top;
    From.cx = (*prcFrom).right - (*prcFrom).left;
    From.cy = (*prcFrom).bottom - (*prcFrom).top;

    (*prcTo).left = (*prcBaseTo).left + ((((*prcFrom).left - (*prcBaseFrom).left) * BaseTo.cx) / BaseFrom.cx);
    (*prcTo).top = (*prcBaseTo).top + ((((*prcFrom).top - (*prcBaseFrom).top) * BaseTo.cy) / BaseFrom.cy);
    (*prcTo).right = (*prcTo).left + ((From.cx * BaseTo.cx) / BaseFrom.cx);
    (*prcTo).bottom = (*prcTo).top + ((From.cy * BaseTo.cy) / BaseFrom.cy);
}
unsafe extern "C" fn ScaleRectSizeFit(prcContainerRect: *const RECT, prcRectToScale: PRECT) {
    let mut ContainerSize: SIZE = core::mem::zeroed();
    let mut RectSize: SIZE = core::mem::zeroed();

    ContainerSize.cx = (*prcContainerRect).right - (*prcContainerRect).left;
    ContainerSize.cy = (*prcContainerRect).bottom - (*prcContainerRect).top;
    RectSize.cx = (*prcRectToScale).right - (*prcRectToScale).left;
    RectSize.cy = (*prcRectToScale).bottom - (*prcRectToScale).top;

    if ((RectSize.cx * 0xFFF) / RectSize.cy) < ((ContainerSize.cx * 0xFFF) / ContainerSize.cy) {
        RectSize.cx = (RectSize.cx * ((ContainerSize.cy * 0xFFF) / RectSize.cy)) / 0xFFF;
        RectSize.cy = ContainerSize.cy;
    } else {
        RectSize.cy = (RectSize.cy * ((ContainerSize.cx * 0xFFF) / RectSize.cx)) / 0xFFF;
        RectSize.cx = ContainerSize.cx;
    }

    (*prcRectToScale).right = (*prcRectToScale).left + RectSize.cx;
    (*prcRectToScale).bottom = (*prcRectToScale).top + RectSize.cy;

    OffsetRect(
        prcRectToScale,
        (*prcContainerRect).left + ((ContainerSize.cx - RectSize.cx) / 2),
        (*prcContainerRect).top + ((ContainerSize.cy - RectSize.cy) / 2),
    );
}
unsafe extern "C" fn MonSelRepaint(infoPtr: *mut MONITORSELWND) {
    let mut rc: RECT = core::mem::zeroed();

    MonSelRectToScreen(
        infoPtr,
        core::ptr::addr_of!((*infoPtr).rcMonitors),
        &mut rc,
    );
    InvalidateRect(
        (*infoPtr).hSelf,
        &rc,
        TRUE,
    );

    if EqualRect(
        core::ptr::addr_of!((*infoPtr).rcMonitors),
        core::ptr::addr_of!((*infoPtr).rcOldMonitors),
    ) == 0
        && (*infoPtr).rcOldMonitors.right != (*infoPtr).rcOldMonitors.left
    {
        MonSelRectToScreen(infoPtr, core::ptr::addr_of!((*infoPtr).rcOldMonitors), &mut rc);
        InvalidateRect((*infoPtr).hSelf, &rc, TRUE);
        (*infoPtr).rcOldMonitors = (*infoPtr).rcMonitors;
    }
}
unsafe extern "C" fn MonSelRepaintMonitor(
    infoPtr: *mut MONITORSELWND,
    Index: DWORD,
) {
    let mut rc: RECT = core::mem::zeroed();
    let mut NoRepaint: BOOL = FALSE;

    if Index < (*infoPtr).MonitorsCount {
        if Index == (*infoPtr).DraggingMonitor as DWORD {
            if (*infoPtr).IsDraggingMonitor() != 0 {
                MonSelRectToScreen(
                    infoPtr,
                    &(*infoPtr).rcDragging,
                    &mut rc,
                );
            } else {
                NoRepaint = TRUE;
            }
        } else {
            MonSelRectToScreen(
                infoPtr,
                &(*(*infoPtr).Monitors.offset(Index as isize)).rc,
                &mut rc,
            );
        }

        if NoRepaint == 0 {
            InvalidateRect(
                (*infoPtr).hSelf,
                &rc,
                TRUE,
            );
        }
    }
}
unsafe extern "C" fn MonSelRepaintSelected(infoPtr: *mut MONITORSELWND) {
    if (*infoPtr).SelectedMonitor >= 0 {
        MonSelRepaintMonitor(infoPtr, (*infoPtr).SelectedMonitor as DWORD);
    }
}
unsafe extern "C" fn MonSelResetMonitors(infoPtr: PMONITORSELWND) {
    let mut Index: DWORD;

    Index = 0;
    while Index < (*infoPtr).MonitorsCount {
        if (*(*infoPtr).Monitors.offset(Index as isize)).hFont != core::ptr::null_mut() {
            DeleteObject((*(*infoPtr).Monitors.offset(Index as isize)).hFont);
            (*(*infoPtr).Monitors.offset(Index as isize)).hFont = core::ptr::null_mut();
        }
        Index = Index.wrapping_add(1);
    }
}
unsafe extern "C" fn MonSelUpdateMonitorsInfo(
    infoPtr: PMONITORSELWND,
    bRepaint: BOOL,
) {
    let mut rcExtSurface: RECT = core::mem::zeroed();
    let mut rcExtDisplay: RECT = core::mem::zeroed();
    let mut Index: DWORD;

    MonSelUpdateExtent(infoPtr);

    (*infoPtr).set_CanDisplay(((*infoPtr).MonitorsCount != 0 &&
                            ((*infoPtr).ClientSize.cx > (2 * ((*infoPtr).Margin.cx + (*infoPtr).SelectionFrame.cx))) &&
                            ((*infoPtr).ClientSize.cy > (2 * ((*infoPtr).Margin.cy + (*infoPtr).SelectionFrame.cy)))) as u32);

    if (*infoPtr).CanDisplay() != 0 {
        rcExtSurface.left = (*infoPtr).Margin.cx;
        rcExtSurface.top = (*infoPtr).Margin.cy;
        rcExtSurface.right = rcExtSurface.left + (*infoPtr).ClientSize.cx - (2 * (*infoPtr).Margin.cx);
        rcExtSurface.bottom = rcExtSurface.top + (*infoPtr).ClientSize.cy - (2 * (*infoPtr).Margin.cy);

        rcExtDisplay.left = 0;
        rcExtDisplay.top = 0;
        rcExtDisplay.right = (*infoPtr).rcExtent.right - (*infoPtr).rcExtent.left;
        rcExtDisplay.bottom = (*infoPtr).rcExtent.bottom - (*infoPtr).rcExtent.top;

        ScaleRectSizeFit(
            &rcExtSurface,
            &mut rcExtDisplay,
        );

        (*infoPtr).rcOldMonitors = (*infoPtr).rcMonitors;
        (*infoPtr).rcMonitors = rcExtDisplay;

        Index = 0;
        while Index < (*infoPtr).MonitorsCount {
            MonSelMonInfoToRect(
                &(*(*infoPtr).MonitorInfo.offset(Index as isize)),
                &mut rcExtDisplay,
            );

            MonSelScaleRectRelative(
                &(*infoPtr).rcExtent,
                &rcExtDisplay,
                &(*infoPtr).rcMonitors,
                &mut (*(*infoPtr).Monitors.offset(Index as isize)).rc,
            );

            Index = Index.wrapping_add(1);
        }

        MonSelResetMonitors(infoPtr);

        if bRepaint != 0 {
            MonSelRepaint(infoPtr);
        }
    } else if bRepaint != 0 {
        InvalidateRect(
            (*infoPtr).hSelf,
            core::ptr::null_mut(),
            TRUE,
        );
    }
}
unsafe extern "C" fn MonSelSetMonitorsInfo(
    infoPtr: PMONITORSELWND,
    dwMonitors: DWORD,
    MonitorsInfo: *const MONSL_MONINFO,
) -> BOOL {
    let mut Index: DWORD;
    let mut Ret: BOOL = TRUE;

    if (*infoPtr).DraggingMonitor >= 0 {
        return FALSE;
    }

    if (*infoPtr).MonitorInfo != core::ptr::null_mut() {
        LocalFree((*infoPtr).MonitorInfo as HLOCAL);
        (*infoPtr).MonitorInfo = core::ptr::null_mut();

        MonSelResetMonitors(infoPtr);

        LocalFree((*infoPtr).Monitors as HLOCAL);
        (*infoPtr).Monitors = core::ptr::null_mut();

        (*infoPtr).MonitorsCount = 0;
    }

    if dwMonitors != 0 {
        (*infoPtr).MonitorInfo = LocalAlloc(
            LMEM_FIXED,
            dwMonitors * core::mem::size_of::<MONSL_MONINFO>() as DWORD,
        ) as PMONSL_MONINFO;
        if (*infoPtr).MonitorInfo != core::ptr::null_mut() {
            (*infoPtr).Monitors = LocalAlloc(
                LMEM_FIXED,
                dwMonitors * core::mem::size_of::<MONSL_MON>() as DWORD,
            ) as PMONSL_MON;
            if (*infoPtr).Monitors != core::ptr::null_mut() {
                CopyMemory!(
                    (*infoPtr).MonitorInfo as *mut core::ffi::c_void,
                    MonitorsInfo as *const core::ffi::c_void,
                    dwMonitors * core::mem::size_of::<MONSL_MONINFO>() as DWORD,
                );
                ZeroMemory!(
                    (*infoPtr).Monitors as *mut core::ffi::c_void,
                    dwMonitors * core::mem::size_of::<MONSL_MON>() as DWORD,
                );

                Index = 0;
                while Index < dwMonitors {
                    _stprintf(
                        (*(*infoPtr).Monitors.offset(Index as isize)).szCaption.as_mut_ptr(),
                        w!("%u"),
                        Index + 1,
                    );
                    Index += 1;
                }

                (*infoPtr).MonitorsCount = dwMonitors;

                if (*infoPtr).SelectedMonitor >= (*infoPtr).MonitorsCount as INT {
                    (*infoPtr).SelectedMonitor = -1;
                }

                if ((*infoPtr).ControlExStyle & MSLM_EX_ALLOWSELECTNONE) == 0
                    && (*infoPtr).SelectedMonitor < 0
                {
                    (*infoPtr).SelectedMonitor = 0;
                }

                MonSelUpdateMonitorsInfo(infoPtr, TRUE);
            } else {
                LocalFree((*infoPtr).MonitorInfo as HLOCAL);
                (*infoPtr).MonitorInfo = core::ptr::null_mut();

                Ret = FALSE;
            }
        } else {
            Ret = FALSE;
        }
    }

    if Ret == 0 {
        (*infoPtr).SelectedMonitor = -1;
    }

    if Ret == 0 || dwMonitors == 0 {
        InvalidateRect(
            (*infoPtr).hSelf,
            core::ptr::null_mut(),
            TRUE,
        );
    }

    return Ret;
}
unsafe extern "C" fn MonSelGetMonitorsInfo(
    infoPtr: PMONITORSELWND,
    dwMonitors: DWORD,
    MonitorsInfo: PMONSL_MONINFO,
) -> DWORD {
    if dwMonitors != 0 {
        let mut dwMonitors = dwMonitors;
        if dwMonitors > (*infoPtr).MonitorsCount {
            dwMonitors = (*infoPtr).MonitorsCount;
        }

        CopyMemory!(
            MonitorsInfo as *mut core::ffi::c_void,
            (*infoPtr).MonitorInfo as *const core::ffi::c_void,
            (dwMonitors as usize) * core::mem::size_of::<MONSL_MONINFO>(),
        );
        dwMonitors
    } else {
        (*infoPtr).MonitorsCount
    }
}
unsafe extern "C" fn MonSelSetMonitorInfo(
    infoPtr: PMONITORSELWND,
    Index: INT,
    MonitorsInfo: *const MONSL_MONINFO,
) -> BOOL {
    if (*infoPtr).DraggingMonitor < 0 &&
        Index >= 0 && Index < (*infoPtr).MonitorsCount as INT
    {
        CopyMemory!(
            &mut (*infoPtr).MonitorInfo.offset(Index as isize),
            MonitorsInfo as *const core::ffi::c_void,
            core::mem::size_of::<MONSL_MONINFO>(),
        );

        MonSelUpdateMonitorsInfo(
            infoPtr,
            TRUE,
        );
        return TRUE;
    }

    return FALSE;
}
unsafe extern "C" fn MonSelGetMonitorInfo(
    infoPtr: PMONITORSELWND,
    Index: INT,
    MonitorsInfo: PMONSL_MONINFO,
) -> BOOL {
    if Index >= 0 && Index < (*infoPtr).MonitorsCount as INT {
        CopyMemory!(
            MonitorsInfo as *mut core::ffi::c_void,
            core::ptr::addr_of!((*(*infoPtr).MonitorInfo.offset(Index as isize))) as *const core::ffi::c_void,
            core::mem::size_of::<MONSL_MONINFO>(),
        );
        return TRUE;
    }

    FALSE
}
unsafe extern "C" fn MonSelGetMonitorRect(
    infoPtr: *mut MONITORSELWND,
    Index: INT,
    prc: PRECT,
) -> INT {
    let mut rc: RECT = core::mem::zeroed();
    let mut rcClient: RECT = core::mem::zeroed();

    if Index < 0 || (Index as UINT) >= (*infoPtr).MonitorsCount {
        return -1;
    }

    if (*infoPtr).CanDisplay() == 0 {
        return 0;
    }

    MonSelRectToScreen(
        infoPtr,
        core::ptr::addr_of!((*(*infoPtr).Monitors.offset(Index as isize)).rc),
        prc,
    );

    rcClient.left = 0;
    rcClient.top = 0;
    rcClient.right = (*infoPtr).ClientSize.cx;
    rcClient.bottom = (*infoPtr).ClientSize.cy;

    (IntersectRect(core::ptr::addr_of_mut!(rc), core::ptr::addr_of!(rcClient), prc) != FALSE) as INT
}
unsafe extern "C" fn MonSelSetCurSelMonitor(
    infoPtr: *mut MONITORSELWND,
    Index: INT,
    bNotify: BOOL,
) -> BOOL {
    let mut PrevSel: INT;
    let mut PreventSelect: BOOL = FALSE;
    let mut Ret: BOOL = FALSE;

    if (*infoPtr).DraggingMonitor < 0
        && (Index == -1 || Index < (*infoPtr).MonitorsCount as INT)
    {
        if Index != (*infoPtr).SelectedMonitor {
            if ((*(*infoPtr).MonitorInfo.offset(Index as isize)).Flags & MSL_MIF_DISABLED != 0)
                && ((*infoPtr).ControlExStyle & MSLM_EX_ALLOWSELECTDISABLED == 0)
            {
                PreventSelect = TRUE;
            }

            if PreventSelect == 0 && bNotify != 0 {
                let mut nmi: MONSL_MONNMMONITORCHANGING = core::mem::zeroed();

                nmi.PreviousSelected = (*infoPtr).SelectedMonitor;
                nmi.AllowChanging = TRUE;

                MonSelNotifyMonitor(
                    infoPtr,
                    MSLN_MONITORCHANGING,
                    Index,
                    &mut nmi.hdr,
                );

                PreventSelect = if nmi.AllowChanging == FALSE { 1 } else { 0 };
            }

            if PreventSelect == 0 {
                PrevSel = (*infoPtr).SelectedMonitor;
                (*infoPtr).SelectedMonitor = Index;

                if PrevSel >= 0 {
                    MonSelRepaintMonitor(infoPtr, PrevSel as DWORD);
                }

                if (*infoPtr).SelectedMonitor >= 0 {
                    MonSelRepaintSelected(infoPtr);
                }

                if bNotify != 0 {
                    let mut nm: MONSL_MONNMHDR = core::mem::zeroed();

                    MonSelNotifyMonitor(
                        infoPtr,
                        MSLN_MONITORCHANGED,
                        Index,
                        &mut nm,
                    );
                }
            }
        }

        Ret = TRUE;
    }

    Ret
}
unsafe extern "C" fn MonSelCreate(infoPtr: PMONITORSELWND) {
    (*infoPtr).SelectionFrame.cx = 4;
    (*infoPtr).SelectionFrame.cy = 4;
    (*infoPtr).Margin.cx = 20;
    (*infoPtr).Margin.cy = 20;
    (*infoPtr).SelectedMonitor = -1;
    (*infoPtr).DraggingMonitor = -1;
    (*infoPtr).ControlExStyle = MSLM_EX_ALLOWSELECTDISABLED | MSLM_EX_HIDENUMBERONSINGLE |
        MSLM_EX_SELECTONRIGHTCLICK | MSLM_EX_SELECTBYARROWKEY;
}
unsafe extern "C" fn MonSelDestroy(infoPtr: PMONITORSELWND) {
    MonSelSetMonitorsInfo(
        infoPtr,
        0,
        core::ptr::null(),
    );

    if (*infoPtr).hbrDisabled != core::ptr::null_mut() {
        DeleteObject((*infoPtr).hbrDisabled);
        (*infoPtr).hbrDisabled = core::ptr::null_mut();
    }

    if (*infoPtr).hbmDisabledPattern != core::ptr::null_mut() {
        DeleteObject((*infoPtr).hbmDisabledPattern);
        (*infoPtr).hbmDisabledPattern = core::ptr::null_mut();
    }
}
unsafe extern "C" fn MonSelSetExtendedStyle(
    infoPtr: *mut MONITORSELWND,
    dwExtendedStyle: DWORD,
) -> BOOL {
    if (*infoPtr).DraggingMonitor >= 0 {
        return FALSE;
    }

    if dwExtendedStyle != (*infoPtr).ControlExStyle {
        (*infoPtr).ControlExStyle = dwExtendedStyle;

        InvalidateRect(
            (*infoPtr).hSelf,
            core::ptr::null_mut(),
            TRUE,
        );
    }

    TRUE
}
unsafe extern "C" fn MonSelGetExtendedStyle(infoPtr: PMONITORSELWND) -> DWORD {
    (*infoPtr).ControlExStyle
}
unsafe extern "C" fn MonSelGetMonitorFont(
    infoPtr: *mut MONITORSELWND,
    hDC: HDC,
    Index: i32,
) -> HFONT {
    let mut tm: TEXTMETRIC = core::mem::zeroed();
    let mut rcsize: SIZE = core::mem::zeroed();
    let mut lf: LOGFONT = core::mem::zeroed();
    let hPrevFont: HFONT;
    let mut hFont: HFONT;

    hFont = (*(*infoPtr).Monitors.offset(Index as isize)).hFont;
    if hFont.is_null()
        && GetObject(
            (*infoPtr).hFont,
            core::mem::size_of::<LOGFONT>() as i32,
            &mut lf as *mut _ as *mut core::ffi::c_void,
        ) != 0
    {
        rcsize.cx = (*(*infoPtr).Monitors.offset(Index as isize)).rc.right
            - (*(*infoPtr).Monitors.offset(Index as isize)).rc.left
            - (2 * (*infoPtr).SelectionFrame.cx)
            - 2;
        rcsize.cy = (*(*infoPtr).Monitors.offset(Index as isize)).rc.bottom
            - (*(*infoPtr).Monitors.offset(Index as isize)).rc.top
            - (2 * (*infoPtr).SelectionFrame.cy)
            - 2;
        rcsize.cy = (rcsize.cy * 60) / 100;

        hPrevFont = SelectObject(hDC, (*infoPtr).hFont);

        if GetTextMetrics(hDC, &mut tm) != 0 {
            lf.lfWeight = FW_SEMIBOLD;
            lf.lfHeight = -MulDiv(
                rcsize.cy - tm.tmExternalLeading,
                GetDeviceCaps(hDC, LOGPIXELSY),
                72,
            );

            hFont = CreateFontIndirect(&lf);
            if !hFont.is_null() {
                (*(*infoPtr).Monitors.offset(Index as isize)).hFont = hFont;
            }
        }

        SelectObject(hDC, hPrevFont);
    }

    hFont
}
unsafe extern "C" fn MonSelDrawDisabledRect(
    infoPtr: *mut MONITORSELWND,
    hDC: HDC,
    prc: *const RECT,
) -> BOOL {
    let mut Ret: BOOL = FALSE;

    if (*infoPtr).hbrDisabled.is_null() {
        static Pattern: [DWORD; 4] = [
            0x5555AAAA,
            0x5555AAAA,
            0x5555AAAA,
            0x5555AAAA,
        ];

        if (*infoPtr).hbmDisabledPattern.is_null() {
            (*infoPtr).hbmDisabledPattern = CreateBitmap(
                8,
                8,
                1,
                1,
                Pattern.as_ptr(),
            );
        }

        if !(*infoPtr).hbmDisabledPattern.is_null() {
            (*infoPtr).hbrDisabled = CreatePatternBrush((*infoPtr).hbmDisabledPattern);
        }
    }

    if !(*infoPtr).hbrDisabled.is_null() {
    }

    Ret
}
unsafe extern "C" fn MonSelPaintMonitor(
    infoPtr: *mut MONITORSELWND,
    hDC: HDC,
    Index: DWORD,
    prc: PRECT,
    crDefFontColor: COLORREF,
    bHideNumber: BOOL,
) {
    let mut hFont: HFONT;
    let mut hPrevFont: HFONT;
    let mut crPrevText: COLORREF;

    if Index as i32 == (*infoPtr).SelectedMonitor {
        FillRect(
            hDC,
            prc,
            (COLOR_HIGHLIGHT + 1) as HBRUSH,
        );

        if (*infoPtr).HasFocus() != 0 && ((*infoPtr).UIState & UISF_HIDEFOCUS) == 0 {
            crPrevText = SetTextColor(hDC, crDefFontColor);

            DrawFocusRect(hDC, prc);

            SetTextColor(hDC, crPrevText);
        }
    }

    InflateRect(
        prc,
        -((*infoPtr).SelectionFrame.cx),
        -((*infoPtr).SelectionFrame.cy),
    );

    Rectangle(
        hDC,
        (*prc).left,
        (*prc).top,
        (*prc).right,
        (*prc).bottom,
    );

    InflateRect(prc, -1, -1);

    if bHideNumber == 0 {
        hFont = MonSelGetMonitorFont(infoPtr, hDC, Index as i32);
        if !hFont.is_null() {
            hPrevFont = SelectObject(hDC, hFont);

            DrawText(
                hDC,
                (*(*infoPtr).Monitors.offset(Index as isize)).szCaption.as_ptr(),
                -1,
                prc,
                DT_VCENTER | DT_CENTER | DT_NOPREFIX | DT_SINGLELINE,
            );

            SelectObject(hDC, hPrevFont);
        }
    }

    if ((*(*infoPtr).MonitorInfo.offset(Index as isize)).Flags & MSL_MIF_DISABLED) != 0 {
        InflateRect(prc, 1, 1);

        MonSelDrawDisabledRect(infoPtr, hDC, prc);
    }
}
unsafe extern "C" fn MonSelPaint(
    infoPtr: *mut MONITORSELWND,
    hDC: HDC,
    prcUpdate: *const RECT,
) {
    let mut crPrevText: COLORREF;
    let hbBk: HBRUSH;
    let hbOldBk: HBRUSH;
    let hpFg: HPEN;
    let hpOldFg: HPEN;
    let mut Index: DWORD;
    let mut rc: RECT = core::mem::zeroed();
    let mut rctmp: RECT = core::mem::zeroed();
    let iPrevBkMode: INT;
    let bHideNumber: BOOL;

    bHideNumber = if ((*infoPtr).ControlExStyle & MSLM_EX_HIDENUMBERS) != 0 ||
        (((*infoPtr).MonitorsCount == 1) && ((*infoPtr).ControlExStyle & MSLM_EX_HIDENUMBERONSINGLE) != 0) {
        1
    } else {
        0
    };

    hbBk = GetSysColorBrush(COLOR_BACKGROUND);
    hpFg = CreatePen(PS_SOLID,
                     0,
                     GetSysColor(COLOR_HIGHLIGHTTEXT));

    hbOldBk = SelectObject(hDC,
                           hbBk);
    hpOldFg = SelectObject(hDC,
                           hpFg);
    iPrevBkMode = SetBkMode(hDC,
                            TRANSPARENT);
    crPrevText = SetTextColor(hDC,
                              GetSysColor(COLOR_HIGHLIGHTTEXT));

    Index = 0;
    while Index < (*infoPtr).MonitorsCount {
        if (*infoPtr).IsDraggingMonitor() != 0 &&
            (*infoPtr).DraggingMonitor as DWORD == Index
        {
            Index = Index.wrapping_add(1);
            continue;
        }

        MonSelRectToScreen(infoPtr,
                           core::ptr::addr_of!((*(*infoPtr).Monitors.offset(Index as isize)).rc),
                           &mut rc);

        if IntersectRect(&mut rctmp,
                          &rc,
                          prcUpdate) != 0
        {
            MonSelPaintMonitor(infoPtr,
                               hDC,
                               Index,
                               &mut rc,
                               crPrevText,
                               bHideNumber);
        }

        Index = Index.wrapping_add(1);
    }

    if (*infoPtr).IsDraggingMonitor() != 0 &&
        (*infoPtr).DraggingMonitor >= 0
    {
        MonSelRectToScreen(infoPtr,
                           core::ptr::addr_of!((*infoPtr).rcDragging),
                           &mut rc);

        if IntersectRect(&mut rctmp,
                          &rc,
                          prcUpdate) != 0
        {
            MonSelPaintMonitor(infoPtr,
                               hDC,
                               (*infoPtr).DraggingMonitor as DWORD,
                               &mut rc,
                               crPrevText,
                               bHideNumber);
        }
    }

    SetTextColor(hDC,
                 crPrevText);
    SetBkMode(hDC,
              iPrevBkMode);
    SelectObject(hDC,
                 hpOldFg);
    SelectObject(hDC,
                 hbOldBk);

    DeleteObject(hpFg);
}
unsafe extern "C" fn MonSelContextMenu(
    infoPtr: PMONITORSELWND,
    x: SHORT,
    y: SHORT,
) {
    let mut nm: MONSL_MONNMBUTTONCLICKED = core::mem::zeroed();
    let Index: INT;

    if (*infoPtr).HasFocus() == 0 {
        SetFocus((*infoPtr).hSelf);
    }

    nm.pt.x = x as i32;
    nm.pt.y = y as i32;

    Index = MonSelHitTest(
        infoPtr,
        &nm.pt,
    );

    MonSelNotifyMonitor(
        infoPtr,
        MSLN_RBUTTONUP,
        Index,
        &mut nm as *mut _ as *mut MONSL_MONNMHDR,
    );

    MapWindowPoints(
        (*infoPtr).hSelf,
        core::ptr::null_mut(),
        &nm.pt,
        1,
    );

    SendMessage(
        (*infoPtr).hSelf,
        WM_CONTEXTMENU,
        (*infoPtr).hSelf as WPARAM,
        MAKELPARAM!(nm.pt.x, nm.pt.y),
    );
}
unsafe extern "C" fn MonSelApplyCursorClipping(
    infoPtr: *mut MONITORSELWND,
    bClip: BOOL,
) {
    let mut rc: RECT = core::mem::zeroed();

    if bClip != 0 {
        rc.left = 0;
        rc.top = 0;
        rc.right = (*infoPtr).ClientSize.cx;
        rc.bottom = (*infoPtr).ClientSize.cy;

        if MapWindowPoints(
            (*infoPtr).hSelf,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(rc) as LPPOINT,
            2,
        ) != 0
        {
            ClipCursor(core::ptr::addr_of_mut!(rc));
        }
    } else {
        ClipCursor(core::ptr::null_mut());
    }
}
unsafe extern "C" fn MonSelMoveDragRect(infoPtr: *mut MONITORSELWND, ppt: *mut POINT) {
    let mut rcPrev: RECT;
    let mut rcUpdate: RECT = core::mem::zeroed();
    let mut prc: *mut RECT;
    let mut hRgnPrev: HRGN;
    let mut hDC: HDC;

    if (*infoPtr).CanDisplay() != 0 {
        hDC = GetDC((*infoPtr).hSelf);
        if hDC != core::ptr::null_mut() {
            if (*infoPtr).ptDrag.x != (*ppt).x || (*infoPtr).ptDrag.y != (*ppt).y {
                (*infoPtr).ptDrag = *ppt;

                rcPrev = (*infoPtr).rcDragging;

                prc = core::ptr::addr_of_mut!((*(*infoPtr).Monitors.offset((*infoPtr).DraggingMonitor as isize)).rc);
                (*infoPtr).rcDragging.left = (*ppt).x - (*infoPtr).DraggingMargin.cx;
                (*infoPtr).rcDragging.top = (*ppt).y - (*infoPtr).DraggingMargin.cy;
                (*infoPtr).rcDragging.right = (*infoPtr).rcDragging.left + ((*prc).right - (*prc).left);
                (*infoPtr).rcDragging.bottom = (*infoPtr).rcDragging.top + ((*prc).bottom - (*prc).top);

                hRgnPrev = CreateRectRgn(
                    rcPrev.left,
                    rcPrev.top,
                    rcPrev.right,
                    rcPrev.bottom,
                );

                if hRgnPrev != core::ptr::null_mut() {
                    if ScrollDC(
                        hDC,
                        (*infoPtr).rcDragging.left - rcPrev.left,
                        (*infoPtr).rcDragging.top - rcPrev.top,
                        core::ptr::addr_of_mut!(rcPrev),
                        core::ptr::null_mut(),
                        hRgnPrev,
                        core::ptr::addr_of_mut!(rcUpdate),
                    ) == 0
                        || InvalidateRgn(
                            (*infoPtr).hSelf,
                            hRgnPrev,
                            TRUE,
                        ) == 0
                    {
                        DeleteObject(hRgnPrev);
                        InvalidateRect(
                            (*infoPtr).hSelf,
                            core::ptr::addr_of_mut!(rcPrev),
                            TRUE,
                        );
                        InvalidateRect(
                            (*infoPtr).hSelf,
                            core::ptr::addr_of_mut!((*infoPtr).rcDragging),
                            TRUE,
                        );
                    } else {
                        DeleteObject(hRgnPrev);
                    }
                } else {
                    InvalidateRect(
                        (*infoPtr).hSelf,
                        core::ptr::addr_of_mut!(rcPrev),
                        TRUE,
                    );
                    InvalidateRect(
                        (*infoPtr).hSelf,
                        core::ptr::addr_of_mut!((*infoPtr).rcDragging),
                        TRUE,
                    );
                }
            }

            ReleaseDC((*infoPtr).hSelf, hDC);
        }
    }
}
unsafe extern "C" fn MonSelCancelDragging(infoPtr: *mut MONITORSELWND) {
    let Index: DWORD;

    if (*infoPtr).DraggingMonitor >= 0 {
        MonSelMoveDragRect(
            infoPtr,
            core::ptr::addr_of_mut!((*infoPtr).ptDragBegin),
        );

        Index = (*infoPtr).DraggingMonitor as DWORD;
        (*infoPtr).DraggingMonitor = -1;

        if (*infoPtr).CanDisplay() != 0 {
            MonSelRepaintMonitor(infoPtr, Index);

            (*infoPtr).set_IsDraggingMonitor(FALSE);

            MonSelRepaintMonitor(infoPtr, Index);
        } else {
            (*infoPtr).set_IsDraggingMonitor(FALSE);
        }

        ReleaseCapture();

        MonSelApplyCursorClipping(infoPtr, FALSE);
    }
}
unsafe extern "C" fn MonSelInitDragging(
    infoPtr: *mut MONITORSELWND,
    Index: DWORD,
    ppt: *mut POINT,
) {
    let mut pt: POINT = core::mem::zeroed();

    MonSelCancelDragging(infoPtr);
    (*infoPtr).set_IsDraggingMonitor(FALSE);

    MonSelScreenToPt(
        infoPtr,
        ppt,
        &mut pt,
    );

    (*infoPtr).ptDrag = pt;
    (*infoPtr).ptDragBegin = pt;
    (*infoPtr).DraggingMonitor = Index as INT;

    (*infoPtr).DraggingMargin.cx = (*ppt).x - (*(*infoPtr).Monitors.offset(Index as isize)).rc.left;
    (*infoPtr).DraggingMargin.cy = (*ppt).y - (*(*infoPtr).Monitors.offset(Index as isize)).rc.top;
    (*infoPtr).rcDragging = (*(*infoPtr).Monitors.offset(Index as isize)).rc;

    MonSelApplyCursorClipping(
        infoPtr,
        TRUE,
    );
}
unsafe extern "C" fn MonSelDrag(infoPtr: PMONITORSELWND, ppt: *mut POINT) {
    let mut szDrag: SIZE = core::mem::zeroed();
    let mut pt: POINT = core::mem::zeroed();
    let mut rcDrag: RECT = core::mem::zeroed();

    if (*infoPtr).DraggingMonitor >= 0 {
        MonSelScreenToPt(infoPtr, ppt, &mut pt);

        if (*infoPtr).IsDraggingMonitor() == 0 {
            szDrag.cx = GetSystemMetrics(SM_CXDRAG);
            szDrag.cy = GetSystemMetrics(SM_CYDRAG);

            rcDrag.left = (*(*infoPtr).Monitors.add((*infoPtr).DraggingMonitor as usize)).rc.left
                + (*infoPtr).DraggingMargin.cx
                - (szDrag.cx / 2);
            rcDrag.top = (*(*infoPtr).Monitors.add((*infoPtr).DraggingMonitor as usize)).rc.top
                + (*infoPtr).DraggingMargin.cy
                - (szDrag.cy / 2);
            rcDrag.right = rcDrag.left + szDrag.cx;
            rcDrag.bottom = rcDrag.top + szDrag.cy;

            if PtInRect(&rcDrag, pt) == 0 {
                (*infoPtr).set_IsDraggingMonitor(TRUE);
                MonSelMoveDragRect(infoPtr, &mut pt);
            }
        } else {
            MonSelMoveDragRect(infoPtr, &mut pt);
        }
    }
}
unsafe extern "system" fn MonitorSelWndProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: PMONITORSELWND;
    let mut Ret: LRESULT = 0;

    infoPtr = GetWindowLongPtrW(hwnd, 0) as PMONITORSELWND;

    if infoPtr == core::ptr::null_mut() && uMsg != WM_CREATE {
        Ret = DefWindowProcW(hwnd, uMsg, wParam, lParam);
        return Ret;
    }

    match uMsg {
        WM_PAINT | WM_PRINTCLIENT => {
            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            let hDC: HDC;

            if wParam != 0 {
                if GetUpdateRect(hwnd, core::ptr::addr_of_mut!(ps.rcPaint), TRUE) == 0 {
                } else {
                    hDC = wParam as HDC;
                    if (*infoPtr).CanDisplay() != 0 {
                        MonSelPaint(infoPtr, hDC, core::ptr::addr_of!(ps.rcPaint));
                    }
                }
            } else {
                hDC = BeginPaint(hwnd, core::ptr::addr_of_mut!(ps));
                if hDC != core::ptr::null_mut() {
                    if (*infoPtr).CanDisplay() != 0 {
                        MonSelPaint(infoPtr, hDC, core::ptr::addr_of!(ps.rcPaint));
                    }
                    EndPaint(hwnd, core::ptr::addr_of!(ps));
                }
            }
        }

        WM_MOUSEMOVE => {
            let mut pt: POINT = core::mem::zeroed();

            if (wParam & MK_LBUTTON) == 0 {
                MonSelCancelDragging(infoPtr);
            } else {
                if (*infoPtr).LeftBtnDown() != 0 {
                    pt.x = LOWORD!(lParam) as LONG;
                    pt.y = HIWORD!(lParam) as LONG;

                    MonSelDrag(infoPtr, core::ptr::addr_of_mut!(pt));
                }
            }
        }

        WM_RBUTTONDOWN => {
            if ((*infoPtr).ControlExStyle & MSLM_EX_SELECTONRIGHTCLICK) == 0 {
            } else {
                let Index: INT;
                let mut pt: POINT = core::mem::zeroed();

                if (*infoPtr).HasFocus() == 0 {
                    SetFocus((*infoPtr).hSelf);
                }

                pt.x = LOWORD!(lParam) as LONG;
                pt.y = HIWORD!(lParam) as LONG;

                Index = MonSelHitTest(infoPtr, core::ptr::addr_of!(pt));
                if Index >= 0 || ((*infoPtr).ControlExStyle & MSLM_EX_ALLOWSELECTNONE) != 0 {
                    MonSelSetCurSelMonitor(infoPtr, Index, TRUE);
                }

                if Index >= 0 && (uMsg == WM_LBUTTONDOWN || uMsg == WM_LBUTTONDBLCLK) {
                    (*infoPtr).set_LeftBtnDown(TRUE);
                    MonSelInitDragging(infoPtr, Index as DWORD, core::ptr::addr_of_mut!(pt));
                }

                if (*infoPtr).HasFocus() == 0 {
                    SetFocus(hwnd);
                }
            }
        }

        WM_LBUTTONDBLCLK | WM_LBUTTONDOWN => {
            let Index: INT;
            let mut pt: POINT = core::mem::zeroed();

            if (*infoPtr).HasFocus() == 0 {
                SetFocus((*infoPtr).hSelf);
            }

            pt.x = LOWORD!(lParam) as LONG;
            pt.y = HIWORD!(lParam) as LONG;

            Index = MonSelHitTest(infoPtr, core::ptr::addr_of!(pt));
            if Index >= 0 || ((*infoPtr).ControlExStyle & MSLM_EX_ALLOWSELECTNONE) != 0 {
                MonSelSetCurSelMonitor(infoPtr, Index, TRUE);
            }

            if Index >= 0 && (uMsg == WM_LBUTTONDOWN || uMsg == WM_LBUTTONDBLCLK) {
                (*infoPtr).set_LeftBtnDown(TRUE);
                MonSelInitDragging(infoPtr, Index as DWORD, core::ptr::addr_of_mut!(pt));
            }

            if (*infoPtr).HasFocus() == 0 {
                SetFocus(hwnd);
            }
        }

        WM_MBUTTONDOWN => {
            if (*infoPtr).HasFocus() == 0 {
                SetFocus(hwnd);
            }
        }

        WM_RBUTTONUP => {
            MonSelContextMenu(infoPtr, LOWORD!(lParam) as SHORT, HIWORD!(lParam) as SHORT);
        }

        WM_LBUTTONUP => {
            MonSelCancelDragging(infoPtr);
            (*infoPtr).set_LeftBtnDown(FALSE);
        }

        WM_GETDLGCODE => {
            let virtKey: INT;

            virtKey = if lParam != 0 {
                (*(lParam as LPMSG)).wParam as INT
            } else {
                0
            };
            match virtKey {
                VK_TAB => {
                    SendMessage(
                        GetAncestor(hwnd, GA_PARENT),
                        WM_CHANGEUISTATE,
                        MAKEWPARAM!(UIS_INITIALIZE, 0),
                        0,
                    );
                }
                _ => {}
            }

            Ret |= DLGC_WANTARROWS;

            if ((*infoPtr).ControlExStyle & MSLM_EX_SELECTBYNUMKEY) != 0 {
                Ret |= DLGC_WANTCHARS;
            }
        }

        WM_SETFOCUS => {
            (*infoPtr).set_HasFocus(TRUE);
            MonSelRepaintSelected(infoPtr);
        }

        WM_KILLFOCUS => {
            (*infoPtr).set_HasFocus(FALSE);
            MonSelCancelDragging(infoPtr);
            MonSelRepaintSelected(infoPtr);
        }

        WM_UPDATEUISTATE => {
            let OldUIState: DWORD;

            Ret = DefWindowProcW(hwnd, uMsg, wParam, lParam);

            OldUIState = (*infoPtr).UIState;
            match LOWORD!(wParam) {
                UIS_SET => {
                    (*infoPtr).UIState |= HIWORD!(wParam);
                }

                UIS_CLEAR => {
                    (*infoPtr).UIState &= !HIWORD!(wParam) as DWORD;
                }
                _ => {}
            }

            if (*infoPtr).UIState != OldUIState {
                MonSelRepaintSelected(infoPtr);
            }
        }

        WM_SETFONT => {
            Ret = MonSelChangeFont(infoPtr, wParam as HFONT, LOWORD!(lParam) as BOOL) as LRESULT;
        }

        WM_SIZE => {
            (*infoPtr).ClientSize.cx = LOWORD!(lParam) as LONG;
            (*infoPtr).ClientSize.cy = HIWORD!(lParam) as LONG;

            MonSelUpdateMonitorsInfo(infoPtr, FALSE);
            InvalidateRect((*infoPtr).hSelf, core::ptr::null_mut(), TRUE);
        }

        WM_GETFONT => {
            Ret = (*infoPtr).hFont as LRESULT;
        }

        WM_ENABLE => {
            (*infoPtr).set_Enabled(if wParam as BOOL != FALSE { 1 } else { 0 });
            MonSelRepaint(infoPtr);
        }

        WM_STYLECHANGED => {
            if wParam == GWL_STYLE {
                let OldEnabled: u32 = (*infoPtr).Enabled();
                (*infoPtr).set_Enabled(if ((*(lParam as LPSTYLESTRUCT)).styleNew & WS_DISABLED) != 0 { 0 } else { 1 });

                if OldEnabled != (*infoPtr).Enabled() {
                    MonSelRepaint(infoPtr);
                }
            }
        }

        WM_KEYDOWN => {
            let mut Index: INT;

            if ((*infoPtr).ControlExStyle & MSLM_EX_SELECTBYARROWKEY) != 0 {
                match wParam {
                    VK_UP | VK_LEFT => {
                        Index = (*infoPtr).SelectedMonitor;

                        if (*infoPtr).MonitorsCount != 0 {
                            if Index < 0 {
                                Index = 0;
                            } else if Index > 0 {
                                Index -= 1;
                            }
                        }

                        if Index >= 0 {
                            MonSelSetCurSelMonitor(infoPtr, Index, TRUE);
                        }
                    }

                    VK_DOWN | VK_RIGHT => {
                        Index = (*infoPtr).SelectedMonitor;

                        if (*infoPtr).MonitorsCount != 0 {
                            if Index < 0 {
                                Index = (*infoPtr).MonitorsCount as INT - 1;
                            } else if Index < (*infoPtr).MonitorsCount as INT - 1 {
                                Index += 1;
                            }
                        }

                        if (*infoPtr).SelectedMonitor < (*infoPtr).MonitorsCount as INT {
                            MonSelSetCurSelMonitor(infoPtr, Index, TRUE);
                        }
                    }
                    _ => {}
                }
            }
        }

        WM_CHAR => {
            if ((*infoPtr).ControlExStyle & MSLM_EX_SELECTBYNUMKEY) != 0
                && wParam >= '1' as WPARAM
                && wParam <= '9' as WPARAM
            {
                let Index: INT = (wParam - '1' as WPARAM) as INT;
                if Index < (*infoPtr).MonitorsCount as INT {
                    MonSelSetCurSelMonitor(infoPtr, Index, TRUE);
                }
            }
        }

        MSLM_SETMONITORSINFO => {
            Ret = MonSelSetMonitorsInfo(infoPtr, wParam as DWORD, lParam as *const MONSL_MONINFO) as LRESULT;
        }

        MSLM_GETMONITORSINFO => {
            Ret = MonSelGetMonitorsInfo(infoPtr, wParam as DWORD, lParam as PMONSL_MONINFO) as LRESULT;
        }

        MSLM_GETMONITORINFOCOUNT => {
            Ret = (*infoPtr).MonitorsCount as LRESULT;
        }

        MSLM_HITTEST => {
            Ret = MonSelHitTest(infoPtr, wParam as *const POINT) as LRESULT;
        }

        MSLM_SETCURSEL => {
            Ret = MonSelSetCurSelMonitor(infoPtr, wParam as INT, FALSE) as LRESULT;
        }

        MSLM_GETCURSEL => {
            Ret = (*infoPtr).SelectedMonitor as LRESULT;
        }

        MSLM_SETMONITORINFO => {
            Ret = MonSelSetMonitorInfo(infoPtr, wParam as INT, lParam as *const MONSL_MONINFO) as LRESULT;
        }

        MSLM_GETMONITORINFO => {
            Ret = MonSelGetMonitorInfo(infoPtr, wParam as INT, lParam as PMONSL_MONINFO) as LRESULT;
        }

        MSLM_SETEXSTYLE => {
            Ret = MonSelSetExtendedStyle(infoPtr, lParam as DWORD) as LRESULT;
        }

        MSLM_GETEXSTYLE => {
            Ret = MonSelGetExtendedStyle(infoPtr) as LRESULT;
        }

        MSLM_GETMONITORRECT => {
            Ret = MonSelGetMonitorRect(infoPtr, wParam as INT, lParam as PRECT) as LRESULT;
        }

        WM_CREATE => {
            infoPtr = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<MONITORSELWND>()) as PMONITORSELWND;
            if infoPtr == core::ptr::null_mut() {
                Ret = -1 as LRESULT;
            } else {
                ZeroMemory!(infoPtr as *mut core::ffi::c_void, core::mem::size_of::<MONITORSELWND>());
                (*infoPtr).hSelf = hwnd;
                (*infoPtr).hNotify = (*(lParam as LPCREATESTRUCTW)).hwndParent;
                (*infoPtr).set_Enabled(if ((*(lParam as LPCREATESTRUCTW)).style & WS_DISABLED) != 0 { 0 } else { 1 });
                (*infoPtr).UIState = SendMessage(hwnd, WM_QUERYUISTATE, 0, 0) as DWORD;

                SetWindowLongPtrW(hwnd, 0, infoPtr as LONG_PTR);

                MonSelCreate(infoPtr);
            }
        }

        WM_DESTROY => {
            MonSelDestroy(infoPtr);

            HeapFree(GetProcessHeap(), 0, infoPtr as *mut core::ffi::c_void);
            SetWindowLongPtrW(hwnd, 0, 0 as DWORD_PTR);
        }

        _ => {
            Ret = DefWindowProcW(hwnd, uMsg, wParam, lParam);
        }
    }

    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn RegisterMonitorSelectionControl(hInstance: HINSTANCE) -> BOOL {
    let mut wc: WNDCLASS = core::mem::zeroed();

    wc.style = CS_DBLCLKS;
    wc.lpfnWndProc = Some(MonitorSelWndProc);
    wc.cbWndExtra = core::mem::size_of::<PMONITORSELWND>() as i32;
    wc.hInstance = hInstance;
    wc.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as LPWSTR,
    );
    wc.hbrBackground = (COLOR_APPWORKSPACE + 1) as HBRUSH;
    wc.lpszClassName = szMonitorSelWndClass.as_ptr();

    (RegisterClass(&wc) != 0) as BOOL
}
#[no_mangle]
pub unsafe extern "C" fn UnregisterMonitorSelectionControl(
    hInstance: HINSTANCE,
) {
    UnregisterClassW(
        szMonitorSelWndClass.as_ptr(),
        hInstance,
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "desk.h"
//
// static const TCHAR szMonitorSelWndClass[] = TEXT("MONITORSELWNDCLASS");
//
// typedef struct _MONSL_MON
// {
//     RECT rc;
//     HFONT hFont;
//     TCHAR szCaption[12];
// } MONSL_MON, *PMONSL_MON;
//
// typedef struct _MONITORSELWND
// {
//     HWND hSelf;
//     HWND hNotify;
//     HFONT hFont;
//     SIZE ClientSize;
//     DWORD UIState;
//     union
//     {
//         DWORD dwInternalFlags;
//         struct
//         {
//             UINT Enabled : 1;
//             UINT HasFocus : 1;
//             UINT CanDisplay : 1;
//             UINT LeftBtnDown : 1;
//             UINT IsDraggingMonitor : 1;
//         };
//     };
//     DWORD ControlExStyle;
//     DWORD MonitorsCount;
//     INT SelectedMonitor;
//     INT DraggingMonitor;
//     RECT rcDragging;
//     POINT ptDrag, ptDragBegin;
//     SIZE DraggingMargin;
//     PMONSL_MONINFO MonitorInfo;
//     PMONSL_MON Monitors;
//     RECT rcExtent;
//     RECT rcMonitors;
//     RECT rcOldMonitors;
//     POINT ScrollPos;
//     SIZE Margin;
//     SIZE SelectionFrame;
//     HBITMAP hbmDisabledPattern;
//     HBRUSH hbrDisabled;
// } MONITORSELWND, *PMONITORSELWND;
//
// static LRESULT
// MonSelNotify( PMONITORSELWND infoPtr,
//               UINT code,
//                PVOID data)
// {
//     LRESULT Ret = 0;
//
//     if (infoPtr->hNotify != NULL)
//     {
//         LPNMHDR pnmh = (LPNMHDR)data;
//
//         pnmh->hwndFrom = infoPtr->hSelf;
//         pnmh->idFrom = GetWindowLongPtr(infoPtr->hSelf,
//                                         GWLP_ID);
//         pnmh->code = code;
//
//         Ret = SendMessage(infoPtr->hNotify,
//                           WM_NOTIFY,
//                           (WPARAM)pnmh->idFrom,
//                           (LPARAM)pnmh);
//     }
//
//     return Ret;
// }
//
// static LRESULT
// MonSelNotifyMonitor( PMONITORSELWND infoPtr,
//                      UINT code,
//                      INT Index,
//                       PMONSL_MONNMHDR pmonnmh)
// {
//     pmonnmh->Index = Index;
//
//     if (Index >= 0)
//     {
//         pmonnmh->MonitorInfo = infoPtr->MonitorInfo[Index];
//     }
//     else
//     {
//         ZeroMemory(&pmonnmh->MonitorInfo,
//                    sizeof(pmonnmh->MonitorInfo));
//     }
//
//     return MonSelNotify(infoPtr,
//                         code,
//                         pmonnmh);
// }
//
// static HFONT
// MonSelChangeFont(  PMONITORSELWND infoPtr,
//                   HFONT hFont,
//                   BOOL Redraw)
// {
//     HFONT hOldFont = infoPtr->hFont;
//     infoPtr->hFont = hFont;
//
//     if (Redraw)
//     {
//         InvalidateRect(infoPtr->hSelf,
//                        NULL,
//                        TRUE);
//     }
//
//     return hOldFont;
// }
//
// static VOID
// MonSelRectToScreen( PMONITORSELWND infoPtr,
//                     const RECT *prc,
//                     PRECT prcOnScreen)
// {
//     *prcOnScreen = *prc;
//     OffsetRect(prcOnScreen,
//                -infoPtr->ScrollPos.x,
//                -infoPtr->ScrollPos.y);
// }
//
// static VOID
// MonSelScreenToPt( PMONITORSELWND infoPtr,
//                   const POINT *pptOnScreen,
//                   PPOINT ppt)
// {
//     ppt->x = pptOnScreen->x + infoPtr->ScrollPos.x;
//     ppt->y = pptOnScreen->y + infoPtr->ScrollPos.y;
// }
//
// static VOID
// MonSelMonInfoToRect( const MONSL_MONINFO *pMonInfo,
//                      PRECT prc)
// {
//     prc->left = pMonInfo->Position.x;
//     prc->top = pMonInfo->Position.y;
//     prc->right = pMonInfo->Position.x + pMonInfo->Size.cx;
//     prc->bottom = pMonInfo->Position.y + pMonInfo->Size.cy;
// }
//
// static INT
// MonSelHitTest( PMONITORSELWND infoPtr,
//                const POINT *ppt)
// {
//     POINT pt;
//     INT Index, Ret = -1;
//
//     if (infoPtr->CanDisplay)
//     {
//         MonSelScreenToPt(infoPtr,
//                          ppt,
//                          &pt);
//
//         for (Index = 0; Index < (INT)infoPtr->MonitorsCount; Index++)
//         {
//             if (PtInRect(&infoPtr->Monitors[Index].rc,
//                          pt))
//             {
//                 Ret = Index;
//                 break;
//             }
//         }
//     }
//
//     return Ret;
// }
//
// static VOID
// MonSelUpdateExtent(  PMONITORSELWND infoPtr)
// {
//     DWORD Index;
//     RECT rcMonitor;
//
//     /* NOTE: This routine calculates the extent of all monitor coordinates.
//              These are not control coordinates! */
//     if (infoPtr->MonitorsCount > 0)
//     {
//         MonSelMonInfoToRect(&infoPtr->MonitorInfo[0],
//                             &infoPtr->rcExtent);
//
//         for (Index = 1; Index < infoPtr->MonitorsCount; Index++)
//         {
//             MonSelMonInfoToRect(&infoPtr->MonitorInfo[Index],
//                                 &rcMonitor);
//
//             UnionRect(&infoPtr->rcExtent,
//                       &infoPtr->rcExtent,
//                       &rcMonitor);
//         }
//     }
//     else
//     {
//         ZeroMemory(&infoPtr->rcExtent,
//                    sizeof(infoPtr->rcExtent));
//     }
// }
//
// static VOID
// MonSelScaleRectRelative( const RECT *prcBaseFrom,
//                          const RECT *prcFrom,
//                          const RECT *prcBaseTo,
//                          PRECT prcTo)
// {
//     SIZE BaseFrom, BaseTo, From;
//
//     BaseFrom.cx = prcBaseFrom->right - prcBaseFrom->left;
//     BaseFrom.cy = prcBaseFrom->bottom - prcBaseFrom->top;
//     BaseTo.cx = prcBaseTo->right - prcBaseTo->left;
//     BaseTo.cy = prcBaseTo->bottom - prcBaseTo->top;
//     From.cx = prcFrom->right - prcFrom->left;
//     From.cy = prcFrom->bottom - prcFrom->top;
//
//     prcTo->left = prcBaseTo->left + (((prcFrom->left - prcBaseFrom->left) * BaseTo.cx) / BaseFrom.cx);
//     prcTo->top = prcBaseTo->top + (((prcFrom->top - prcBaseFrom->top) * BaseTo.cy) / BaseFrom.cy);
//     prcTo->right = prcTo->left + ((From.cx * BaseTo.cx) / BaseFrom.cx);
//     prcTo->bottom = prcTo->top + ((From.cy * BaseTo.cy) / BaseFrom.cy);
// }
//
// static VOID
// ScaleRectSizeFit( const RECT *prcContainerRect,
//                    PRECT prcRectToScale)
// {
//     SIZE ContainerSize, RectSize;
//
//     ContainerSize.cx = prcContainerRect->right - prcContainerRect->left;
//     ContainerSize.cy = prcContainerRect->bottom - prcContainerRect->top;
//     RectSize.cx = prcRectToScale->right - prcRectToScale->left;
//     RectSize.cy = prcRectToScale->bottom - prcRectToScale->top;
//
//     if (((RectSize.cx * 0xFFF) / RectSize.cy) < ((ContainerSize.cx * 0xFFF) / ContainerSize.cy))
//     {
//         RectSize.cx = (RectSize.cx * ((ContainerSize.cy * 0xFFF) / RectSize.cy)) / 0xFFF;
//         RectSize.cy = ContainerSize.cy;
//     }
//     else
//     {
//         RectSize.cy = (RectSize.cy * ((ContainerSize.cx * 0xFFF) / RectSize.cx)) / 0xFFF;
//         RectSize.cx = ContainerSize.cx;
//     }
//
//     prcRectToScale->right = prcRectToScale->left + RectSize.cx;
//     prcRectToScale->bottom = prcRectToScale->top + RectSize.cy;
//
//     OffsetRect(prcRectToScale,
//                prcContainerRect->left + ((ContainerSize.cx - RectSize.cx) / 2),
//                prcContainerRect->top + ((ContainerSize.cy - RectSize.cy) / 2));
// }
//
// static VOID
// MonSelRepaint( PMONITORSELWND infoPtr)
// {
//     RECT rc;
//
//     MonSelRectToScreen(infoPtr,
//                        &infoPtr->rcMonitors,
//                        &rc);
//     InvalidateRect(infoPtr->hSelf,
//                    &rc,
//                    TRUE);
//
//     if (!EqualRect(&infoPtr->rcMonitors, &infoPtr->rcOldMonitors) &&
//         infoPtr->rcOldMonitors.right != infoPtr->rcOldMonitors.left)
//     {
//         MonSelRectToScreen(infoPtr, &infoPtr->rcOldMonitors, &rc);
//         InvalidateRect(infoPtr->hSelf, &rc, TRUE);
//         infoPtr->rcOldMonitors = infoPtr->rcMonitors;
//     }
// }
//
// static VOID
// MonSelRepaintMonitor( PMONITORSELWND infoPtr,
//                       DWORD Index)
// {
//     RECT rc;
//     BOOL NoRepaint = FALSE;
//
//     if (Index < infoPtr->MonitorsCount)
//     {
//         if (Index == (DWORD)infoPtr->DraggingMonitor)
//         {
//             if (infoPtr->IsDraggingMonitor)
//             {
//                 MonSelRectToScreen(infoPtr,
//                                    &infoPtr->rcDragging,
//                                    &rc);
//             }
//             else
//                 NoRepaint = TRUE;
//         }
//         else
//         {
//             MonSelRectToScreen(infoPtr,
//                                &infoPtr->Monitors[Index].rc,
//                                &rc);
//         }
//
//         if (!NoRepaint)
//         {
//             InvalidateRect(infoPtr->hSelf,
//                            &rc,
//                            TRUE);
//         }
//     }
// }
//
// static VOID
// MonSelRepaintSelected( PMONITORSELWND infoPtr)
// {
//     if (infoPtr->SelectedMonitor >= 0)
//     {
//         MonSelRepaintMonitor(infoPtr,
//                              (DWORD)infoPtr->SelectedMonitor);
//     }
// }
//
// static VOID
// MonSelResetMonitors(  PMONITORSELWND infoPtr)
// {
//     DWORD Index;
//
//     for (Index = 0; Index < infoPtr->MonitorsCount; Index++)
//     {
//         if (infoPtr->Monitors[Index].hFont != NULL)
//         {
//             DeleteObject(infoPtr->Monitors[Index].hFont);
//             infoPtr->Monitors[Index].hFont = NULL;
//         }
//     }
// }
//
//
// static VOID
// MonSelUpdateMonitorsInfo(  PMONITORSELWND infoPtr,
//                           BOOL bRepaint)
// {
//     RECT rcExtSurface, rcExtDisplay;
//     DWORD Index;
//
//     /* Recalculate rcExtent */
//     MonSelUpdateExtent(infoPtr);
//
//     infoPtr-> CanDisplay = infoPtr->MonitorsCount != 0 &&
//                            (infoPtr->ClientSize.cx > (2 * (infoPtr->Margin.cx + infoPtr->SelectionFrame.cx))) &&
//                            (infoPtr->ClientSize.cy > (2 * (infoPtr->Margin.cy + infoPtr->SelectionFrame.cy)));
//
//     if (infoPtr->CanDisplay)
//     {
//         /* Calculate the rectangle on the control in which may be painted */
//         rcExtSurface.left = infoPtr->Margin.cx;
//         rcExtSurface.top = infoPtr->Margin.cy;
//         rcExtSurface.right = rcExtSurface.left + infoPtr->ClientSize.cx - (2 * infoPtr->Margin.cx);
//         rcExtSurface.bottom = rcExtSurface.top + infoPtr->ClientSize.cy - (2 * infoPtr->Margin.cy);
//
//         /* Calculate the rectangle on the control that is actually painted on */
//         rcExtDisplay.left = rcExtDisplay.top = 0;
//         rcExtDisplay.right = infoPtr->rcExtent.right - infoPtr->rcExtent.left;
//         rcExtDisplay.bottom = infoPtr->rcExtent.bottom - infoPtr->rcExtent.top;
//
//         ScaleRectSizeFit(&rcExtSurface,
//                          &rcExtDisplay);
//
//         infoPtr->rcOldMonitors = infoPtr->rcMonitors;
//         infoPtr->rcMonitors = rcExtDisplay;
//
//         /* Now that we know in which area all monitors are located,
//            calculate the monitors selection rectangles on the screen */
//
//         for (Index = 0; Index < infoPtr->MonitorsCount; Index++)
//         {
//             MonSelMonInfoToRect(&infoPtr->MonitorInfo[Index],
//                                 &rcExtDisplay);
//
//             MonSelScaleRectRelative(&infoPtr->rcExtent,
//                                     &rcExtDisplay,
//                                     &infoPtr->rcMonitors,
//                                     &infoPtr->Monitors[Index].rc);
//         }
//
//         MonSelResetMonitors(infoPtr);
//
//         if (bRepaint)
//             MonSelRepaint(infoPtr);
//     }
//     else if (bRepaint)
//     {
//         InvalidateRect(infoPtr->hSelf,
//                        NULL,
//                        TRUE);
//     }
// }
//
// static BOOL
// MonSelSetMonitorsInfo(  PMONITORSELWND infoPtr,
//                        DWORD dwMonitors,
//                        const MONSL_MONINFO *MonitorsInfo)
// {
//     DWORD Index;
//     BOOL Ret = TRUE;
//
//     if (infoPtr->DraggingMonitor >= 0)
//         return FALSE;
//
//     if (infoPtr->MonitorInfo != NULL)
//     {
//         LocalFree((HLOCAL)infoPtr->MonitorInfo);
//         infoPtr->MonitorInfo = NULL;
//
//         MonSelResetMonitors(infoPtr);
//
//         LocalFree((HLOCAL)infoPtr->Monitors);
//         infoPtr->Monitors = NULL;
//
//         infoPtr->MonitorsCount = 0;
//     }
//
//     if (dwMonitors != 0)
//     {
//         infoPtr->MonitorInfo = (PMONSL_MONINFO)LocalAlloc(LMEM_FIXED,
//                                                           dwMonitors * sizeof(MONSL_MONINFO));
//         if (infoPtr->MonitorInfo != NULL)
//         {
//             infoPtr->Monitors = (PMONSL_MON)LocalAlloc(LMEM_FIXED,
//                                                        dwMonitors * sizeof(MONSL_MON));
//             if (infoPtr->Monitors != NULL)
//             {
//                 CopyMemory(infoPtr->MonitorInfo,
//                            MonitorsInfo,
//                            dwMonitors * sizeof(MONSL_MONINFO));
//                 ZeroMemory(infoPtr->Monitors,
//                            dwMonitors * sizeof(MONSL_MON));
//
//                 for (Index = 0; Index < dwMonitors; Index++)
//                 {
//                     _stprintf(infoPtr->Monitors[Index].szCaption,
//                               _T("%u"),
//                               Index + 1);
//                 }
//
//                 infoPtr->MonitorsCount = dwMonitors;
//
//                 if (infoPtr->SelectedMonitor >= (INT)infoPtr->MonitorsCount)
//                     infoPtr->SelectedMonitor = -1;
//
//                 if (!(infoPtr->ControlExStyle & MSLM_EX_ALLOWSELECTNONE) && infoPtr->SelectedMonitor < 0)
//                     infoPtr->SelectedMonitor = 0;
//
//                 MonSelUpdateMonitorsInfo(infoPtr,
//                                          TRUE);
//             }
//             else
//             {
//                 LocalFree((HLOCAL)infoPtr->MonitorInfo);
//                 infoPtr->MonitorInfo = NULL;
//
//                 Ret = FALSE;
//             }
//         }
//         else
//             Ret = FALSE;
//     }
//
//     if (!Ret)
//         infoPtr->SelectedMonitor = -1;
//
//     if (!Ret || dwMonitors == 0)
//     {
//         InvalidateRect(infoPtr->hSelf,
//                        NULL,
//                        TRUE);
//     }
//
//     return Ret;
// }
//
// static DWORD
// MonSelGetMonitorsInfo( PMONITORSELWND infoPtr,
//                        DWORD dwMonitors,
//                         PMONSL_MONINFO MonitorsInfo)
// {
//     if (dwMonitors != 0)
//     {
//         if (dwMonitors > infoPtr->MonitorsCount)
//             dwMonitors = infoPtr->MonitorsCount;
//
//         CopyMemory(MonitorsInfo,
//                    infoPtr->MonitorInfo,
//                    dwMonitors * sizeof(MONSL_MONINFO));
//         return dwMonitors;
//     }
//     else
//         return infoPtr->MonitorsCount;
// }
//
// static BOOL
// MonSelSetMonitorInfo(  PMONITORSELWND infoPtr,
//                       INT Index,
//                       const MONSL_MONINFO *MonitorsInfo)
// {
//     if (infoPtr->DraggingMonitor < 0 &&
//         Index >= 0 && Index < (INT)infoPtr->MonitorsCount)
//     {
//         CopyMemory(&infoPtr->MonitorInfo[Index],
//                    MonitorsInfo,
//                    sizeof(MONSL_MONINFO));
//
//         MonSelUpdateMonitorsInfo(infoPtr,
//                                  TRUE);
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// static BOOL
// MonSelGetMonitorInfo( PMONITORSELWND infoPtr,
//                       INT Index,
//                        PMONSL_MONINFO MonitorsInfo)
// {
//     if (Index >= 0 && Index < (INT)infoPtr->MonitorsCount)
//     {
//         CopyMemory(MonitorsInfo,
//                    &infoPtr->MonitorInfo[Index],
//                    sizeof(MONSL_MONINFO));
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// static INT
// MonSelGetMonitorRect(  PMONITORSELWND infoPtr,
//                       INT Index,
//                       PRECT prc)
// {
//     RECT rc, rcClient;
//
//     if (Index < 0 || (UINT)Index >= infoPtr->MonitorsCount)
//         return -1;
//
//     if (!infoPtr->CanDisplay)
//         return 0;
//
//     MonSelRectToScreen(infoPtr,
//                        &infoPtr->Monitors[Index].rc,
//                        prc);
//
//     rcClient.left = rcClient.top = 0;
//     rcClient.right = infoPtr->ClientSize.cx;
//     rcClient.bottom = infoPtr->ClientSize.cy;
//
//     return IntersectRect(&rc,
//                          &rcClient,
//                          prc) != FALSE;
// }
//
// static BOOL
// MonSelSetCurSelMonitor(  PMONITORSELWND infoPtr,
//                         INT Index,
//                         BOOL bNotify)
// {
//     INT PrevSel;
//     BOOL PreventSelect = FALSE;
//     BOOL Ret = FALSE;
//
//     if (infoPtr->DraggingMonitor < 0 &&
//         (Index == -1 || Index < (INT)infoPtr->MonitorsCount))
//     {
//         if (Index != infoPtr->SelectedMonitor)
//         {
//             if ((infoPtr->MonitorInfo[Index].Flags & MSL_MIF_DISABLED) &&
//                 !(infoPtr->ControlExStyle & MSLM_EX_ALLOWSELECTDISABLED))
//             {
//                 PreventSelect = TRUE;
//             }
//
//             if (!PreventSelect && bNotify)
//             {
//                 MONSL_MONNMMONITORCHANGING nmi;
//
//                 nmi.PreviousSelected = infoPtr->SelectedMonitor;
//                 nmi.AllowChanging = TRUE;
//
//                 MonSelNotifyMonitor(infoPtr,
//                                     MSLN_MONITORCHANGING,
//                                     Index,
//                                     &nmi.hdr);
//
//                 PreventSelect = (nmi.AllowChanging == FALSE);
//             }
//
//             if (!PreventSelect)
//             {
//                 PrevSel = infoPtr->SelectedMonitor;
//                 infoPtr->SelectedMonitor = Index;
//
//                 if (PrevSel >= 0)
//                 {
//                     MonSelRepaintMonitor(infoPtr,
//                                          PrevSel);
//                 }
//
//                 if (infoPtr->SelectedMonitor >= 0)
//                     MonSelRepaintSelected(infoPtr);
//
//                 if (bNotify)
//                 {
//                     MONSL_MONNMHDR nm;
//
//                     MonSelNotifyMonitor(infoPtr,
//                                         MSLN_MONITORCHANGED,
//                                         Index,
//                                         &nm);
//                 }
//             }
//         }
//
//         Ret = TRUE;
//     }
//
//     return Ret;
// }
//
// static VOID
// MonSelCreate(  PMONITORSELWND infoPtr)
// {
//     infoPtr->SelectionFrame.cx = infoPtr->SelectionFrame.cy = 4;
//     infoPtr->Margin.cx = infoPtr->Margin.cy = 20;
//     infoPtr->SelectedMonitor = -1;
//     infoPtr->DraggingMonitor = -1;
//     infoPtr->ControlExStyle = MSLM_EX_ALLOWSELECTDISABLED | MSLM_EX_HIDENUMBERONSINGLE |
//         MSLM_EX_SELECTONRIGHTCLICK | MSLM_EX_SELECTBYARROWKEY;
//     return;
// }
//
// static VOID
// MonSelDestroy(  PMONITORSELWND infoPtr)
// {
//     /* Free all monitors */
//     MonSelSetMonitorsInfo(infoPtr,
//                           0,
//                           NULL);
//
//     if (infoPtr->hbrDisabled != NULL)
//     {
//         DeleteObject(infoPtr->hbrDisabled);
//         infoPtr->hbrDisabled = NULL;
//     }
//
//     if (infoPtr->hbmDisabledPattern != NULL)
//     {
//         DeleteObject(infoPtr->hbmDisabledPattern);
//         infoPtr->hbmDisabledPattern = NULL;
//     }
// }
//
// static BOOL
// MonSelSetExtendedStyle(  PMONITORSELWND infoPtr,
//                         DWORD dwExtendedStyle)
// {
//     if (infoPtr->DraggingMonitor >= 0)
//         return FALSE;
//
//     if (dwExtendedStyle != infoPtr->ControlExStyle)
//     {
//         infoPtr->ControlExStyle = dwExtendedStyle;
//
//         /* Repaint the control */
//         InvalidateRect(infoPtr->hSelf,
//                        NULL,
//                        TRUE);
//     }
//
//     return TRUE;
// }
//
// static DWORD
// MonSelGetExtendedStyle( PMONITORSELWND infoPtr)
// {
//     return infoPtr->ControlExStyle;
// }
//
// static HFONT
// MonSelGetMonitorFont(  PMONITORSELWND infoPtr,
//                       HDC hDC,
//                       INT Index)
// {
//     TEXTMETRIC tm;
//     SIZE rcsize;
//     LOGFONT lf;
//     HFONT hPrevFont, hFont;
//     //INT len;
//
//     hFont = infoPtr->Monitors[Index].hFont;
//     if (hFont == NULL &&
//         GetObject(infoPtr->hFont,
//                   sizeof(lf),
//                   &lf) != 0)
//     {
//         rcsize.cx = infoPtr->Monitors[Index].rc.right - infoPtr->Monitors[Index].rc.left -
//                     (2 * infoPtr->SelectionFrame.cx) - 2;
//         rcsize.cy = infoPtr->Monitors[Index].rc.bottom - infoPtr->Monitors[Index].rc.top -
//                     (2 * infoPtr->SelectionFrame.cy) - 2;
//         rcsize.cy = (rcsize.cy * 60) / 100;
//
//         //len = _tcslen(infoPtr->Monitors[Index].szCaption);
//
//         hPrevFont = SelectObject(hDC,
//                                  infoPtr->hFont);
//
//         if (GetTextMetrics(hDC,
//                            &tm))
//         {
//             lf.lfWeight = FW_SEMIBOLD;
//             lf.lfHeight = -MulDiv(rcsize.cy - tm.tmExternalLeading,
//                                   GetDeviceCaps(hDC,
//                                                 LOGPIXELSY),
//                                   72);
//
//             hFont = CreateFontIndirect(&lf);
//             if (hFont != NULL)
//                 infoPtr->Monitors[Index].hFont = hFont;
//         }
//
//         SelectObject(hDC,
//                      hPrevFont);
//     }
//
//     return hFont;
// }
//
// static BOOL
// MonSelDrawDisabledRect(  PMONITORSELWND infoPtr,
//                         HDC hDC,
//                         const RECT *prc)
// {
//     BOOL Ret = FALSE;
//
//     if (infoPtr->hbrDisabled == NULL)
//     {
//         static const DWORD Pattern[4] = {0x5555AAAA, 0x5555AAAA, 0x5555AAAA, 0x5555AAAA};
//
//         if (infoPtr->hbmDisabledPattern == NULL)
//         {
//             infoPtr->hbmDisabledPattern = CreateBitmap(8,
//                                                        8,
//                                                        1,
//                                                        1,
//                                                        Pattern);
//         }
//
//         if (infoPtr->hbmDisabledPattern != NULL)
//             infoPtr->hbrDisabled = CreatePatternBrush(infoPtr->hbmDisabledPattern);
//     }
//
//     if (infoPtr->hbrDisabled != NULL)
//     {
//         /* FIXME: Implement */
//     }
//
//     return Ret;
// }
//
// static VOID
// MonSelPaintMonitor(  PMONITORSELWND infoPtr,
//                     HDC hDC,
//                     DWORD Index,
//                      PRECT prc,
//                     COLORREF crDefFontColor,
//                     BOOL bHideNumber)
// {
//     HFONT hFont, hPrevFont;
//     COLORREF crPrevText;
//
//     if ((INT)Index == infoPtr->SelectedMonitor)
//     {
//         FillRect(hDC,
//                  prc,
//                  (HBRUSH)(COLOR_HIGHLIGHT + 1));
//
//         if (infoPtr->HasFocus && !(infoPtr->UIState & UISF_HIDEFOCUS))
//         {
//             /* NOTE: We need to switch the text color to the default, because
//                      DrawFocusRect draws a solid line if the text is white! */
//
//             crPrevText = SetTextColor(hDC,
//                                       crDefFontColor);
//
//             DrawFocusRect(hDC,
//                           prc);
//
//             SetTextColor(hDC,
//                          crPrevText);
//         }
//     }
//
//     InflateRect(prc,
//                 -infoPtr->SelectionFrame.cx,
//                 -infoPtr->SelectionFrame.cy);
//
//     Rectangle(hDC,
//               prc->left,
//               prc->top,
//               prc->right,
//               prc->bottom);
//
//     InflateRect(prc,
//                 -1,
//                 -1);
//
//     if (!bHideNumber)
//     {
//         hFont = MonSelGetMonitorFont(infoPtr,
//                                      hDC,
//                                      Index);
//         if (hFont != NULL)
//         {
//             hPrevFont = SelectObject(hDC,
//                                      hFont);
//
//             DrawText(hDC,
//                      infoPtr->Monitors[Index].szCaption,
//                      -1,
//                      prc,
//                      DT_VCENTER | DT_CENTER | DT_NOPREFIX | DT_SINGLELINE);
//
//             SelectObject(hDC,
//                          hPrevFont);
//         }
//     }
//
//     if (infoPtr->MonitorInfo[Index].Flags & MSL_MIF_DISABLED)
//     {
//         InflateRect(prc,
//                     1,
//                     1);
//
//         MonSelDrawDisabledRect(infoPtr,
//                                hDC,
//                                prc);
//     }
// }
//
// static VOID
// MonSelPaint(  PMONITORSELWND infoPtr,
//              HDC hDC,
//              const RECT *prcUpdate)
// {
//     COLORREF crPrevText;
//     HBRUSH hbBk, hbOldBk;
//     HPEN hpFg, hpOldFg;
//     DWORD Index;
//     RECT rc, rctmp;
//     INT iPrevBkMode;
//     BOOL bHideNumber;
//
//     bHideNumber = (infoPtr->ControlExStyle & MSLM_EX_HIDENUMBERS) ||
//         ((infoPtr->MonitorsCount == 1) && (infoPtr->ControlExStyle & MSLM_EX_HIDENUMBERONSINGLE));
//
//     hbBk = GetSysColorBrush(COLOR_BACKGROUND);
//     hpFg = CreatePen(PS_SOLID,
//                      0,
//                      GetSysColor(COLOR_HIGHLIGHTTEXT));
//
//     hbOldBk = SelectObject(hDC,
//                            hbBk);
//     hpOldFg = SelectObject(hDC,
//                            hpFg);
//     iPrevBkMode = SetBkMode(hDC,
//                             TRANSPARENT);
//     crPrevText = SetTextColor(hDC,
//                               GetSysColor(COLOR_HIGHLIGHTTEXT));
//
//     for (Index = 0; Index < infoPtr->MonitorsCount; Index++)
//     {
//         if (infoPtr->IsDraggingMonitor &&
//             (DWORD)infoPtr->DraggingMonitor == Index)
//         {
//             continue;
//         }
//
//         MonSelRectToScreen(infoPtr,
//                            &infoPtr->Monitors[Index].rc,
//                            &rc);
//
//         if (IntersectRect(&rctmp,
//                           &rc,
//                           prcUpdate))
//         {
//             MonSelPaintMonitor(infoPtr,
//                                hDC,
//                                Index,
//                                &rc,
//                                crPrevText,
//                                bHideNumber);
//         }
//     }
//
//     /* Paint the dragging monitor last */
//     if (infoPtr->IsDraggingMonitor &&
//         infoPtr->DraggingMonitor >= 0)
//     {
//         MonSelRectToScreen(infoPtr,
//                            &infoPtr->rcDragging,
//                            &rc);
//
//         if (IntersectRect(&rctmp,
//                           &rc,
//                           prcUpdate))
//         {
//             MonSelPaintMonitor(infoPtr,
//                                hDC,
//                                (DWORD)infoPtr->DraggingMonitor,
//                                &rc,
//                                crPrevText,
//                                bHideNumber);
//         }
//     }
//
//     SetTextColor(hDC,
//                  crPrevText);
//     SetBkMode(hDC,
//               iPrevBkMode);
//     SelectObject(hDC,
//                  hpOldFg);
//     SelectObject(hDC,
//                  hbOldBk);
//
//     DeleteObject(hpFg);
// }
//
// static VOID
// MonSelContextMenu(  PMONITORSELWND infoPtr,
//                    SHORT x,
//                    SHORT y)
// {
//     MONSL_MONNMBUTTONCLICKED nm;
//     INT Index;
//
//     if (!infoPtr->HasFocus)
//         SetFocus(infoPtr->hSelf);
//
//     nm.pt.x = x;
//     nm.pt.y = y;
//
//     Index = MonSelHitTest(infoPtr,
//                           &nm.pt);
//
//     MonSelNotifyMonitor(infoPtr,
//                         MSLN_RBUTTONUP,
//                         Index,
//                         (PMONSL_MONNMHDR)&nm);
//
//     /* Send a WM_CONTEXTMENU notification */
//     MapWindowPoints(infoPtr->hSelf,
//                     NULL,
//                     &nm.pt,
//                     1);
//
//     SendMessage(infoPtr->hSelf,
//                 WM_CONTEXTMENU,
//                 (WPARAM)infoPtr->hSelf,
//                 MAKELPARAM(nm.pt.x,
//                            nm.pt.y));
// }
//
// static VOID
// MonSelApplyCursorClipping( PMONITORSELWND infoPtr,
//                            BOOL bClip)
// {
//     RECT rc;
//
//     if (bClip)
//     {
//         rc.left = rc.top = 0;
//         rc.right = infoPtr->ClientSize.cx;
//         rc.bottom = infoPtr->ClientSize.cy;
//
//         if (MapWindowPoints(infoPtr->hSelf,
//                             NULL,
//                             (LPPOINT)&rc,
//                             2))
//         {
//             ClipCursor(&rc);
//         }
//     }
//     else
//     {
//         ClipCursor(NULL);
//     }
// }
//
// static VOID
// MonSelMoveDragRect(  PMONITORSELWND infoPtr,
//                     PPOINT ppt)
// {
//     RECT rcPrev, rcUpdate, *prc;
//     HRGN hRgnPrev;
//     HDC hDC;
//
//     if (infoPtr->CanDisplay)
//     {
//         hDC = GetDC(infoPtr->hSelf);
//         if (hDC != NULL)
//         {
//             if (infoPtr->ptDrag.x != ppt->x ||
//                 infoPtr->ptDrag.y != ppt->y)
//             {
//                 infoPtr->ptDrag = *ppt;
//
//                 rcPrev = infoPtr->rcDragging;
//
//                 /* Calculate updated dragging rectangle */
//                 prc = &infoPtr->Monitors[infoPtr->DraggingMonitor].rc;
//                 infoPtr->rcDragging.left = ppt->x - infoPtr->DraggingMargin.cx;
//                 infoPtr->rcDragging.top = ppt->y - infoPtr->DraggingMargin.cy;
//                 infoPtr->rcDragging.right = infoPtr->rcDragging.left + (prc->right - prc->left);
//                 infoPtr->rcDragging.bottom = infoPtr->rcDragging.top + (prc->bottom - prc->top);
//
//                 hRgnPrev = CreateRectRgn(rcPrev.left,
//                                          rcPrev.top,
//                                          rcPrev.right,
//                                          rcPrev.bottom);
//
//                 if (hRgnPrev != NULL)
//                 {
//                     if (!ScrollDC(hDC,
//                                   infoPtr->rcDragging.left - rcPrev.left,
//                                   infoPtr->rcDragging.top - rcPrev.top,
//                                   &rcPrev,
//                                   NULL,
//                                   hRgnPrev,
//                                   &rcUpdate) ||
//                         !InvalidateRgn(infoPtr->hSelf,
//                                        hRgnPrev,
//                                        TRUE))
//                     {
//                         DeleteObject(hRgnPrev);
//                         goto InvRects;
//                     }
//
//                     DeleteObject(hRgnPrev);
//                 }
//                 else
//                 {
// InvRects:
//                     InvalidateRect(infoPtr->hSelf,
//                                    &rcPrev,
//                                    TRUE);
//                     InvalidateRect(infoPtr->hSelf,
//                                    &infoPtr->rcDragging,
//                                    TRUE);
//                 }
//             }
//
//             ReleaseDC(infoPtr->hSelf,
//                       hDC);
//         }
//     }
// }
//
// static VOID
// MonSelCancelDragging(  PMONITORSELWND infoPtr)
// {
//     DWORD Index;
//
//     if (infoPtr->DraggingMonitor >= 0)
//     {
//         MonSelMoveDragRect(infoPtr,
//                            &infoPtr->ptDragBegin);
//
//         Index = (DWORD)infoPtr->DraggingMonitor;
//         infoPtr->DraggingMonitor = -1;
//
//         if (infoPtr->CanDisplay)
//         {
//             /* Repaint the area where the monitor was last dragged */
//             MonSelRepaintMonitor(infoPtr,
//                                  Index);
//
//             infoPtr->IsDraggingMonitor = FALSE;
//
//             /* Repaint the area where the monitor is located */
//             MonSelRepaintMonitor(infoPtr,
//                                  Index);
//         }
//         else
//             infoPtr->IsDraggingMonitor = FALSE;
//
//         ReleaseCapture();
//
//         MonSelApplyCursorClipping(infoPtr,
//                                   FALSE);
//     }
// }
//
// static VOID
// MonSelInitDragging(  PMONITORSELWND infoPtr,
//                     DWORD Index,
//                     PPOINT ppt)
// {
//     POINT pt;
//
//     MonSelCancelDragging(infoPtr);
//     infoPtr->IsDraggingMonitor = FALSE;
//
//     MonSelScreenToPt(infoPtr,
//                      ppt,
//                      &pt);
//
//     infoPtr->ptDrag = infoPtr->ptDragBegin = pt;
//     infoPtr->DraggingMonitor = (INT)Index;
//
//     infoPtr->DraggingMargin.cx = ppt->x - infoPtr->Monitors[Index].rc.left;
//     infoPtr->DraggingMargin.cy = ppt->y - infoPtr->Monitors[Index].rc.top;
//     infoPtr->rcDragging = infoPtr->Monitors[Index].rc;
//
//     MonSelApplyCursorClipping(infoPtr,
//                               TRUE);
// }
//
// static VOID
// MonSelDrag(  PMONITORSELWND infoPtr,
//             PPOINT ppt)
// {
//     SIZE szDrag;
//     POINT pt;
//     RECT rcDrag;
//
//     if (infoPtr->DraggingMonitor >= 0)
//     {
//         MonSelScreenToPt(infoPtr,
//                          ppt,
//                          &pt);
//
//         if (!infoPtr->IsDraggingMonitor)
//         {
//             szDrag.cx = GetSystemMetrics(SM_CXDRAG);
//             szDrag.cy = GetSystemMetrics(SM_CYDRAG);
//
//             rcDrag.left = infoPtr->Monitors[infoPtr->DraggingMonitor].rc.left + infoPtr->DraggingMargin.cx - (szDrag.cx / 2);
//             rcDrag.top = infoPtr->Monitors[infoPtr->DraggingMonitor].rc.top + infoPtr->DraggingMargin.cy - (szDrag.cy / 2);
//             rcDrag.right = rcDrag.left + szDrag.cx;
//             rcDrag.bottom = rcDrag.top + szDrag.cy;
//
//             if (!PtInRect(&rcDrag,
//                           pt))
//             {
//                 /* The user started moving around the mouse: Begin dragging */
//                 infoPtr->IsDraggingMonitor = TRUE;
//                 MonSelMoveDragRect(infoPtr,
//                                    &pt);
//             }
//         }
//         else
//         {
//             MonSelMoveDragRect(infoPtr,
//                                &pt);
//         }
//     }
// }
//
// static LRESULT __stdcall
// MonitorSelWndProc( HWND hwnd,
//                    UINT uMsg,
//                    WPARAM wParam,
//                    LPARAM lParam)
// {
//     PMONITORSELWND infoPtr;
//     LRESULT Ret = 0;
//
//     infoPtr = (PMONITORSELWND)GetWindowLongPtrW(hwnd,
//                                                 0);
//
//     if (infoPtr == NULL && uMsg != WM_CREATE)
//     {
//         goto HandleDefaultMessage;
//     }
//
//     switch (uMsg)
//     {
//         case WM_PAINT:
//         case WM_PRINTCLIENT:
//         {
//             PAINTSTRUCT ps;
//             HDC hDC;
//
//             if (wParam != 0)
//             {
//                 if (!GetUpdateRect(hwnd,
//                                    &ps.rcPaint,
//                                    TRUE))
//                 {
//                     break;
//                 }
//                 hDC = (HDC)wParam;
//             }
//             else
//             {
//                 hDC = BeginPaint(hwnd,
//                                  &ps);
//                 if (hDC == NULL)
//                 {
//                     break;
//                 }
//             }
//
//             if (infoPtr->CanDisplay)
//             {
//                 MonSelPaint(infoPtr,
//                             hDC,
//                             &ps.rcPaint);
//             }
//
//             if (wParam == 0)
//             {
//                 EndPaint(hwnd,
//                          &ps);
//             }
//             break;
//         }
//
//         case WM_MOUSEMOVE:
//         {
//             POINT pt;
//
//             if (!(wParam & MK_LBUTTON))
//             {
//                 MonSelCancelDragging(infoPtr);
//                 break;
//             }
//
//             if (infoPtr->LeftBtnDown)
//             {
//                 pt.x = (LONG)LOWORD(lParam);
//                 pt.y = (LONG)HIWORD(lParam);
//
//                 MonSelDrag(infoPtr,
//                            &pt);
//             }
//
//             break;
//         }
//
//         case WM_RBUTTONDOWN:
//         {
//             if (!(infoPtr->ControlExStyle & MSLM_EX_SELECTONRIGHTCLICK))
//                 break;
//
//             /* Fall through */
//         }
//
//         case WM_LBUTTONDBLCLK:
//         case WM_LBUTTONDOWN:
//         {
//             INT Index;
//             POINT pt;
//
//             if (!infoPtr->HasFocus)
//                 SetFocus(infoPtr->hSelf);
//
//             pt.x = (LONG)LOWORD(lParam);
//             pt.y = (LONG)HIWORD(lParam);
//
//             Index = MonSelHitTest(infoPtr,
//                                   &pt);
//             if (Index >= 0 || (infoPtr->ControlExStyle & MSLM_EX_ALLOWSELECTNONE))
//             {
//                 MonSelSetCurSelMonitor(infoPtr,
//                                        Index,
//                                        TRUE);
//             }
//
//             if (Index >= 0 && (uMsg == WM_LBUTTONDOWN || uMsg == WM_LBUTTONDBLCLK))
//             {
//                 infoPtr->LeftBtnDown = TRUE;
//                 MonSelInitDragging(infoPtr,
//                                    (DWORD)Index,
//                                    &pt);
//             }
//
//             /* Fall through */
//         }
//
//         case WM_MBUTTONDOWN:
//         {
//             if (!infoPtr->HasFocus)
//                 SetFocus(hwnd);
//             break;
//         }
//
//         case WM_RBUTTONUP:
//         {
//             MonSelContextMenu(infoPtr,
//                               (SHORT)LOWORD(lParam),
//                               (SHORT)HIWORD(lParam));
//             break;
//         }
//
//         case WM_LBUTTONUP:
//         {
//             MonSelCancelDragging(infoPtr);
//             infoPtr->LeftBtnDown = FALSE;
//             break;
//         }
//
//         case WM_GETDLGCODE:
//         {
//             INT virtKey;
//
//             virtKey = (lParam != 0 ? (INT)((LPMSG)lParam)->wParam : 0);
//             switch (virtKey)
//             {
//                 case VK_TAB:
//                 {
//                     /* Change the UI status */
//                     SendMessage(GetAncestor(hwnd,
//                                             GA_PARENT),
//                                 WM_CHANGEUISTATE,
//                                 MAKEWPARAM(UIS_INITIALIZE,
//                                            0),
//                                 0);
//                     break;
//                 }
//             }
//
//             Ret |= DLGC_WANTARROWS;
//
//             if (infoPtr->ControlExStyle & MSLM_EX_SELECTBYNUMKEY)
//                 Ret |= DLGC_WANTCHARS;
//             break;
//         }
//
//         case WM_SETFOCUS:
//         {
//             infoPtr->HasFocus = TRUE;
//             MonSelRepaintSelected(infoPtr);
//             break;
//         }
//
//         case WM_KILLFOCUS:
//         {
//             infoPtr->HasFocus = FALSE;
//             MonSelCancelDragging(infoPtr);
//             MonSelRepaintSelected(infoPtr);
//             break;
//         }
//
//         case WM_UPDATEUISTATE:
//         {
//             DWORD OldUIState;
//
//             Ret = DefWindowProcW(hwnd,
//                                  uMsg,
//                                  wParam,
//                                  lParam);
//
//             OldUIState = infoPtr->UIState;
//             switch (LOWORD(wParam))
//             {
//                 case UIS_SET:
//                     infoPtr->UIState |= HIWORD(wParam);
//                     break;
//
//                 case UIS_CLEAR:
//                     infoPtr->UIState &= ~HIWORD(wParam);
//                     break;
//             }
//
//             if (infoPtr->UIState != OldUIState)
//                 MonSelRepaintSelected(infoPtr);
//             break;
//         }
//
//         case WM_SETFONT:
//         {
//             Ret = (LRESULT)MonSelChangeFont(infoPtr,
//                                             (HFONT)wParam,
//                                             (BOOL)LOWORD(lParam));
//             break;
//         }
//
//         case WM_SIZE:
//         {
//             infoPtr->ClientSize.cx = LOWORD(lParam);
//             infoPtr->ClientSize.cy = HIWORD(lParam);
//
//             /* Don't let MonSelUpdateMonitorsInfo repaint the control
//                because this won't work properly in case the control
//                was sized down! */
//             MonSelUpdateMonitorsInfo(infoPtr,
//                                      FALSE);
//             InvalidateRect(infoPtr->hSelf,
//                            NULL,
//                            TRUE);
//             break;
//         }
//
//         case WM_GETFONT:
//         {
//             Ret = (LRESULT)infoPtr->hFont;
//             break;
//         }
//
//         case WM_ENABLE:
//         {
//             infoPtr->Enabled = ((BOOL)wParam != FALSE);
//             MonSelRepaint(infoPtr);
//             break;
//         }
//
//         case WM_STYLECHANGED:
//         {
//             if (wParam == GWL_STYLE)
//             {
//                 unsigned int OldEnabled = infoPtr->Enabled;
//                 infoPtr->Enabled = !(((LPSTYLESTRUCT)lParam)->styleNew & WS_DISABLED);
//
//                 if (OldEnabled != infoPtr->Enabled)
//                     MonSelRepaint(infoPtr);
//             }
//             break;
//         }
//
//         case WM_KEYDOWN:
//         {
//             INT Index;
//
//             if (infoPtr->ControlExStyle & MSLM_EX_SELECTBYARROWKEY)
//             {
//                 switch (wParam)
//                 {
//                     case VK_UP:
//                     case VK_LEFT:
//                     {
//                         Index = infoPtr->SelectedMonitor;
//
//                         if (infoPtr->MonitorsCount != 0)
//                         {
//                             if (Index < 0)
//                                 Index = 0;
//                             else if (Index > 0)
//                                 Index--;
//                         }
//
//                         if (Index >= 0)
//                         {
//                             MonSelSetCurSelMonitor(infoPtr,
//                                                    Index,
//                                                    TRUE);
//                         }
//                         break;
//                     }
//
//                     case VK_DOWN:
//                     case VK_RIGHT:
//                     {
//                         Index = infoPtr->SelectedMonitor;
//
//                         if (infoPtr->MonitorsCount != 0)
//                         {
//                             if (Index < 0)
//                                 Index = (INT)infoPtr->MonitorsCount - 1;
//                             else if (Index < (INT)infoPtr->MonitorsCount - 1)
//                                 Index++;
//                         }
//
//                         if (infoPtr->SelectedMonitor < (INT)infoPtr->MonitorsCount)
//                         {
//                             MonSelSetCurSelMonitor(infoPtr,
//                                                    Index,
//                                                    TRUE);
//                         }
//                         break;
//                     }
//                 }
//             }
//             break;
//         }
//
//         case WM_CHAR:
//         {
//             if ((infoPtr->ControlExStyle & MSLM_EX_SELECTBYNUMKEY) &&
//                 wParam >= '1' && wParam <= '9')
//             {
//                 INT Index = (INT)(wParam - '1');
//                 if (Index < (INT)infoPtr->MonitorsCount)
//                 {
//                     MonSelSetCurSelMonitor(infoPtr,
//                                            Index,
//                                            TRUE);
//                 }
//             }
//             break;
//         }
//
//         case MSLM_SETMONITORSINFO:
//         {
//             Ret = MonSelSetMonitorsInfo(infoPtr,
//                                         (DWORD)wParam,
//                                         (const MONSL_MONINFO *)lParam);
//             break;
//         }
//
//         case MSLM_GETMONITORSINFO:
//         {
//             Ret = MonSelGetMonitorsInfo(infoPtr,
//                                         (DWORD)wParam,
//                                         (PMONSL_MONINFO)lParam);
//             break;
//         }
//
//         case MSLM_GETMONITORINFOCOUNT:
//         {
//             Ret = infoPtr->MonitorsCount;
//             break;
//         }
//
//         case MSLM_HITTEST:
//         {
//             Ret = MonSelHitTest(infoPtr,
//                                 (const POINT *)wParam);
//             break;
//         }
//
//         case MSLM_SETCURSEL:
//         {
//             Ret = MonSelSetCurSelMonitor(infoPtr,
//                                          (INT)wParam,
//                                          FALSE);
//             break;
//         }
//
//         case MSLM_GETCURSEL:
//         {
//             Ret = infoPtr->SelectedMonitor;
//             break;
//         }
//
//         case MSLM_SETMONITORINFO:
//         {
//             Ret = MonSelSetMonitorInfo(infoPtr,
//                                        (INT)wParam,
//                                        (const MONSL_MONINFO *)lParam);
//             break;
//         }
//
//         case MSLM_GETMONITORINFO:
//         {
//             Ret = MonSelGetMonitorInfo(infoPtr,
//                                        (INT)wParam,
//                                        (PMONSL_MONINFO)lParam);
//             break;
//         }
//
//         case MSLM_SETEXSTYLE:
//         {
//             Ret = MonSelSetExtendedStyle(infoPtr,
//                                          (DWORD)lParam);
//             break;
//         }
//
//         case MSLM_GETEXSTYLE:
//         {
//             Ret = MonSelGetExtendedStyle(infoPtr);
//             break;
//         }
//
//         case MSLM_GETMONITORRECT:
//         {
//             Ret = (LRESULT)MonSelGetMonitorRect(infoPtr,
//                                                 (INT)wParam,
//                                                 (PRECT)lParam);
//             break;
//         }
//
//         case WM_CREATE:
//         {
//             infoPtr = (PMONITORSELWND)HeapAlloc(GetProcessHeap(),
//                                                 0,
//                                                 sizeof(MONITORSELWND));
//             if (infoPtr == NULL)
//             {
//                 Ret = (LRESULT)-1;
//                 break;
//             }
//
//             ZeroMemory(infoPtr,
//                        sizeof(MONITORSELWND));
//             infoPtr->hSelf = hwnd;
//             infoPtr->hNotify = ((LPCREATESTRUCTW)lParam)->hwndParent;
//             infoPtr->Enabled = !(((LPCREATESTRUCTW)lParam)->style & WS_DISABLED);
//             infoPtr->UIState = SendMessage(hwnd,
//                                            WM_QUERYUISTATE,
//                                            0,
//                                            0);
//
//             SetWindowLongPtrW(hwnd,
//                               0,
//                               (LONG_PTR)infoPtr);
//
//             MonSelCreate(infoPtr);
//             break;
//         }
//
//         case WM_DESTROY:
//         {
//             MonSelDestroy(infoPtr);
//
//             HeapFree(GetProcessHeap(),
//                      0,
//                      infoPtr);
//             SetWindowLongPtrW(hwnd,
//                               0,
//                               (DWORD_PTR)NULL);
//             break;
//         }
//
//         default:
//         {
// HandleDefaultMessage:
//             Ret = DefWindowProcW(hwnd,
//                                  uMsg,
//                                  wParam,
//                                  lParam);
//             break;
//         }
//     }
//
//     return Ret;
// }
//
// BOOL
// RegisterMonitorSelectionControl( HINSTANCE hInstance)
// {
//     WNDCLASS wc = {0};
//
//     wc.style = CS_DBLCLKS;
//     wc.lpfnWndProc = MonitorSelWndProc;
//     wc.cbWndExtra = sizeof(PMONITORSELWND);
//     wc.hInstance = hInstance;
//     wc.hCursor = LoadCursorW(NULL,
//                              (LPWSTR)IDC_ARROW);
//     wc.hbrBackground = (HBRUSH)(COLOR_APPWORKSPACE + 1);
//     wc.lpszClassName = szMonitorSelWndClass;
//
//     return RegisterClass(&wc) != 0;
// }
//
// VOID
// UnregisterMonitorSelectionControl( HINSTANCE hInstance)
// {
//     UnregisterClassW(szMonitorSelWndClass,
//                      hInstance);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
