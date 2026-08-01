//! `sdk/include/reactos/ui/layout.h` — the resizable-dialog layout engine, ported.
//!
//! Every function in that header is `static __inline`, so it lives in NO translation unit and
//! rustconv never sees it: a translated TU that uses the engine comes out referencing
//! `LayoutInit`/`LayoutUpdate`/`LayoutDestroy` with nothing to link against. Ported once here
//! rather than per-TU, the same as `crate::shellutils` does for `shellutils.h`/`utils.h`.
//!
//! NOT `#[no_mangle]`: a `static __inline` at namespace scope has INTERNAL linkage, so every C
//! TU that uses the engine already carries its own copy. Exporting these would invent symbols
//! the C side never had.
//!
//! The header's `assert()` calls are deliberately not translated. They compile out of a release
//! build, and a Rust `assert!` would put a PANIC into a shipping DLL on a path C merely ignores
//! — `LayoutInit` already returns NULL on the allocation failure it asserts about.

use crate::island::{
    BeginDeferWindowPos, CreateWindowExW, DeferWindowPos, EndDeferWindowPos, EqualRect,
    GetClientRect, GetDlgItem, GetModuleHandleW, GetProcessHeap, GetSystemMenu, GetSystemMetrics,
    GetWindowRect, HeapAlloc, HeapFree, IsWindowVisible, MapWindowPoints, RemoveMenu, SetWindowPos,
    ShowWindow, BF_BOTTOM, BF_LEFT, BF_RIGHT, BF_TOP, BOOL, DWORD, FALSE, GWL_STYLE, HDWP, HWND,
    LAYOUT_DATA, LAYOUT_INFO, LONG, LPRECT, MF_BYCOMMAND, POINT, RECT, SBS_SIZEGRIP, SC_MAXIMIZE,
    SC_RESTORE, SC_SIZE, SIZE, SIZE_T, SM_CXVSCROLL, SM_CYHSCROLL, SWP_FRAMECHANGED,
    SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOMOVE, SWP_NOOWNERZORDER, SWP_NOREPOSITION, SWP_NOSIZE,
    SWP_NOZORDER, SW_HIDE, SW_SHOWNOACTIVATE, TRUE, UINT, WS_CHILD, WS_CLIPSIBLINGS, WS_GROUP,
    WS_SIZEBOX,
};
use crate::prelude::{GetWindowLongPtrW, SetWindowLongPtrW};

/// `L"SCROLLBAR"` — the grip is a size-grip scrollbar child.
static SCROLLBAR_CLASS: [u16; 10] = [
    0x0053, 0x0043, 0x0052, 0x004F, 0x004C, 0x004C, 0x0042, 0x0041, 0x0052, 0x0000,
];

unsafe fn _layout_ModifySystemMenu(pData: *mut LAYOUT_DATA, bEnableResize: BOOL) {
    if bEnableResize != 0 {
        GetSystemMenu((*pData).m_hwndParent, TRUE as BOOL); // revert
    } else {
        let hSysMenu = GetSystemMenu((*pData).m_hwndParent, FALSE as BOOL);
        RemoveMenu(hSysMenu, SC_MAXIMIZE, MF_BYCOMMAND);
        RemoveMenu(hSysMenu, SC_SIZE, MF_BYCOMMAND);
        RemoveMenu(hSysMenu, SC_RESTORE, MF_BYCOMMAND);
    }
}

unsafe fn _layout_MoveGrip(pData: *mut LAYOUT_DATA, hDwp: HDWP) -> HDWP {
    if IsWindowVisible((*pData).m_hwndGrip) == 0 {
        return hDwp;
    }

    let size = SIZE {
        cx: GetSystemMetrics(SM_CXVSCROLL as i32),
        cy: GetSystemMetrics(SM_CYHSCROLL as i32),
    };
    let uFlags: UINT = SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOZORDER | SWP_NOCOPYBITS;
    let mut rcClient: RECT = core::mem::zeroed();
    GetClientRect((*pData).m_hwndParent, &mut rcClient);

    if !hDwp.is_null() {
        DeferWindowPos(
            hDwp,
            (*pData).m_hwndGrip,
            core::ptr::null_mut(),
            rcClient.right - size.cx,
            rcClient.bottom - size.cy,
            size.cx,
            size.cy,
            uFlags,
        )
    } else {
        SetWindowPos(
            (*pData).m_hwndGrip,
            core::ptr::null_mut(),
            rcClient.right - size.cx,
            rcClient.bottom - size.cy,
            size.cx,
            size.cy,
            uFlags,
        );
        hDwp
    }
}

