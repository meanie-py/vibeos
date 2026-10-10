use crate::taskmgr::perfdata::PerfDataGetProcessorUsage;
use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn TrayIcon_GetProcessorUsageIcon(mut CpuUsage: ULONG) -> HICON {
    let mut hTrayIcon: HICON = core::ptr::null_mut();
    let mut hScreenDC: HDC = core::ptr::null_mut();
    let mut hDC: HDC = core::ptr::null_mut();
    let mut hBitmap: HBITMAP = core::ptr::null_mut();
    let mut hOldBitmap: HBITMAP = core::ptr::null_mut();
    let mut hBitmapMask: HBITMAP = core::ptr::null_mut();
    let mut iconInfo: ICONINFO = core::mem::zeroed();
    let mut nLinesToDraw: ::core::ffi::c_int = 0;
    let mut hBitmapBrush: HBRUSH = core::ptr::null_mut();
    let mut rc: RECT = core::mem::zeroed();

    'done: {
        /* Get a handle to the screen DC */
        hScreenDC = GetDC(core::ptr::null_mut());
        if hScreenDC.is_null() {
            break 'done;
        }

        /* Create our own DC from it */
        hDC = CreateCompatibleDC(hScreenDC);
        if hDC.is_null() {
            break 'done;
        }

        /*
         * Load the bitmaps
         */
        hBitmap = LoadBitmapW(hInst, MAKEINTRESOURCEW!(IDB_TRAYICON));
        hBitmapMask = LoadBitmapW(hInst, MAKEINTRESOURCEW!(IDB_TRAYMASK));
        if hBitmap.is_null() || hBitmapMask.is_null() {
            break 'done;
        }

        hBitmapBrush = CreateSolidBrush(RGB!(
            0,
            255,
            0
        ));
        if hBitmapBrush.is_null() {
            break 'done;
        }

        /*
         * Select the bitmap into our device context
         * so we can draw on it.
         */
        hOldBitmap = SelectObject(hDC, (hBitmap).cast()) as HBITMAP;

        /*
         * Calculate how many lines to draw
         * since we have 11 rows of space
         * to draw the cpu usage instead of
         * just having 10.
         */
        nLinesToDraw = (CpuUsage.wrapping_add(CpuUsage.wrapping_div(10)))
            .wrapping_div(11) as ::core::ffi::c_int;
        rc.left = 3 as LONG;
        rc.top = (12 - nLinesToDraw) as LONG;
        rc.right = 13 as LONG;
        rc.bottom = 13 as LONG;

        /* Now draw the cpu usage */
        if nLinesToDraw != 0 {
            FillRect(hDC, &raw mut rc as LPCRECT, hBitmapBrush);
        }

        /*
         * Now that we are done drawing put the
         * old bitmap back.
         */
        hBitmap = SelectObject(hDC, hOldBitmap as HGDIOBJ) as HBITMAP;
        hOldBitmap = core::ptr::null_mut();

        iconInfo.fIcon = TRUE as BOOL;
        iconInfo.hbmMask = hBitmapMask;
        iconInfo.hbmColor = hBitmap;

        hTrayIcon = CreateIconIndirect(&raw mut iconInfo);
    }

    /*
     * Cleanup
     */
    if !hScreenDC.is_null() {
        ReleaseDC(core::ptr::null_mut(), hScreenDC);
    }
    if !hDC.is_null() {
        DeleteDC(hDC);
    }
    if !hBitmapBrush.is_null() {
        DeleteObject(hBitmapBrush as HGDIOBJ);
    }
    if !hBitmap.is_null() {
        DeleteObject(hBitmap as HGDIOBJ);
    }
    if !hBitmapMask.is_null() {
        DeleteObject(hBitmapMask as HGDIOBJ);
    }

    /* Return the newly created tray icon (if successful) */
    return hTrayIcon;
}

unsafe extern "C" fn TrayIcon_Update(dwMessage: DWORD) -> BOOL {
    static mut szMsg: [WCHAR; 64] = [0; 64];

    let mut nid: NOTIFYICONDATAW = core::mem::zeroed();
    let mut CpuUsage: ULONG;
    let mut hIcon: HICON = core::ptr::null_mut();
    let mut bRetVal: BOOL;

    if *(core::ptr::addr_of_mut!(szMsg) as *mut WCHAR) == 0 {
        LoadStringW(
            hInst,
            IDS_MSG_TRAYICONCPUUSAGE as UINT,
            core::ptr::addr_of_mut!(szMsg) as LPWSTR,
            _countof!(szMsg) as ::core::ffi::c_int,
        );
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(nid) as *mut ::core::ffi::c_void,
        core::mem::size_of::<NOTIFYICONDATAW>() as size_t,
    );

    CpuUsage = PerfDataGetProcessorUsage();
    hIcon = TrayIcon_GetProcessorUsageIcon(CpuUsage);

    nid.cbSize = core::mem::size_of::<NOTIFYICONDATAW>() as DWORD;
    nid.hWnd = hMainWnd;
    nid.uID = 0 as UINT;
    nid.uFlags = (NIF_ICON | NIF_MESSAGE | NIF_TIP) as UINT;
    nid.uCallbackMessage = WM_ONTRAYICON as UINT;
    nid.hIcon = hIcon;

    StringCchPrintfW!(
        core::ptr::addr_of_mut!(nid.szTip) as STRSAFE_LPWSTR,
        _countof!(*core::ptr::addr_of!(nid.szTip)) as size_t,
        core::ptr::addr_of_mut!(szMsg) as *mut WCHAR as STRSAFE_LPCWSTR,
        CpuUsage,
    );

    bRetVal = Shell_NotifyIconW(dwMessage, core::ptr::addr_of_mut!(nid));

    if !hIcon.is_null() {
        DestroyIcon(hIcon);
    }

    return bRetVal;
}
#[no_mangle]
pub unsafe extern "C" fn TrayIcon_AddIcon() -> BOOL {
    return TrayIcon_Update(NIM_ADD);
}
#[no_mangle]
pub unsafe extern "C" fn TrayIcon_RemoveIcon() -> BOOL {
    let mut nid: NOTIFYICONDATAW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(nid) as *mut core::ffi::c_void,
        core::mem::size_of::<NOTIFYICONDATAW>(),
    );

    nid.cbSize = core::mem::size_of::<NOTIFYICONDATAW>() as DWORD;
    nid.hWnd = hMainWnd;
    nid.uID = 0 as UINT;
    nid.uFlags = 0 as UINT;
    nid.uCallbackMessage = WM_ONTRAYICON as UINT;

    return Shell_NotifyIconW(NIM_DELETE as DWORD, core::ptr::addr_of_mut!(nid));
}
#[no_mangle]
pub unsafe extern "C" fn TrayIcon_UpdateIcon() -> BOOL {
    return TrayIcon_Update(NIM_MODIFY as DWORD);
}
