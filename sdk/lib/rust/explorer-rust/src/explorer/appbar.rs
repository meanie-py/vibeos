//! ORPHAN-BY-DESIGN: appbar.cpp was replaced by Rust folded into traywnd.rs —
//! CAppBarManager is a base class of CTrayWindow and was translated with it, so
//! the live code spells these as CTrayWindow methods rather than the 29
//! CAppBarManager_* free functions here. See base/shell/explorer/CMakeLists.txt.
//!
use core::ffi::c_void;
use explorer_sys::*;
use explorer_sys::com::*;
use explorer_sys::externs::*;
use explorer_sys::globals::*;
use explorer_sys::types::*;
use winsrv_sys::consts::*;
use winsrv_sys::w;
use winsrv_sys::winsrv::*;

extern "C" {
    fn DPA_GetPtrCount(hdpa: HDPA) -> i32;
    fn TRACE(fmt: *const i8, ...);
    fn AppBar_LockOutput(pData: *mut APPBAR_COMMAND) -> *mut APPBARDATAINTEROP;
    fn AppBar_UnLockOutput(pOutput: *mut APPBARDATAINTEROP);
    fn ERR(fmt: *const i8, ...);
}
unsafe extern "system" {
    fn DPA_GetPtr(hdpa: HDPA, i: i32) -> *mut core::ffi::c_void;
    fn DPA_Destroy(hdpa: HDPA) -> i32;
}
extern "system" {
    fn DPA_DeletePtr(hdpa: HDPA, i: i32) -> *mut core::ffi::c_void;
    fn RedrawWindow(hWnd: HWND, lprcUpdate: *const RECT, hrgnUpdate: *mut core::ffi::c_void, flags: u32) -> i32;
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_ctor(this: *mut CAppBarManager) {
    (*this).m_hAppBarDPA = core::ptr::null_mut();
    core::ptr::write_bytes(
        (*this).m_ahwndAutoHideBars.as_mut_ptr(),
        0,
        4
    );
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_dtor(this: *mut CAppBarManager) {
    CAppBarManager_DestroyAppBarDPA(this);
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_FindAppBar(
    this: *const CAppBarManager,
    hwndAppBar: HWND,
) -> PAPPBAR {
    if (*this).m_hAppBarDPA.is_null() {
        return core::ptr::null_mut();
    }

    let mut nItems = DPA_GetPtrCount((*this).m_hAppBarDPA);
    loop {
        nItems -= 1;
        if nItems < 0 {
            break;
        }

        let pAppBar = DPA_GetPtr((*this).m_hAppBarDPA, nItems) as PAPPBAR;
        if !pAppBar.is_null() && hwndAppBar == (*pAppBar).hWnd {
            return pAppBar;
        }
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_EliminateAppBar(
    this: *mut CAppBarManager,
    iItem: i32,
) {
    
    LocalFree(DPA_GetPtr((*this).m_hAppBarDPA, iItem));
    DPA_DeletePtr((*this).m_hAppBarDPA, iItem);
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_DestroyAppBarDPA(this: *mut CAppBarManager) {
    if (*this).m_hAppBarDPA.is_null() {
        return;
    }

    let mut nItems = DPA_GetPtrCount((*this).m_hAppBarDPA);
    loop {
        nItems -= 1;
        if nItems < 0 {
            break;
        }
        LocalFree(DPA_GetPtr((*this).m_hAppBarDPA, nItems));
    }

    DPA_Destroy((*this).m_hAppBarDPA);
    (*this).m_hAppBarDPA = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_OnAppBarNew(
    this: *mut CAppBarManager,
    pData: *const APPBAR_COMMAND,
) -> BOOL {
    let hWnd = UlongToHandle((*pData).abd.hWnd32) as HWND;

    if !(*this).m_hAppBarDPA.is_null() {
        if !CAppBarManager_FindAppBar(this, hWnd).is_null() {
            ERR!(b"Already exists: %p\n\0".as_ptr() as PCHAR, hWnd);
            return 0u8;
        }
    } else {
        let c_nGrow: UINT = 4;
        (*this).m_hAppBarDPA = DPA_Create(c_nGrow as INT);
        if (*this).m_hAppBarDPA.is_null() {
            ERR!(b"Out of memory\n\0".as_ptr() as PCHAR);
            return 0u8;
        }
    }

    let pAppBar = LocalAlloc(LPTR, core::mem::size_of::<APPBAR>()) as PAPPBAR;
    if !pAppBar.is_null() {
        (*pAppBar).hWnd = hWnd;
        (*pAppBar).uEdge = UINT_MAX;
        (*pAppBar).uCallbackMessage = (*pData).abd.uCallbackMessage;
        if DPA_InsertPtr((*this).m_hAppBarDPA, INT_MAX, pAppBar as PVOID) >= 0 {
            return 1u8;
        }

        LocalFree(pAppBar as HLOCAL);
    }

    ERR!(b"Out of memory\n\0".as_ptr() as PCHAR);
    0u8
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_OnAppBarRemove(
    this: *mut CAppBarManager,
    pData: *const APPBAR_COMMAND,
) {
    if (*this).m_hAppBarDPA.is_null() {
        return;
    }

    let hWnd = UlongToHandle((*pData).abd.hWnd32) as HWND;
    let mut nItems = DPA_GetPtrCount((*this).m_hAppBarDPA);
    while { nItems -= 1; nItems >= 0 } {
        let pAppBar = DPA_GetPtr((*this).m_hAppBarDPA, nItems) as PAPPBAR;
        if pAppBar.is_null() {
            continue;
        }

        if (*pAppBar).hWnd == hWnd {
            let rcOld = (*pAppBar).rc;
            CAppBarManager_EliminateAppBar(this, nItems);
            CAppBarManager_StuckAppChange(
                this,
                hWnd,
                core::ptr::addr_of!(rcOld),
                core::ptr::null(),
                0u8,
            );
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_OnAppBarQueryPos(
    this: *mut CAppBarManager,
    pData: *mut APPBAR_COMMAND,
) {
    let hWnd = UlongToHandle((*pData).abd.hWnd32) as HWND;
    let pAppBar1 = CAppBarManager_FindAppBar(this, hWnd);
    if pAppBar1.is_null() {
        ERR!(b"Not found: %p\n\0".as_ptr() as *const i8, hWnd);
        return;
    }

    let pOutput = AppBar_LockOutput(pData);
    if pOutput.is_null() {
        ERR!(b"!pOutput: %d\n\0".as_ptr() as *const i8, (*pData).dwProcessId);
        return;
    }
    (*pOutput).rc = (*pData).abd.rc;

    if IsRectEmpty(&(*pOutput).rc) != 0 {
        ERR!(b"IsRectEmpty\n\0".as_ptr() as *const i8);
    }

    let hMon1 = MonitorFromRect(&(*pOutput).rc, MONITOR_DEFAULTTOPRIMARY);

    if hMon1 == CAppBarManager_GetMonitor(this) && CAppBarManager_IsAutoHideState(this) == 0 {
        let mut dummyAppBar: APPBAR = core::mem::zeroed();
        dummyAppBar.uEdge = CAppBarManager_GetPosition(this);
        CAppBarManager_GetDockedRect(this, &mut dummyAppBar.rc);
        CAppBarManager_AppBarSubtractRect(this, &mut dummyAppBar, &mut (*pOutput).rc);
    }

    let uEdge = (*pData).abd.uEdge;
    let mut nItems = DPA_GetPtrCount((*this).m_hAppBarDPA);
    loop {
        nItems -= 1;
        if nItems < 0 {
            break;
        }

        let pAppBar2 = DPA_GetPtr((*this).m_hAppBarDPA, nItems) as *mut APPBAR;
        if pAppBar2.is_null() || (*pAppBar1).hWnd == (*pAppBar2).hWnd {
            continue;
        }

        if (Edge_IsVertical(uEdge) != 0 || Edge_IsVertical((*pAppBar2).uEdge) == 0) &&
            ((*pAppBar1).uEdge != uEdge || CAppBarManager_AppBarOutsideOf(this, pAppBar1, pAppBar2) == 0)
        {
            if (*pAppBar1).uEdge == uEdge || (*pAppBar2).uEdge != uEdge {
                continue;
            }
        }

        let hMon2 = MonitorFromRect(&(*pAppBar2).rc, MONITOR_DEFAULTTONULL);
        if hMon1 == hMon2 {
            CAppBarManager_AppBarSubtractRect(this, pAppBar2, &mut (*pOutput).rc);
        }
    }

    AppBar_UnLockOutput(pOutput);
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_OnAppBarSetPos(
    this: *mut CAppBarManager,
    pData: *mut APPBAR_COMMAND,
) {
    let hWnd = UlongToHandle((*pData).abd.hWnd32) as HWND;
    let pAppBar = CAppBarManager_FindAppBar(this, hWnd);
    if pAppBar.is_null() {
        return;
    }

    CAppBarManager_OnAppBarQueryPos(this, pData);

    let pOutput = AppBar_LockOutput(pData);
    if pOutput.is_null() {
        return;
    }

    let rcOld = (*pAppBar).rc;
    let rcNew = (*pData).abd.rc;
    let bChanged = if EqualRect(&rcOld as *const RECT, &rcNew as *const RECT) != 0 {
        0u8
    } else {
        1u8
    };

    (*pAppBar).rc = rcNew;
    (*pAppBar).uEdge = (*pData).abd.uEdge;

    AppBar_UnLockOutput(pOutput);

    if bChanged != 0 {
        CAppBarManager_StuckAppChange(this, hWnd, &rcOld as *const RECT, &rcNew as *const RECT, 0);
    }
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_OnAppBarNotifyAll(
    this: *mut CAppBarManager,
    hMon: HMONITOR,
    hwndIgnore: HWND,
    dwNotify: u32,
    lParam: LPARAM,
) {

    TRACE!(
        b"%p, %p, 0x%X, %p\n\0".as_ptr() as *const i8,
        hMon,
        hwndIgnore,
        dwNotify,
        lParam,
    );

    if (*this).m_hAppBarDPA.is_null() {
        return;
    }

    let mut nItems = DPA_GetPtrCount((*this).m_hAppBarDPA);
    while { nItems -= 1; nItems >= 0 } {
        let pAppBar = DPA_GetPtr((*this).m_hAppBarDPA, nItems) as PAPPBAR;
        if pAppBar.is_null() || (*pAppBar).hWnd == hwndIgnore {
            continue;
        }

        let hwndAppBar = (*pAppBar).hWnd;
        if IsWindow(hwndAppBar) == 0 {
            CAppBarManager_EliminateAppBar(this, nItems);
            continue;
        }

        if hMon.is_null() || hMon == MonitorFromWindow(hwndAppBar, MONITOR_DEFAULTTONULL) {
            PostMessageW(hwndAppBar, (*pAppBar).uCallbackMessage, dwNotify as WPARAM, lParam);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_AppBarSubtractRect(
    this: *mut CAppBarManager,
    pAppBar: PAPPBAR,
    prc: PRECT,
) {
    match (*pAppBar).uEdge {
        ABE_LEFT => {
            (*prc).left = if (*prc).left > (*pAppBar).rc.right {
                (*prc).left
            } else {
                (*pAppBar).rc.right
            };
        }
        ABE_TOP => {
            (*prc).top = if (*prc).top > (*pAppBar).rc.bottom {
                (*prc).top
            } else {
                (*pAppBar).rc.bottom
            };
        }
        ABE_RIGHT => {
            (*prc).right = if (*prc).right < (*pAppBar).rc.left {
                (*prc).right
            } else {
                (*pAppBar).rc.left
            };
        }
        ABE_BOTTOM => {
            (*prc).bottom = if (*prc).bottom < (*pAppBar).rc.top {
                (*prc).bottom
            } else {
                (*pAppBar).rc.top
            };
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_AppBarOutsideOf(
    this: *mut CAppBarManager,
    pAppBar1: *const APPBAR,
    pAppBar2: *const APPBAR,
) -> BOOL {
    if (*pAppBar1).uEdge != (*pAppBar2).uEdge {
        return 0;
    }

    match (*pAppBar2).uEdge {
        ABE_LEFT => {
            if (*pAppBar1).rc.left >= (*pAppBar2).rc.left {
                1
            } else {
                0
            }
        }
        ABE_TOP => {
            if (*pAppBar1).rc.top >= (*pAppBar2).rc.top {
                1
            } else {
                0
            }
        }
        ABE_RIGHT => {
            if (*pAppBar1).rc.right <= (*pAppBar2).rc.right {
                1
            } else {
                0
            }
        }
        ABE_BOTTOM => {
            if (*pAppBar1).rc.bottom <= (*pAppBar2).rc.bottom {
                1
            } else {
                0
            }
        }
        _ => {
            0
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_GetDockedRect(
    this: *mut CAppBarManager,
    prcDocked: PRECT,
) {
    *prcDocked = *((*this).GetTrayRect.unwrap())(this);
    if ((*this).IsAutoHideState.unwrap())(this) != 0 && ((*this).IsHidingState.unwrap())(this) != 0 {
        CAppBarManager_ComputeHiddenRect(this, prcDocked, ((*this).GetPosition.unwrap())(this) as UINT);
    }
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_ComputeHiddenRect(
    this: *mut CAppBarManager,
    prc: PRECT,
    uSide: UINT,
) {
    let mut mi = MONITORINFO {
        cbSize: core::mem::size_of::<MONITORINFO>() as u32,
        rcMonitor: core::mem::zeroed(),
        rcWork: core::mem::zeroed(),
        dwFlags: 0,
    };
    let hMonitor = MonitorFromRect(prc, MONITOR_DEFAULTTONULL);
    if GetMonitorInfoW(hMonitor, &mut mi) == 0 {
        return;
    }
    let rcMon = mi.rcMonitor;

    let cxy = if Edge_IsVertical(uSide) != 0 {
        (*prc).bottom - (*prc).top
    } else {
        (*prc).right - (*prc).left
    };
    
    match uSide {
        ABE_LEFT => {
            (*prc).right = rcMon.left + GetSystemMetrics(SM_CXFRAME) / 2;
            (*prc).left = (*prc).right - cxy;
        }
        ABE_TOP => {
            (*prc).bottom = rcMon.top + GetSystemMetrics(SM_CYFRAME) / 2;
            (*prc).top = (*prc).bottom - cxy;
        }
        ABE_RIGHT => {
            (*prc).left = rcMon.right - GetSystemMetrics(SM_CXFRAME) / 2;
            (*prc).right = (*prc).left + cxy;
        }
        ABE_BOTTOM => {
            (*prc).top = rcMon.bottom - GetSystemMetrics(SM_CYFRAME) / 2;
            (*prc).bottom = (*prc).top + cxy;
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_StuckAppChange(
    this: *mut CAppBarManager,
    hwndTarget: HWND,
    prcOld: *const RECT,
    prcNew: *const RECT,
    bTray: i32,
) {
    let mut rcWorkArea1: RECT = core::mem::zeroed();
    let mut rcWorkArea2: RECT = core::mem::zeroed();
    let mut hMon1: HMONITOR = core::ptr::null_mut();
    let mut flags: u32 = 0;
    const SET_WORKAREA_1: u32 = 1;
    const SET_WORKAREA_2: u32 = 2;
    const NEED_SIZING: u32 = 4;

    if !prcOld.is_null() {
        hMon1 = if bTray != 0 {
            CAppBarManager_GetPreviousMonitor(this)
        } else {
            MonitorFromRect(prcOld, MONITOR_DEFAULTTONEAREST)
        };
        if !hMon1.is_null() {
            let type1 = CAppBarManager_RecomputeWorkArea(
                this,
                CAppBarManager_GetTrayRect(this),
                hMon1,
                &mut rcWorkArea1,
            );
            if type1 == WORKAREA_TYPE_WORKAREA_IS_NOT_MONITOR {
                flags = SET_WORKAREA_1;
            }
            if type1 == WORKAREA_TYPE_WORKAREA_SAME_AS_MONITOR {
                flags = NEED_SIZING;
            }
        }
    }

    if !prcNew.is_null() {
        let hMon2 = MonitorFromRect(prcNew, MONITOR_DEFAULTTONULL);
        if !hMon2.is_null() && hMon2 != hMon1 {
            let type2 = CAppBarManager_RecomputeWorkArea(
                this,
                CAppBarManager_GetTrayRect(this),
                hMon2,
                &mut rcWorkArea2,
            );
            if type2 == WORKAREA_TYPE_WORKAREA_IS_NOT_MONITOR {
                flags |= SET_WORKAREA_2;
            } else if type2 == WORKAREA_TYPE_WORKAREA_SAME_AS_MONITOR && flags == 0 {
                flags = NEED_SIZING;
            }
        }
    }

    if (flags & SET_WORKAREA_1) != 0 {
        let fWinIni = if flags == SET_WORKAREA_1 && !CAppBarManager_GetDesktopWnd(this).is_null() {
            SPIF_SENDCHANGE
        } else {
            0
        };
        SystemParametersInfoW(SPI_SETWORKAREA, 1, &mut rcWorkArea1 as *mut _ as *mut _, fWinIni);
        CAppBarManager_RedrawDesktop(this, CAppBarManager_GetDesktopWnd(this), &mut rcWorkArea1);
    }

    if (flags & SET_WORKAREA_2) != 0 {
        let fWinIni = if !CAppBarManager_GetDesktopWnd(this).is_null() {
            SPIF_SENDCHANGE
        } else {
            0
        };
        SystemParametersInfoW(SPI_SETWORKAREA, 1, &mut rcWorkArea2 as *mut _ as *mut _, fWinIni);
        CAppBarManager_RedrawDesktop(this, CAppBarManager_GetDesktopWnd(this), &mut rcWorkArea2);
    }

    if bTray != 0 || flags == NEED_SIZING {
        SendMessageW(CAppBarManager_GetDesktopWnd(this), WM_SIZE, 0, 0);
    }

    CAppBarManager_OnAppBarNotifyAll(this, core::ptr::null_mut(), hwndTarget, ABN_POSCHANGED, 1);
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_RedrawDesktop(
    this: *mut CAppBarManager,
    hwndDesktop: HWND,
    prc: PRECT,
) {
    if hwndDesktop.is_null() {
        return;
    }
    MapWindowPoints(
        core::ptr::null_mut(),
        hwndDesktop,
        prc as *mut POINT,
        (core::mem::size_of::<RECT>() / core::mem::size_of::<POINT>()) as i32,
    );
    RedrawWindow(
        hwndDesktop,
        prc,
        core::ptr::null_mut(),
        RDW_ALLCHILDREN | RDW_ERASE | RDW_INVALIDATE,
    );
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_RecomputeWorkArea(
    this: *mut CAppBarManager,
    prcTray: *const RECT,
    hMonitor: HMONITOR,
    prcWorkArea: PRECT,
) -> WORKAREA_TYPE {
    let mut mi = MONITORINFO {
        cbSize: core::mem::size_of::<MONITORINFO>() as u32,
        rcMonitor: core::mem::zeroed(),
        rcWork: core::mem::zeroed(),
        dwFlags: 0,
    };
    
    if GetMonitorInfoW(hMonitor, &mut mi) == 0 {
        return WORKAREA_TYPE_WORKAREA_NO_TRAY_AREA;
    }

    if CAppBarManager_IsAutoHideState(this) != 0 {
        *prcWorkArea = mi.rcMonitor;
    } else {
        SubtractRect(prcWorkArea, &mi.rcMonitor, prcTray);
    }

    if (*this).m_hAppBarDPA != core::ptr::null_mut() {
        let mut nItems = DPA_GetPtrCount((*this).m_hAppBarDPA);
        while { nItems -= 1; nItems >= 0 } {
            let pAppBar = DPA_GetPtr((*this).m_hAppBarDPA, nItems) as PAPPBAR;
            if !pAppBar.is_null() && hMonitor == MonitorFromRect(&(*pAppBar).rc, MONITOR_DEFAULTTONULL) {
                CAppBarManager_AppBarSubtractRect(this, pAppBar, prcWorkArea);
            }
        }
    }

    if EqualRect(prcWorkArea, &mi.rcWork) == 0 {
        return WORKAREA_TYPE_WORKAREA_IS_NOT_MONITOR;
    }

    if CAppBarManager_IsAutoHideState(this) != 0 || IsRectEmpty(prcTray) != 0 {
        return WORKAREA_TYPE_WORKAREA_NO_TRAY_AREA;
    }

    WORKAREA_TYPE_WORKAREA_SAME_AS_MONITOR
}

#[no_mangle]
pub unsafe extern "system" fn CAppBarManager_MonitorEnumProc(
    hMonitor: HMONITOR,
    hDC: HDC,
    prc: LPRECT,
    lParam: LPARAM,
) -> i32 {
    let pThis = lParam as *mut CAppBarManager;

    let mut rcWorkArea: RECT = core::mem::zeroed();
    if CAppBarManager_RecomputeWorkArea(pThis, prc, hMonitor, &mut rcWorkArea) != WORKAREA_IS_NOT_MONITOR {
        return 1;
    }

    let hwndDesktop = CAppBarManager_GetDesktopWnd(pThis);
    SystemParametersInfoW(
        SPI_SETWORKAREA,
        0,
        &mut rcWorkArea as *mut _ as *mut core::ffi::c_void,
        if !hwndDesktop.is_null() { SPIF_SENDCHANGE } else { 0 },
    );
    CAppBarManager_RedrawDesktop(pThis, hwndDesktop, &mut rcWorkArea);
    1
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_RecomputeAllWorkareas(this: *mut CAppBarManager) {
    EnumDisplayMonitors(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Some(CAppBarManager_MonitorEnumProc),
        this as LPARAM,
    );
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_SetAutoHideBar(
    this: *mut CAppBarManager,
    hwndTarget: HWND,
    bSetOrReset: BOOL,
    uSide: UINT,
) -> BOOL {
    let phwndAutoHide = &mut (*this).m_ahwndAutoHideBars[uSide as usize];
    if IsWindow(*phwndAutoHide) == 0 {
        *phwndAutoHide = core::ptr::null_mut();
    }

    if bSetOrReset != 0 {
        if (*phwndAutoHide).is_null() {
            *phwndAutoHide = hwndTarget;
        }
        return if *phwndAutoHide == hwndTarget { 1 } else { 0 };
    } else {
        if *phwndAutoHide == hwndTarget {
            *phwndAutoHide = core::ptr::null_mut();
        }
        return 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_OnAppBarActivationChange2(
    this: *mut CAppBarManager,
    hwndNewAutoHide: HWND,
    uSide: UINT,
) {
    let hwndAutoHideBar = CAppBarManager_OnAppBarGetAutoHideBar(this, uSide);
    if !hwndAutoHideBar.is_null() && hwndAutoHideBar != hwndNewAutoHide {
        PostMessageW(
            CAppBarManager_GetTrayWnd(this),
            TWM_SETZORDER,
            hwndAutoHideBar as WPARAM,
            uSide as LPARAM,
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_GetAppBarMessage(
    this: *mut CAppBarManager,
    pCopyData: PCOPYDATASTRUCT,
) -> PAPPBAR_COMMAND {
    let pData = (*pCopyData).lpData as PAPPBAR_COMMAND;

    if (*pCopyData).cbData != core::mem::size_of::<APPBAR_COMMAND>() as u32
        || (*pData).abd.cbSize != core::mem::size_of_val(&(*pData).abd) as u32
    {
        ERR!(b"Invalid AppBar message\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    pData
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_OnAppBarGetState(this: *mut CAppBarManager) -> UINT {
    let is_auto_hide = ((*this).lpVtbl as *const CAppBarManagerVtbl).as_ref().unwrap().IsAutoHideState.unwrap()(this);
    let is_always_on_top = ((*this).lpVtbl as *const CAppBarManagerVtbl).as_ref().unwrap().IsAlwaysOnTop.unwrap()(this);
    
    (if is_auto_hide != 0 { ABS_AUTOHIDE } else { 0 }) | (if is_always_on_top != 0 { ABS_ALWAYSONTOP } else { 0 })
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_OnAppBarGetTaskbarPos(
    this: *mut CAppBarManager,
    pData: *mut APPBAR_COMMAND,
) -> BOOL {

    let pOutput = AppBar_LockOutput(pData);
    if pOutput.is_null() {
        ERR!(b"!pOutput: %d\n\0".as_ptr() as *const i8, (*pData).dwProcessId);
        return 0u8;
    }

    (*pOutput).rc = *GetTrayRect(this);
    (*pOutput).uEdge = GetPosition(this);

    AppBar_UnLockOutput(pOutput);
    1u8
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_OnAppBarActivationChange(
    this: *mut CAppBarManager,
    pData: *const APPBAR_COMMAND,
) {
    let hWnd = UlongToHandle((*pData).abd.hWnd32) as HWND;
    let pAppBar = FindAppBar(this, hWnd);
    if pAppBar.is_null() {
        ERR!(b"Not found: %p\n\0".as_ptr() as *const i8, hWnd);
        return;
    }

    let hwndAppBar = (*pAppBar).hWnd;
    let mut uSide = ABE_LEFT;
    while uSide <= ABE_BOTTOM {
        if (*this).m_ahwndAutoHideBars[uSide as usize] == hwndAppBar && uSide != (*pAppBar).uEdge {
            return;
        }
        uSide += 1;
    }

    CAppBarManager_OnAppBarActivationChange2(this, hwndAppBar, (*pAppBar).uEdge);
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_OnAppBarGetAutoHideBar(
    this: *mut CAppBarManager,
    uSide: UINT,
) -> HWND {
    if uSide >= 4 {
        return core::ptr::null_mut();
    }

    if IsWindow((*this).m_ahwndAutoHideBars[uSide as usize]) == 0 {
        (*this).m_ahwndAutoHideBars[uSide as usize] = core::ptr::null_mut();
    }
    (*this).m_ahwndAutoHideBars[uSide as usize]
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_OnAppBarSetAutoHideBar(
    this: *mut CAppBarManager,
    pData: *const APPBAR_COMMAND,
) -> BOOL {
    if (*pData).abd.uEdge >= 4 {
        return 0u8;
    }
    let hwndTarget = UlongToHandle((*pData).abd.hWnd32) as HWND;
    CAppBarManager_SetAutoHideBar(
        this,
        hwndTarget,
        (*pData).abd.lParam64 as BOOL,
        (*pData).abd.uEdge,
    )
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_OnAppBarSetState(
    this: *mut CAppBarManager,
    uState: UINT,
) {
    if (uState & !(ABS_AUTOHIDE | ABS_ALWAYSONTOP)) != 0 {
        return;
    }

    ((*(*this).lpVtbl).SetAutoHideState)(this, if (uState & ABS_AUTOHIDE) != 0 { 1 } else { 0 });
    ((*(*this).lpVtbl).UpdateAlwaysOnTop)(this, if (uState & ABS_ALWAYSONTOP) != 0 { 1 } else { 0 });
}

#[no_mangle]
pub unsafe extern "C" fn CAppBarManager_OnAppBarMessage(
    this: *mut CAppBarManager,
    pCopyData: PCOPYDATASTRUCT,
) -> LRESULT {
    let pData = CAppBarManager_GetAppBarMessage(this, pCopyData);
    if pData.is_null() {
        return 0;
    }

    match (*pData).dwMessage {
        ABM_NEW => {
            return CAppBarManager_OnAppBarNew(this, pData) as LRESULT;
        }
        ABM_REMOVE => {
            CAppBarManager_OnAppBarRemove(this, pData);
        }
        ABM_QUERYPOS => {
            CAppBarManager_OnAppBarQueryPos(this, pData);
        }
        ABM_SETPOS => {
            CAppBarManager_OnAppBarSetPos(this, pData);
        }
        ABM_GETSTATE => {
            return CAppBarManager_OnAppBarGetState(this) as LRESULT;
        }
        ABM_GETTASKBARPOS => {
            return CAppBarManager_OnAppBarGetTaskbarPos(this, pData) as LRESULT;
        }
        ABM_ACTIVATE | ABM_WINDOWPOSCHANGED => {
            CAppBarManager_OnAppBarActivationChange(this, pData);
        }
        ABM_GETAUTOHIDEBAR => {
            return CAppBarManager_OnAppBarGetAutoHideBar(this, (*pData).abd.uEdge) as LRESULT;
        }
        ABM_SETAUTOHIDEBAR => {
            return CAppBarManager_OnAppBarSetAutoHideBar(this, pData) as LRESULT;
        }
        ABM_SETSTATE => {
            CAppBarManager_OnAppBarSetState(this, (*pData).abd.lParam64 as UINT);
        }
        _ => {
            FIXME!(b"0x%X\n\0".as_ptr() as *const i8, (*pData).dwMessage);
            return 0;
        }
    }
    1
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Explorer
//  * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
//  * PURPOSE:     AppBar implementation
//  * COPYRIGHT:   Copyright 2008 Vincent Povirk for CodeWeavers
//  *              Copyright 2025 Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// #include "precomp.h"
// #include "appbar.h"
//
// CAppBarManager::CAppBarManager()
//     : m_hAppBarDPA(NULL)
//     , m_ahwndAutoHideBars { 0 }
// {
// }
//
// CAppBarManager::~CAppBarManager()
// {
//     DestroyAppBarDPA();
// }
//
// PAPPBAR CAppBarManager::FindAppBar( HWND hwndAppBar) const
// {
//     if (!m_hAppBarDPA)
//         return NULL;
//
//     INT nItems = DPA_GetPtrCount(m_hAppBarDPA);
//     while (--nItems >= 0)
//     {
//         PAPPBAR pAppBar = (PAPPBAR)DPA_GetPtr(m_hAppBarDPA, nItems);
//         if (pAppBar && hwndAppBar == pAppBar->hWnd)
//             return pAppBar;
//     }
//
//     return NULL;
// }
//
// void CAppBarManager::EliminateAppBar( INT iItem)
// {
//     LocalFree(DPA_GetPtr(m_hAppBarDPA, iItem));
//     DPA_DeletePtr(m_hAppBarDPA, iItem);
// }
//
// void CAppBarManager::DestroyAppBarDPA()
// {
//     if (!m_hAppBarDPA)
//         return;
//
//     INT nItems = DPA_GetPtrCount(m_hAppBarDPA);
//     while (--nItems >= 0)
//     {
//         ::LocalFree(DPA_GetPtr(m_hAppBarDPA, nItems));
//     }
//
//     DPA_Destroy(m_hAppBarDPA);
//     m_hAppBarDPA = NULL;
// }
//
// // ABM_NEW
// BOOL CAppBarManager::OnAppBarNew( const APPBAR_COMMAND *pData)
// {
//     HWND hWnd = (HWND)UlongToHandle(pData->abd.hWnd32);
//
//     if (m_hAppBarDPA)
//     {
//         if (FindAppBar(hWnd))
//         {
//             ERR("Already exists: %p\n", hWnd);
//             return FALSE;
//         }
//     }
//     else
//     {
//         const UINT c_nGrow = 4;
//         m_hAppBarDPA = DPA_Create(c_nGrow);
//         if (!m_hAppBarDPA)
//         {
//             ERR("Out of memory\n");
//             return FALSE;
//         }
//     }
//
//     PAPPBAR pAppBar = (PAPPBAR)::LocalAlloc(LPTR, sizeof(*pAppBar));
//     if (pAppBar)
//     {
//         pAppBar->hWnd = hWnd;
//         pAppBar->uEdge = UINT_MAX;
//         pAppBar->uCallbackMessage = pData->abd.uCallbackMessage;
//         if (DPA_InsertPtr(m_hAppBarDPA, INT_MAX, pAppBar) >= 0)
//             return TRUE; // Success!
//
//         ::LocalFree(pAppBar);
//     }
//
//     ERR("Out of memory\n");
//     return FALSE;
// }
//
// // ABM_REMOVE
// void CAppBarManager::OnAppBarRemove( const APPBAR_COMMAND *pData)
// {
//     if (!m_hAppBarDPA)
//         return;
//
//     HWND hWnd = (HWND)UlongToHandle(pData->abd.hWnd32);
//     INT nItems = DPA_GetPtrCount(m_hAppBarDPA);
//     while (--nItems >= 0)
//     {
//         PAPPBAR pAppBar = (PAPPBAR)DPA_GetPtr(m_hAppBarDPA, nItems);
//         if (!pAppBar)
//             continue;
//
//         if (pAppBar->hWnd == hWnd)
//         {
//             RECT rcOld = pAppBar->rc;
//             EliminateAppBar(nItems);
//             StuckAppChange(hWnd, &rcOld, NULL, FALSE);
//         }
//     }
// }
//
// // ABM_QUERYPOS
// void CAppBarManager::OnAppBarQueryPos( PAPPBAR_COMMAND pData)
// {
//     HWND hWnd = (HWND)UlongToHandle(pData->abd.hWnd32);
//     PAPPBAR pAppBar1 = FindAppBar(hWnd);
//     if (!pAppBar1)
//     {
//         ERR("Not found: %p\n", hWnd);
//         return;
//     }
//
//     PAPPBARDATAINTEROP pOutput = AppBar_LockOutput(pData);
//     if (!pOutput)
//     {
//         ERR("!pOutput: %d\n", pData->dwProcessId);
//         return;
//     }
//     pOutput->rc = pData->abd.rc;
//
//     if (::IsRectEmpty(&pOutput->rc))
//         ERR("IsRectEmpty\n");
//
//     HMONITOR hMon1 = ::MonitorFromRect(&pOutput->rc, MONITOR_DEFAULTTOPRIMARY);
//     ASSERT(hMon1 != NULL);
//
//     // Subtract tray rectangle from pOutput->rc if necessary
//     if (hMon1 == GetMonitor() && !IsAutoHideState())
//     {
//         APPBAR dummyAppBar;
//         dummyAppBar.uEdge = GetPosition();
//         GetDockedRect(&dummyAppBar.rc);
//         AppBarSubtractRect(&dummyAppBar, &pOutput->rc);
//     }
//
//     // Subtract area from pOutput->rc
//     UINT uEdge = pData->abd.uEdge;
//     INT nItems = DPA_GetPtrCount(m_hAppBarDPA);
//     while (--nItems >= 0)
//     {
//         PAPPBAR pAppBar2 = (PAPPBAR)DPA_GetPtr(m_hAppBarDPA, nItems);
//         if (!pAppBar2 || pAppBar1->hWnd == pAppBar2->hWnd)
//             continue;
//
//         if ((Edge_IsVertical(uEdge) || !Edge_IsVertical(pAppBar2->uEdge)) &&
//             (pAppBar1->uEdge != uEdge || !AppBarOutsideOf(pAppBar1, pAppBar2)))
//         {
//             if (pAppBar1->uEdge == uEdge || pAppBar2->uEdge != uEdge)
//                 continue;
//         }
//
//         HMONITOR hMon2 = ::MonitorFromRect(&pAppBar2->rc, MONITOR_DEFAULTTONULL);
//         if (hMon1 == hMon2)
//             AppBarSubtractRect(pAppBar2, &pOutput->rc);
//     }
//
//     AppBar_UnLockOutput(pOutput);
// }
//
// // ABM_SETPOS
// void CAppBarManager::OnAppBarSetPos( PAPPBAR_COMMAND pData)
// {
//     HWND hWnd = (HWND)UlongToHandle(pData->abd.hWnd32);
//     PAPPBAR pAppBar = FindAppBar(hWnd);
//     if (!pAppBar)
//         return;
//
//     OnAppBarQueryPos(pData);
//
//     PAPPBARDATAINTEROP pOutput = AppBar_LockOutput(pData);
//     if (!pOutput)
//         return;
//
//     RECT rcOld = pAppBar->rc, rcNew = pData->abd.rc;
//     BOOL bChanged = !::EqualRect(&rcOld, &rcNew);
//
//     pAppBar->rc = rcNew;
//     pAppBar->uEdge = pData->abd.uEdge;
//
//     AppBar_UnLockOutput(pOutput);
//
//     if (bChanged)
//         StuckAppChange(hWnd, &rcOld, &rcNew, FALSE);
// }
//
// void CAppBarManager::OnAppBarNotifyAll(
//      HMONITOR hMon,
//      HWND hwndIgnore,
//      DWORD dwNotify,
//      LPARAM lParam)
// {
//     TRACE("%p, %p, 0x%X, %p\n", hMon, hwndIgnore, dwNotify, lParam);
//
//     if (!m_hAppBarDPA)
//         return;
//
//     INT nItems = DPA_GetPtrCount(m_hAppBarDPA);
//     while (--nItems >= 0)
//     {
//         PAPPBAR pAppBar = (PAPPBAR)DPA_GetPtr(m_hAppBarDPA, nItems);
//         if (!pAppBar || pAppBar->hWnd == hwndIgnore)
//             continue;
//
//         HWND hwndAppBar = pAppBar->hWnd;
//         if (!::IsWindow(hwndAppBar))
//         {
//             EliminateAppBar(nItems);
//             continue;
//         }
//
//         if (!hMon || hMon == ::MonitorFromWindow(hwndAppBar, MONITOR_DEFAULTTONULL))
//             ::PostMessageW(hwndAppBar, pAppBar->uCallbackMessage, dwNotify, lParam);
//     }
// }
//
// /// @param pAppBar The target AppBar to subtract.
// /// @param prc The rectangle to be subtracted.
// void CAppBarManager::AppBarSubtractRect( PAPPBAR pAppBar,  PRECT prc)
// {
//     switch (pAppBar->uEdge)
//     {
//         case ABE_LEFT:   prc->left   = max(prc->left, pAppBar->rc.right); break;
//         case ABE_TOP:    prc->top    = max(prc->top, pAppBar->rc.bottom); break;
//         case ABE_RIGHT:  prc->right  = min(prc->right, pAppBar->rc.left); break;
//         case ABE_BOTTOM: prc->bottom = min(prc->bottom, pAppBar->rc.top); break;
//         default:
//             ASSERT(FALSE);
//             break;
//     }
// }
//
// // Is pAppBar1 outside of pAppBar2?
// BOOL CAppBarManager::AppBarOutsideOf(
//      const APPBAR *pAppBar1,
//      const APPBAR *pAppBar2)
// {
//     if (pAppBar1->uEdge != pAppBar2->uEdge)
//         return FALSE;
//
//     switch (pAppBar2->uEdge)
//     {
//         case ABE_LEFT:   return pAppBar1->rc.left >= pAppBar2->rc.left;
//         case ABE_TOP:    return pAppBar1->rc.top >= pAppBar2->rc.top;
//         case ABE_RIGHT:  return pAppBar1->rc.right <= pAppBar2->rc.right;
//         case ABE_BOTTOM: return pAppBar1->rc.bottom <= pAppBar2->rc.bottom;
//         default:
//             ASSERT(FALSE);
//             return FALSE;
//     }
// }
//
// /// Get rectangle of the tray window.
// /// @param prcDocked The pointer to the rectangle to be received.
// void CAppBarManager::GetDockedRect( PRECT prcDocked)
// {
//     *prcDocked = *GetTrayRect();
//     if (IsAutoHideState() && IsHidingState())
//         ComputeHiddenRect(prcDocked, GetPosition());
// }
//
// /// Compute the position and size of the hidden TaskBar.
// /// @param prc The rectangle before hiding TaskBar.
// /// @param uSide The side of TaskBar (ABE_...).
// void CAppBarManager::ComputeHiddenRect( PRECT prc,  UINT uSide)
// {
//     MONITORINFO mi = { sizeof(mi) };
//     HMONITOR hMonitor = ::MonitorFromRect(prc, MONITOR_DEFAULTTONULL);
//     if (!::GetMonitorInfoW(hMonitor, &mi))
//         return;
//     RECT rcMon = mi.rcMonitor;
//
//     INT cxy = Edge_IsVertical(uSide) ? (prc->bottom - prc->top) : (prc->right - prc->left);
//     switch (uSide)
//     {
//         case ABE_LEFT:
//             prc->right = rcMon.left + GetSystemMetrics(SM_CXFRAME) / 2;
//             prc->left = prc->right - cxy;
//             break;
//         case ABE_TOP:
//             prc->bottom = rcMon.top + GetSystemMetrics(SM_CYFRAME) / 2;
//             prc->top = prc->bottom - cxy;
//             break;
//         case ABE_RIGHT:
//             prc->left = rcMon.right - GetSystemMetrics(SM_CXFRAME) / 2;
//             prc->right = prc->left + cxy;
//             break;
//         case ABE_BOTTOM:
//             prc->top = rcMon.bottom - GetSystemMetrics(SM_CYFRAME) / 2;
//             prc->bottom = prc->top + cxy;
//             break;
//         default:
//             ASSERT(FALSE);
//             break;
//     }
// }
//
// /// This function is called when AppBar and/or TaskBar is being moved, removed, and/or updated.
// /// @param hwndTarget The target window. Optional.
// /// @param prcOld The old position and size. Optional.
// /// @param prcNew The new position and size. Optional.
// /// @param bTray TRUE if the tray is being moved.
// void
// CAppBarManager::StuckAppChange(
//      HWND hwndTarget,
//      const RECT *prcOld,
//      const RECT *prcNew,
//      BOOL bTray)
// {
//     RECT rcWorkArea1, rcWorkArea2;
//     HMONITOR hMon1 = NULL;
//     UINT flags = 0;
//     enum { SET_WORKAREA_1 = 1, SET_WORKAREA_2 = 2, NEED_SIZING = 4 }; // for flags
//
//     if (prcOld)
//     {
//         hMon1 = (bTray ? GetPreviousMonitor() : ::MonitorFromRect(prcOld, MONITOR_DEFAULTTONEAREST));
//         if (hMon1)
//         {
//             WORKAREA_TYPE type1 = RecomputeWorkArea(GetTrayRect(), hMon1, &rcWorkArea1);
//             if (type1 == WORKAREA_IS_NOT_MONITOR)
//                 flags = SET_WORKAREA_1;
//             if (type1 == WORKAREA_SAME_AS_MONITOR)
//                 flags = NEED_SIZING;
//         }
//     }
//
//     if (prcNew)
//     {
//         HMONITOR hMon2 = ::MonitorFromRect(prcNew, MONITOR_DEFAULTTONULL);
//         if (hMon2 && hMon2 != hMon1)
//         {
//             WORKAREA_TYPE type2 = RecomputeWorkArea(GetTrayRect(), hMon2, &rcWorkArea2);
//             if (type2 == WORKAREA_IS_NOT_MONITOR)
//                 flags |= SET_WORKAREA_2;
//             else if (type2 == WORKAREA_SAME_AS_MONITOR && !flags)
//                 flags = NEED_SIZING;
//         }
//     }
//
//     if (flags & SET_WORKAREA_1)
//     {
//         UINT fWinIni = ((flags == SET_WORKAREA_1 && GetDesktopWnd()) ? SPIF_SENDCHANGE : 0);
//         ::SystemParametersInfoW(SPI_SETWORKAREA, TRUE, &rcWorkArea1, fWinIni);
//         RedrawDesktop(GetDesktopWnd(), &rcWorkArea1);
//     }
//
//     if (flags & SET_WORKAREA_2)
//     {
//         UINT fWinIni = (GetDesktopWnd() ? SPIF_SENDCHANGE : 0);
//         ::SystemParametersInfoW(SPI_SETWORKAREA, TRUE, &rcWorkArea2, fWinIni);
//         RedrawDesktop(GetDesktopWnd(), &rcWorkArea2);
//     }
//
//     if (bTray || flags == NEED_SIZING)
//         ::SendMessageW(GetDesktopWnd(), WM_SIZE, 0, 0);
//
//     // Post ABN_POSCHANGED messages to AppBar windows
//     OnAppBarNotifyAll(NULL, hwndTarget, ABN_POSCHANGED, TRUE);
// }
//
// void CAppBarManager::RedrawDesktop( HWND hwndDesktop,  PRECT prc)
// {
//     if (!hwndDesktop)
//         return;
//     ::MapWindowPoints(NULL, hwndDesktop, (POINT*)prc, sizeof(*prc) / sizeof(POINT));
//     ::RedrawWindow(hwndDesktop, prc, 0, RDW_ALLCHILDREN | RDW_ERASE | RDW_INVALIDATE);
// }
//
// /// Re-compute the work area.
// /// @param prcTray The position and size of the tray window
// /// @param hMonitor The monitor of the work area to re-compute.
// /// @param prcWorkArea The work area to be re-computed.
// WORKAREA_TYPE
// CAppBarManager::RecomputeWorkArea(
//      const RECT *prcTray,
//      HMONITOR hMonitor,
//      PRECT prcWorkArea)
// {
//     MONITORINFO mi = { sizeof(mi) };
//     if (!::GetMonitorInfoW(hMonitor, &mi))
//         return WORKAREA_NO_TRAY_AREA;
//
//     if (IsAutoHideState())
//         *prcWorkArea = mi.rcMonitor;
//     else
//         ::SubtractRect(prcWorkArea, &mi.rcMonitor, prcTray);
//
//     if (m_hAppBarDPA)
//     {
//         INT nItems = DPA_GetPtrCount(m_hAppBarDPA);
//         while (--nItems >= 0)
//         {
//             PAPPBAR pAppBar = (PAPPBAR)DPA_GetPtr(m_hAppBarDPA, nItems);
//             if (pAppBar && hMonitor == ::MonitorFromRect(&pAppBar->rc, MONITOR_DEFAULTTONULL))
//                 AppBarSubtractRect(pAppBar, prcWorkArea);
//         }
//     }
//
//     if (!::EqualRect(prcWorkArea, &mi.rcWork))
//         return WORKAREA_IS_NOT_MONITOR;
//
//     if (IsAutoHideState() || ::IsRectEmpty(prcTray))
//         return WORKAREA_NO_TRAY_AREA;
//
//     return WORKAREA_SAME_AS_MONITOR;
// }
//
// BOOL __stdcall
// CAppBarManager::MonitorEnumProc(
//      HMONITOR hMonitor,
//      HDC hDC,
//      LPRECT prc,
//      LPARAM lParam)
// {
//     CAppBarManager *pThis = (CAppBarManager *)lParam;
//     UNREFERENCED_PARAMETER(hDC);
//
//     RECT rcWorkArea;
//     if (pThis->RecomputeWorkArea(prc, hMonitor, &rcWorkArea) != WORKAREA_IS_NOT_MONITOR)
//         return TRUE;
//
//     HWND hwndDesktop = pThis->GetDesktopWnd();
//     ::SystemParametersInfoW(SPI_SETWORKAREA, 0, &rcWorkArea, hwndDesktop ? SPIF_SENDCHANGE : 0);
//     pThis->RedrawDesktop(hwndDesktop, &rcWorkArea);
//     return TRUE;
// }
//
// void CAppBarManager::RecomputeAllWorkareas()
// {
//     ::EnumDisplayMonitors(NULL, NULL, CAppBarManager::MonitorEnumProc, (LPARAM)this);
// }
//
// BOOL CAppBarManager::SetAutoHideBar( HWND hwndTarget,  BOOL bSetOrReset,  UINT uSide)
// {
//     ATLASSERT(uSide < _countof(m_ahwndAutoHideBars));
//     HWND *phwndAutoHide = &m_ahwndAutoHideBars[uSide];
//     if (!IsWindow(*phwndAutoHide))
//         *phwndAutoHide = NULL;
//
//     if (bSetOrReset) // Set?
//     {
//         if (!*phwndAutoHide)
//             *phwndAutoHide = hwndTarget;
//         return *phwndAutoHide == hwndTarget;
//     }
//     else // Reset
//     {
//         if (*phwndAutoHide == hwndTarget)
//             *phwndAutoHide = NULL;
//         return TRUE;
//     }
// }
//
// void CAppBarManager::OnAppBarActivationChange2( HWND hwndNewAutoHide,  UINT uSide)
// {
//     HWND hwndAutoHideBar = OnAppBarGetAutoHideBar(uSide);
//     if (hwndAutoHideBar && hwndAutoHideBar != hwndNewAutoHide)
//         ::PostMessageW(GetTrayWnd(), TWM_SETZORDER, (WPARAM)hwndAutoHideBar, uSide);
// }
//
// PAPPBAR_COMMAND
// CAppBarManager::GetAppBarMessage( PCOPYDATASTRUCT pCopyData)
// {
//     PAPPBAR_COMMAND pData = (PAPPBAR_COMMAND)pCopyData->lpData;
//
//     if (pCopyData->cbData != sizeof(*pData) || pData->abd.cbSize != sizeof(pData->abd))
//     {
//         ERR("Invalid AppBar message\n");
//         return NULL;
//     }
//
//     return pData;
// }
//
// // ABM_GETSTATE
// UINT CAppBarManager::OnAppBarGetState()
// {
//     return (IsAutoHideState() ? ABS_AUTOHIDE : 0) | (IsAlwaysOnTop() ? ABS_ALWAYSONTOP : 0);
// }
//
// // ABM_GETTASKBARPOS
// BOOL CAppBarManager::OnAppBarGetTaskbarPos( PAPPBAR_COMMAND pData)
// {
//     PAPPBARDATAINTEROP pOutput = AppBar_LockOutput(pData);
//     if (!pOutput)
//     {
//         ERR("!pOutput: %d\n", pData->dwProcessId);
//         return FALSE;
//     }
//
//     pOutput->rc = *GetTrayRect();
//     pOutput->uEdge = GetPosition();
//
//     AppBar_UnLockOutput(pOutput);
//     return TRUE;
// }
//
// // ABM_ACTIVATE, ABM_WINDOWPOSCHANGED
// void CAppBarManager::OnAppBarActivationChange( const APPBAR_COMMAND *pData)
// {
//     HWND hWnd = (HWND)UlongToHandle(pData->abd.hWnd32);
//     PAPPBAR pAppBar = FindAppBar(hWnd);
//     if (!pAppBar)
//     {
//         ERR("Not found: %p\n", hWnd);
//         return;
//     }
//
//     HWND hwndAppBar = pAppBar->hWnd;
//     for (UINT uSide = ABE_LEFT; uSide <= ABE_BOTTOM; ++uSide)
//     {
//         if (m_ahwndAutoHideBars[uSide] == hwndAppBar && uSide != pAppBar->uEdge)
//             return;
//     }
//
//     OnAppBarActivationChange2(hwndAppBar, pAppBar->uEdge);
// }
//
// // ABM_GETAUTOHIDEBAR
// HWND CAppBarManager::OnAppBarGetAutoHideBar( UINT uSide)
// {
//     if (uSide >= _countof(m_ahwndAutoHideBars))
//         return NULL;
//
//     if (!::IsWindow(m_ahwndAutoHideBars[uSide]))
//         m_ahwndAutoHideBars[uSide] = NULL;
//     return m_ahwndAutoHideBars[uSide];
// }
//
// // ABM_SETAUTOHIDEBAR
// BOOL CAppBarManager::OnAppBarSetAutoHideBar( const APPBAR_COMMAND *pData)
// {
//     if (pData->abd.uEdge >= _countof(m_ahwndAutoHideBars))
//         return FALSE;
//     HWND hwndTarget = (HWND)UlongToHandle(pData->abd.hWnd32);
//     return SetAutoHideBar(hwndTarget, (BOOL)pData->abd.lParam64, pData->abd.uEdge);
// }
//
// // ABM_SETSTATE
// void CAppBarManager::OnAppBarSetState( UINT uState)
// {
//     if ((uState & ~(ABS_AUTOHIDE | ABS_ALWAYSONTOP)))
//         return;
//
//     SetAutoHideState(!!(uState & ABS_AUTOHIDE));
//     UpdateAlwaysOnTop(!!(uState & ABS_ALWAYSONTOP));
// }
//
// // WM_COPYDATA TABDMC_APPBAR
// LRESULT CAppBarManager::OnAppBarMessage( PCOPYDATASTRUCT pCopyData)
// {
//     PAPPBAR_COMMAND pData = GetAppBarMessage(pCopyData);
//     if (!pData)
//         return 0;
//
//     switch (pData->dwMessage)
//     {
//         case ABM_NEW:
//             return OnAppBarNew(pData);
//         case ABM_REMOVE:
//             OnAppBarRemove(pData);
//             break;
//         case ABM_QUERYPOS:
//             OnAppBarQueryPos(pData);
//             break;
//         case ABM_SETPOS:
//             OnAppBarSetPos(pData);
//             break;
//         case ABM_GETSTATE:
//             return OnAppBarGetState();
//         case ABM_GETTASKBARPOS:
//             return OnAppBarGetTaskbarPos(pData);
//         case ABM_ACTIVATE:
//         case ABM_WINDOWPOSCHANGED:
//             OnAppBarActivationChange(pData);
//             break;
//         case ABM_GETAUTOHIDEBAR:
//             return (LRESULT)OnAppBarGetAutoHideBar(pData->abd.uEdge);
//         case ABM_SETAUTOHIDEBAR:
//             return OnAppBarSetAutoHideBar(pData);
//         case ABM_SETSTATE:
//             OnAppBarSetState((UINT)pData->abd.lParam64);
//             break;
//         default:
//         {
//             FIXME("0x%X\n", pData->dwMessage);
//             return FALSE;
//         }
//     }
//     return TRUE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