pub unsafe fn LayoutShowGrip(pData: *mut LAYOUT_DATA, bShow: BOOL) {
    let uSWP: UINT = SWP_NOMOVE
        | SWP_NOSIZE
        | SWP_NOACTIVATE
        | SWP_NOZORDER
        | SWP_NOOWNERZORDER
        | SWP_FRAMECHANGED;
    let style = GetWindowLongPtrW((*pData).m_hwndParent, GWL_STYLE as i32) as DWORD;
    let new_style = if bShow != 0 { style | WS_SIZEBOX } else { style & !WS_SIZEBOX };
    if style != new_style {
        SetWindowLongPtrW((*pData).m_hwndParent, GWL_STYLE as i32, new_style as isize as _);
        SetWindowPos((*pData).m_hwndParent, core::ptr::null_mut(), 0, 0, 0, 0, uSWP);
    }

    if bShow == 0 {
        ShowWindow((*pData).m_hwndGrip, SW_HIDE as i32);
        return;
    }

    if (*pData).m_hwndGrip.is_null() {
        // CORE-19585: WS_GROUP set to avoid navigation over the grip control
        let style = WS_GROUP | WS_CHILD | WS_CLIPSIBLINGS | SBS_SIZEGRIP;
        (*pData).m_hwndGrip = CreateWindowExW(
            0,
            SCROLLBAR_CLASS.as_ptr(),
            core::ptr::null(),
            style,
            0,
            0,
            0,
            0,
            (*pData).m_hwndParent,
            core::ptr::null_mut(),
            GetModuleHandleW(core::ptr::null()),
            core::ptr::null_mut(),
        );
    }
    _layout_MoveGrip(pData, core::ptr::null_mut());
    ShowWindow((*pData).m_hwndGrip, SW_SHOWNOACTIVATE as i32);
}

unsafe fn _layout_GetPercents(prcPercents: LPRECT, uEdges: UINT) {
    (*prcPercents).left = if (uEdges & BF_LEFT) != 0 { 0 } else { 100 };
    (*prcPercents).right = if (uEdges & BF_RIGHT) != 0 { 100 } else { 0 };
    (*prcPercents).top = if (uEdges & BF_TOP) != 0 { 0 } else { 100 };
    (*prcPercents).bottom = if (uEdges & BF_BOTTOM) != 0 { 100 } else { 0 };
}

unsafe fn _layout_DoMoveItem(
    pData: *mut LAYOUT_DATA,
    hDwp: HDWP,
    pLayout: *const LAYOUT_INFO,
    rcClient: *const RECT,
) -> HDWP {
    let mut rcChild: RECT = core::mem::zeroed();
    let mut NewRect: RECT = core::mem::zeroed();
    let mut rcPercents: RECT = core::mem::zeroed();

    if GetWindowRect((*pLayout).m_hwndCtrl, &mut rcChild) == 0 {
        return hDwp;
    }
    // A RECT is two POINTs; the C cast is what lets one call map both corners.
    MapWindowPoints(
        core::ptr::null_mut(),
        (*pData).m_hwndParent,
        core::ptr::addr_of_mut!(rcChild) as *mut POINT,
        2,
    );

    let nWidth: LONG = (*rcClient).right - (*rcClient).left;
    let nHeight: LONG = (*rcClient).bottom - (*rcClient).top;

    _layout_GetPercents(&mut rcPercents, (*pLayout).m_uEdges);
    NewRect.left = (*pLayout).m_margin1.cx + nWidth * rcPercents.left / 100;
    NewRect.top = (*pLayout).m_margin1.cy + nHeight * rcPercents.top / 100;
    NewRect.right = (*pLayout).m_margin2.cx + nWidth * rcPercents.right / 100;
    NewRect.bottom = (*pLayout).m_margin2.cy + nHeight * rcPercents.bottom / 100;

    if EqualRect(&NewRect, &rcChild) == 0 {
        return DeferWindowPos(
            hDwp,
            (*pLayout).m_hwndCtrl,
            core::ptr::null_mut(),
            NewRect.left,
            NewRect.top,
            NewRect.right - NewRect.left,
            NewRect.bottom - NewRect.top,
            SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOREPOSITION | SWP_NOCOPYBITS,
        );
    }
    hDwp
}

unsafe fn _layout_ArrangeLayout(pData: *mut LAYOUT_DATA) {
    let mut hDwp = BeginDeferWindowPos(((*pData).m_cLayouts + 1) as i32);
    if hDwp.is_null() {
        return;
    }

    let mut rcClient: RECT = core::mem::zeroed();
    GetClientRect((*pData).m_hwndParent, &mut rcClient);

    let mut iItem: UINT = 0;
    while iItem < (*pData).m_cLayouts {
        hDwp = _layout_DoMoveItem(pData, hDwp, (*pData).m_pLayouts.add(iItem as usize), &rcClient);
        iItem += 1;
    }

    hDwp = _layout_MoveGrip(pData, hDwp);
    EndDeferWindowPos(hDwp);
}

unsafe fn _layout_InitLayouts(pData: *mut LAYOUT_DATA) {
    let mut rcClient: RECT = core::mem::zeroed();
    let mut rcChild: RECT = core::mem::zeroed();
    let mut rcPercents: RECT = core::mem::zeroed();

    GetClientRect((*pData).m_hwndParent, &mut rcClient);
    let nWidth: LONG = rcClient.right - rcClient.left;
    let nHeight: LONG = rcClient.bottom - rcClient.top;

    let mut iItem: UINT = 0;
    while iItem < (*pData).m_cLayouts {
        let pInfo = (*pData).m_pLayouts.add(iItem as usize);
        iItem += 1;
        if (*pInfo).m_hwndCtrl.is_null() {
            (*pInfo).m_hwndCtrl = GetDlgItem((*pData).m_hwndParent, (*pInfo).m_nCtrlID as i32);
            if (*pInfo).m_hwndCtrl.is_null() {
                continue;
            }
        }

        GetWindowRect((*pInfo).m_hwndCtrl, &mut rcChild);
        MapWindowPoints(
            core::ptr::null_mut(),
            (*pData).m_hwndParent,
            core::ptr::addr_of_mut!(rcChild) as *mut POINT,
            2,
        );

        _layout_GetPercents(&mut rcPercents, (*pInfo).m_uEdges);
        (*pInfo).m_margin1.cx = rcChild.left - nWidth * rcPercents.left / 100;
        (*pInfo).m_margin1.cy = rcChild.top - nHeight * rcPercents.top / 100;
        (*pInfo).m_margin2.cx = rcChild.right - nWidth * rcPercents.right / 100;
        (*pInfo).m_margin2.cy = rcChild.bottom - nHeight * rcPercents.bottom / 100;
    }
}

/// Call on the parent's `WM_SIZE`. The three ignored parameters are the header's own shape.
pub unsafe fn LayoutUpdate(
    _ignored1: HWND,
    pData: *mut LAYOUT_DATA,
    _ignored2: *const core::ffi::c_void,
    _ignored3: UINT,
) {
    if pData.is_null() || (*pData).m_hwndParent.is_null() {
        return;
    }
    _layout_ArrangeLayout(pData);
}

pub unsafe fn LayoutEnableResize(pData: *mut LAYOUT_DATA, bEnable: BOOL) {
    LayoutShowGrip(pData, bEnable);
    _layout_ModifySystemMenu(pData, bEnable);
}

pub unsafe fn LayoutInit(
    hwndParent: HWND,
    pLayouts: *const LAYOUT_INFO,
    cLayouts: i32,
) -> *mut LAYOUT_DATA {
    let pData = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<LAYOUT_DATA>() as SIZE_T,
    ) as *mut LAYOUT_DATA;
    if pData.is_null() {
        return core::ptr::null_mut();
    }

    // A NEGATIVE count means "this many layouts, and no size grip".
    let (cLayouts, bShowGrip): (i32, BOOL) =
        if cLayouts < 0 { (-cLayouts, FALSE as BOOL) } else { (cLayouts, TRUE as BOOL) };

    let cb = (cLayouts as usize) * core::mem::size_of::<LAYOUT_INFO>();
    (*pData).m_cLayouts = cLayouts as UINT;
    (*pData).m_pLayouts = HeapAlloc(GetProcessHeap(), 0, cb as SIZE_T) as *mut LAYOUT_INFO;
    if (*pData).m_pLayouts.is_null() {
        HeapFree(GetProcessHeap(), 0, pData.cast());
        return core::ptr::null_mut();
    }
    core::ptr::copy_nonoverlapping(pLayouts, (*pData).m_pLayouts, cLayouts as usize);

    (*pData).m_hwndParent = hwndParent;

    (*pData).m_hwndGrip = core::ptr::null_mut();
    if bShowGrip != 0 {
        LayoutShowGrip(pData, bShowGrip);
    }

    _layout_InitLayouts(pData);
    pData
}

pub unsafe fn LayoutDestroy(pData: *mut LAYOUT_DATA) {
    if pData.is_null() {
        return;
    }
    HeapFree(GetProcessHeap(), 0, (*pData).m_pLayouts.cast());
    HeapFree(GetProcessHeap(), 0, pData.cast());
}
