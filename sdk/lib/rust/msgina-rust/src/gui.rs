use crate::prelude::*;
use crate::msgina_externs::*;
use crate::msgina_types::*;
use crate::resources::*;
use core::ffi::c_void;

extern "C" {
    fn GetDlgItem(hDlg: *mut core::ffi::c_void, nIDDlgItem: i32) -> *mut core::ffi::c_void;
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn GetWindowTextLengthW(hWnd: *mut core::ffi::c_void) -> i32;
    fn GetWindowTextW(hWnd: *mut core::ffi::c_void, lpString: *mut u16, nMaxCount: i32) -> i32;
    fn HeapAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, dwBytes: usize) -> *mut core::ffi::c_void;
    fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
    fn EndDialog(hDlg: HWND, nResult: INT_PTR);
    fn MAKEINTRESOURCEW(id: u16) -> *mut u16;
}
extern "system" {
    fn GetComputerNameW(lpBuffer: PWSTR, nSize: *mut DWORD) -> i32;
    fn SendMessageW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> isize;
    fn SetWindowTextW(hWnd: HWND, lpString: LPWSTR) -> BOOL;
    fn SetDlgItemTextW(hDlg: HWND, nIDDlgItem: i32, lpString: LPWSTR) -> BOOL;
    fn BeginPaint(hwnd: HWND, lpPaint: *mut PAINTSTRUCT) -> HDC;
    fn DrawStateW( hdc: HDC, hForeBrush: *mut core::ffi::c_void, pCallBack: *mut core::ffi::c_void, lData: isize, wData: usize, x: i32, y: i32, cx: i32, cy: i32, flags: u32, ) -> i32;
    fn EnableWindow(hwnd: HWND, bEnable: i32) -> i32;
    fn EndPaint(hwnd: HWND, lpPaint: *const PAINTSTRUCT) -> i32;
    fn GetWindowLongPtrW(hwnd: HWND, nIndex: i32) -> isize;
    fn PostMessageW(hwnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> i32;
    fn SetFocus(hwnd: HWND) -> HWND;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _DISPLAYSTATUSMSG {
    pub Context: PGINA_CONTEXT,
    pub hDesktop: HDESK,
    pub dwOptions: u32,
    pub pTitle: PWSTR,
    pub pMessage: PWSTR,
    pub StartupEvent: HANDLE,
}
pub type DISPLAYSTATUSMSG = _DISPLAYSTATUSMSG;
pub type PDISPLAYSTATUSMSG = *mut _DISPLAYSTATUSMSG;

#[repr(C)]
pub struct _LEGALNOTICEDATA {
    pub pszCaption: LPWSTR,
    pub pszText: LPWSTR,
}
pub type LEGALNOTICEDATA = _LEGALNOTICEDATA;
pub type PLEGALNOTICEDATA = *mut _LEGALNOTICEDATA;

pub const IDT_BAR: u32 = 1;

#[repr(C)]
pub struct _DLG_DATA {
    pub pgContext: PGINA_CONTEXT,
    pub hLogoBitmap: HBITMAP,
    pub hBarBitmap: HBITMAP,
    pub hWndBarCtrl: HWND,
    pub BarCounter: u32,
    pub LogoWidth: u32,
    pub LogoHeight: u32,
    pub BarWidth: u32,
    pub BarHeight: u32,
}
pub type DLG_DATA = _DLG_DATA;
pub type PDLG_DATA = *mut _DLG_DATA;

pub static mut GinaGraphicalUI: GINA_UI = GINA_UI {
    GUIInitialize: GUIInitialize,
    GUIDisplayStatusMessage: GUIDisplayStatusMessage,
    GUIRemoveStatusMessage: GUIRemoveStatusMessage,
    GUIDisplaySASNotice: GUIDisplaySASNotice,
    GUILoggedOnSAS: GUILoggedOnSAS,
    GUILoggedOutSAS: GUILoggedOutSAS,
    GUILockedSAS: GUILockedSAS,
    GUIDisplayLockedNotice: GUIDisplayLockedNotice,
};

unsafe fn DlgData_Create(hwndDlg: HWND, pgContext: PGINA_CONTEXT) -> PDLG_DATA {
    let p_dlg_data = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<_DLG_DATA>()) as PDLG_DATA;
    if !p_dlg_data.is_null() {
        (*p_dlg_data).pgContext = pgContext;
        SetWindowLongPtrW(hwndDlg, GWLP_USERDATA, p_dlg_data as isize);
    }
    p_dlg_data
}

unsafe fn DlgData_LoadBitmaps(pDlgData: PDLG_DATA) {
    if pDlgData.is_null() {
        return;
    }

    let mut bm: BITMAP = /* zero-init or uninit? */ core::mem::zeroed();

    (*pDlgData).hLogoBitmap = LoadImageW(
        (*(*pDlgData).pgContext).hDllInstance,
        MAKEINTRESOURCEW(IDI_ROSLOGO),
        IMAGE_BITMAP,
        0,
        0,
        LR_DEFAULTCOLOR,
    );
    if !(*pDlgData).hLogoBitmap.is_null() {
        GetObject(
            (*pDlgData).hLogoBitmap,
            core::mem::size_of::<BITMAP>() as i32,
            &mut bm as *mut BITMAP as *mut core::ffi::c_void,
        );
        (*pDlgData).LogoWidth = bm.bmWidth;
        (*pDlgData).LogoHeight = bm.bmHeight;
    }

    (*pDlgData).hBarBitmap = LoadImageW(
        hDllInstance,
        MAKEINTRESOURCEW(IDI_BAR),
        IMAGE_BITMAP,
        0,
        0,
        LR_DEFAULTCOLOR,
    );
    if !(*pDlgData).hBarBitmap.is_null() {
        GetObject(
            (*pDlgData).hBarBitmap,
            core::mem::size_of::<BITMAP>() as i32,
            &mut bm as *mut BITMAP as *mut core::ffi::c_void,
        );
        (*pDlgData).BarWidth = bm.bmWidth;
        (*pDlgData).BarHeight = bm.bmHeight;
    }
}

unsafe fn DlgData_Destroy(hwndDlg: HWND) {
    let pDlgData = GetWindowLongPtrW(hwndDlg, GWLP_USERDATA) as *mut _DLG_DATA;
    if pDlgData.is_null() {
        return;
    }

    SetWindowLongPtrW(hwndDlg, GWLP_USERDATA, 0);

    if !(*pDlgData).hBarBitmap.is_null() {
        DeleteObject((*pDlgData).hBarBitmap as HGDIOBJ);
    }

    if !(*pDlgData).hLogoBitmap.is_null() {
        DeleteObject((*pDlgData).hLogoBitmap as HGDIOBJ);
    }

    HeapFree(GetProcessHeap(), 0, pDlgData as LPVOID);
}

unsafe fn GUIInitialize(pgContext: *mut PGINA_CONTEXT) -> i32 {
    TRACE!(
        b"GUIInitialize(%p)\n\0".as_ptr() as *const c_char,
        pgContext,
    );
    1i32
}

unsafe fn SetWelcomeText(hWnd: HWND) {
    let mut pBuffer: *mut u16 = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut dwWelcomeSize: DWORD = 0;

    /* Open the Winlogon key */
    let rc = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        b"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon\0"
            .as_ptr() as *const u16,
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    );
    if rc != ERROR_SUCCESS {
        eprintln!("RegOpenKeyExW() failed with error {}", rc);
        return;
    }

    /* Get the size of the Welcome value */
    dwWelcomeSize = 0;
    let rc = RegQueryValueExW(
        hKey,
        b"Welcome\0".as_ptr() as *const u16,
        core::ptr::null_mut(),
        &mut dwType,
        core::ptr::null_mut(),
        &mut dwWelcomeSize,
    );
    if rc == ERROR_FILE_NOT_FOUND || dwWelcomeSize == 0 || dwType != REG_SZ {
        goto done;
    }

    let title_len = GetWindowTextLengthW(hWnd) as usize;
    let buf_size = dwWelcomeSize as usize + (title_len + 1) * core::mem::size_of::<u16>();

    pBuffer = HeapAlloc(GetProcessHeap(), 0, buf_size) as *mut u16;
    if pBuffer.is_null() {
        goto done;
    }

    GetWindowTextW(hWnd, pBuffer, (buf_size / core::mem::size_of::<u16>()) as i32);
    const SPACE: [u16; 2] = [0x20, 0];
    wcscat(pBuffer, SPACE.as_ptr());
    let p = pBuffer.add(title_len + 1);

    RegQueryValueExW(
        hKey,
        b"Welcome\0".as_ptr() as *const u16,
        core::ptr::null_mut(),
        &mut dwType,
        p as *mut u8,
        &mut dwWelcomeSize,
    );

    SetWindowText(hWnd, pBuffer);

done:
    if !pBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, pBuffer as *mut _);
    }
    RegCloseKey(hKey);
}

unsafe fn AdjustStatusMessageWindow(hwndDlg: HWND, pDlgData: PDLG_DATA) {
    let hwndLogo = GetDlgItem(hwndDlg, IDC_ROSLOGO);
    let hwndBar = GetDlgItem(hwndDlg, IDC_BAR);
    let hwndLabel = GetDlgItem(hwndDlg, IDC_STATUS_MESSAGE);

    match PRIMARYLANGID(GetUserDefaultLangID()) {
        LANG_CHINESE | LANG_JAPANESE | LANG_KOREAN => (),
        _ => return,
    }

    let mut bmLogo: BITMAP = core::mem::zeroed();
    let mut bmBar: BITMAP = core::mem::zeroed();

    if GetObjectW(
        (*pDlgData).hLogoBitmap,
        core::mem::size_of::<BITMAP>() as i32,
        &mut bmLogo as *mut _ as *mut core::ffi::c_void,
    ) == 0
        || GetObjectW(
            (*pDlgData).hBarBitmap,
            core::mem::size_of::<BITMAP>() as i32,
            &mut bmBar as *mut _ as *mut core::ffi::c_void,
        ) == 0
    {
        return;
    }

    let mut rcBar: RECT = core::mem::zeroed();
    GetWindowRect(hwndBar, &mut rcBar);
    let rcBar_ptr = core::ptr::addr_of_mut!(rcBar) as *mut POINT;
    MapWindowPoints(core::ptr::null_mut(), hwndDlg, rcBar_ptr, 2);
    let dyLabel = bmLogo.bmHeight - rcBar.top;

    let mut rcLabel: RECT = core::mem::zeroed();
    GetWindowRect(hwndLabel, &mut rcLabel);
    let rcLabel_ptr = core::ptr::addr_of_mut!(rcLabel) as *mut POINT;
    MapWindowPoints(core::ptr::null_mut(), hwndDlg, rcLabel_ptr, 2);
    let cxLabel = rcLabel.right - rcLabel.left;
    let cyLabel = rcLabel.bottom - rcLabel.top;

    MoveWindow(
        hwndLogo,
        0,
        0,
        bmLogo.bmWidth,
        bmLogo.bmHeight,
        1u8,
    );
    MoveWindow(
        hwndBar,
        0,
        bmLogo.bmHeight,
        bmLogo.bmWidth,
        bmBar.bmHeight,
        1u8,
    );
    MoveWindow(
        hwndLabel,
        rcLabel.left,
        rcLabel.top + dyLabel,
        cxLabel,
        cyLabel,
        1u8,
    );

    let mut rcWnd: RECT = core::mem::zeroed();
    GetWindowRect(hwndDlg, &mut rcWnd);
    let xOld = rcWnd.left;
    let yOld = rcWnd.top;
    let cxOld = rcWnd.right - rcWnd.left;
    let cyOld = rcWnd.bottom - rcWnd.top;

    let mut rc: RECT = core::mem::zeroed();
    GetClientRect(hwndDlg, &mut rc);
    SetRect(
        &mut rc,
        0,
        0,
        bmLogo.bmWidth,
        rc.bottom - rc.top,
    );

    let style = GetWindowLongPtrW(hwndDlg, GWL_STYLE) as u32;
    let exstyle = GetWindowLongPtrW(hwndDlg, GWL_EXSTYLE) as u32;
    AdjustWindowRectEx(&mut rc, style, 0u8, exstyle);

    let cxNew = rc.right - rc.left;
    let cyNew = (rc.bottom - rc.top) + dyLabel;
    let xNew = xOld - (cxNew - cxOld) / 2;
    let yNew = yOld - (cyNew - cyOld) / 2;
    MoveWindow(hwndDlg, xNew, yNew, cxNew, cyNew, 1u8);
}
unsafe fn StatusDialogProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let pDlgData: PDLG_DATA;
    let _ = wParam;

    pDlgData = GetWindowLongPtrW(hwndDlg, GWLP_USERDATA) as PDLG_DATA;

    match uMsg {
        WM_INITDIALOG => {
            let msg = lParam as PDISPLAYSTATUSMSG;
            if msg.is_null() {
                return 0 as INT_PTR;
            }

            (*(*msg).Context).hStatusWindow = hwndDlg;

            if !(*msg).pTitle.is_null() {
                SetWindowTextW(hwndDlg, (*msg).pTitle);
            }
            SetDlgItemTextW(hwndDlg, IDC_STATUS_MESSAGE, (*msg).pMessage);
            SetEvent((*msg).StartupEvent);

            pDlgData = DlgData_Create(hwndDlg, (*msg).Context);
            if pDlgData.is_null() {
                return 0 as INT_PTR;
            }

            DlgData_LoadBitmaps(pDlgData);
            if !(*pDlgData).hBarBitmap.is_null() {
                if SetTimer(hwndDlg, 1, 20, core::ptr::null_mut()) == 0 {
                    ERR!(b"SetTimer(IDT_BAR) failed: %d\n\0".as_ptr() as PCHAR, GetLastError());
                } else {
                    (*pDlgData).hWndBarCtrl = GetDlgItem(hwndDlg, IDC_BAR);
                }
            }

            AdjustStatusMessageWindow(hwndDlg, pDlgData);
            return 1 as INT_PTR;
        }

        WM_TIMER => {
            if !pDlgData.is_null() && !(*pDlgData).hBarBitmap.is_null() {
                (*pDlgData).BarCounter = ((*pDlgData).BarCounter + 7) % (*pDlgData).BarWidth;
                InvalidateRect((*pDlgData).hWndBarCtrl, core::ptr::null_mut(), 0u8);
                UpdateWindow((*pDlgData).hWndBarCtrl);
            }
            return 1 as INT_PTR;
        }

        WM_DRAWITEM => {
            let lpDis = lParam as LPDRAWITEMSTRUCT;

            if (*lpDis).CtlID != IDC_BAR {
                return 0 as INT_PTR;
            }

            if !pDlgData.is_null() && !(*pDlgData).hBarBitmap.is_null() {
                let hdcMem: HDC;
                let hOld: HGDIOBJ;
                let off = (*pDlgData).BarCounter;
                let iw = (*pDlgData).BarWidth;
                let ih = (*pDlgData).BarHeight;

                hdcMem = CreateCompatibleDC((*lpDis).hDC);
                hOld = SelectObject(hdcMem, (*pDlgData).hBarBitmap as HGDIOBJ);
                BitBlt((*lpDis).hDC, off as i32, 0, (iw - off) as i32, ih as i32, hdcMem, 0, 0, 0x00CC0020u32);
                BitBlt((*lpDis).hDC, 0, 0, off as i32, ih as i32, hdcMem, (iw - off) as i32, 0, 0x00CC0020u32);
                SelectObject(hdcMem, hOld);
                DeleteDC(hdcMem);

                return 1 as INT_PTR;
            }
            return 0 as INT_PTR;
        }

        WM_DESTROY => {
            if !pDlgData.is_null() && !(*pDlgData).hBarBitmap.is_null() {
                KillTimer(hwndDlg, 1);
            }
            DlgData_Destroy(hwndDlg);
            return 1 as INT_PTR;
        }

        _ => {}
    }

    0 as INT_PTR
}
unsafe fn StartupWindowThread(lpParam: LPVOID) -> u32 {
    let msg = lpParam as PDISPLAYSTATUSMSG;
    let mut hDesk: HDESK;

    if DuplicateHandle(
        GetCurrentProcess(),
        (*msg).hDesktop,
        GetCurrentProcess(),
        core::ptr::addr_of_mut!(hDesk) as *mut HANDLE,
        0,
        0i32,
        DUPLICATE_SAME_ACCESS,
    ) == 0
    {
        ERR!(b"Duplicating handle failed!\n".as_ptr());
        HeapFree(GetProcessHeap(), 0, lpParam);
        return 0;
    }

    if SetThreadDesktop(hDesk) == 0 {
        ERR!(b"Setting thread desktop failed!\n".as_ptr());
        HeapFree(GetProcessHeap(), 0, lpParam);
        return 0;
    }

    DialogBoxParamW(
        hDllInstance,
        IDD_STATUS as *const u16,
        GetDesktopWindow(),
        StatusDialogProc,
        msg as LPARAM,
    );

    HeapFree(GetProcessHeap(), 0, lpParam);
    1
}

unsafe fn GUIDisplayStatusMessage(
    pgContext: PGINA_CONTEXT,
    hDesktop: HDESK,
    dwOptions: DWORD,
    pTitle: PWSTR,
    pMessage: PWSTR,
) -> i32 {
    let mut msg: *mut DISPLAYSTATUSMSG;
    let mut Thread: HANDLE;
    let mut ThreadId: DWORD;

    TRACE!(
        {
            static FMT: [u16; 30] = [
                71, 85, 73, 68, 105, 115, 112, 108, 97, 121, 83, 116, 97, 116, 117, 115,
                77, 101, 115, 115, 97, 103, 101, 40, 37, 119, 115, 41, 10, 0,
            ];
            FMT.as_ptr()
        },
        pMessage,
    );

    if (*(pgContext)).hStatusWindow as *mut core::ffi::c_void == core::ptr::null_mut() {
        msg = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<DISPLAYSTATUSMSG>(),
        ) as *mut DISPLAYSTATUSMSG;
        if msg == core::ptr::null_mut() {
            return 0i32;
        }

        (*msg).Context = pgContext;
        (*msg).dwOptions = dwOptions;
        (*msg).pTitle = pTitle;
        (*msg).pMessage = pMessage;
        (*msg).hDesktop = hDesktop;

        (*msg).StartupEvent = CreateEventW(
            core::ptr::null_mut(),
            1i32,
            0i32,
            core::ptr::null_mut(),
        );
        if (*msg).StartupEvent == core::ptr::null_mut() {
            HeapFree(GetProcessHeap(), 0, msg as *mut core::ffi::c_void);
            return 0i32;
        }

        Thread = CreateThread(
            core::ptr::null_mut(),
            0,
            Some(StartupWindowThread),
            msg as *mut core::ffi::c_void,
            0,
            &mut ThreadId,
        );
        if Thread != core::ptr::null_mut() {
            CloseHandle(Thread);
            WaitForSingleObject((*msg).StartupEvent, INFINITE);
            CloseHandle((*msg).StartupEvent);
            return 1i32;
        } else {
            HeapFree(GetProcessHeap(), 0, msg as *mut core::ffi::c_void);
        }

        return 0i32;
    }

    if pTitle != core::ptr::null_mut() {
        SetWindowTextW((*(pgContext)).hStatusWindow, pTitle);
    }
    SetDlgItemTextW((*(pgContext)).hStatusWindow, IDC_STATUS_MESSAGE, pMessage);

    return 1i32;
}

unsafe fn GUIRemoveStatusMessage(pgContext: PGINA_CONTEXT) -> i32 {
    if !(*pgContext).hStatusWindow.is_null() {
        EndDialog((*pgContext).hStatusWindow, 0);
        (*pgContext).hStatusWindow = core::ptr::null_mut();
    }
    1i32
}
unsafe fn WelcomeDialogProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> isize {
    let mut pDlgData: PDLG_DATA = GetWindowLongPtrW(hwndDlg, GWLP_USERDATA) as PDLG_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pDlgData = DlgData_Create(hwndDlg, lParam as PGINA_CONTEXT);
            if pDlgData.is_null() {
                return 0;
            }
            DlgData_LoadBitmaps(pDlgData);
            return 1;
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            if !pDlgData.is_null() && !(*pDlgData).hLogoBitmap.is_null() {
                BeginPaint(hwndDlg, core::ptr::addr_of_mut!(ps));
                DrawStateW(
                    ps.hdc,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    (*pDlgData).hLogoBitmap as LPARAM,
                    0 as WPARAM,
                    0,
                    0,
                    0,
                    0,
                    DST_BITMAP,
                );
                EndPaint(hwndDlg, core::ptr::addr_of_mut!(ps));
            }
            return 1;
        }
        WM_DESTROY => {
            DlgData_Destroy(hwndDlg);
            return 1;
        }
        _ => {}
    }
    0
}

unsafe fn GUIDisplaySASNotice(pgContext: PGINA_CONTEXT) {
    let wlx_funcs = (*pgContext).pWlxFuncs;
    ((*wlx_funcs).WlxDialogBoxParam)(
        (*pgContext).hWlx,
        (*pgContext).hDllInstance,
        IDD_WELCOME as PWSTR,
        GetDesktopWindow(),
        WelcomeDialogProc,
        pgContext as LPARAM,
    );
}

unsafe fn GetTextboxText(
    hwndDlg: *mut core::ffi::c_void,
    TextboxId: i32,
    pText: *mut *mut u16,
) -> i32 {
    let h_item = GetDlgItem(hwndDlg, TextboxId);
    let count = GetWindowTextLengthW(h_item);
    let heap = GetProcessHeap();
    let size = ((count + 1) as usize).wrapping_mul(core::mem::size_of::<u16>());
    let text = HeapAlloc(heap, 0, size) as *mut u16;
    if text.is_null() {
        return 0;
    }
    let result = GetWindowTextW(h_item, text, count + 1);
    if result != count {
        HeapFree(heap, 0, text as *mut core::ffi::c_void);
        return 0;
    }
    *pText = text;
    1
}

unsafe fn ResourceMessageBox(
    pgContext: *mut _GINA_CONTEXT,
    hwnd: HWND,
    uType: u32,
    uCaption: u32,
    uText: u32,
) -> i32 {
    let mut szCaption: [u16; 256] = [0; 256];
    let mut szText: [u16; 256] = [0; 256];

    LoadStringW(
        (*pgContext).hDllInstance,
        uCaption,
        szCaption.as_mut_ptr(),
        szCaption.len() as i32,
    );
    LoadStringW(
        (*pgContext).hDllInstance,
        uText,
        szText.as_mut_ptr(),
        szText.len() as i32,
    );

    let pFuncs = (*pgContext).pWlxFuncs;
    let wlx_message_box = (*pFuncs).WlxMessageBox;
    wlx_message_box(
        (*pgContext).hWlx,
        hwnd,
        szText.as_mut_ptr(),
        szCaption.as_mut_ptr(),
        uType,
    )
}

unsafe fn DoChangePassword(pgContext: *mut _GINA_CONTEXT, hwndDlg: HWND) -> BOOL {
    let mut UserName: [u16; 256] = [0u16; 256];
    let mut Domain: [u16; 256] = [0u16; 256];
    let mut OldPassword: [u16; 256] = [0u16; 256];
    let mut NewPassword1: [u16; 256] = [0u16; 256];
    let mut NewPassword2: [u16; 256] = [0u16; 256];
    let mut RequestBuffer: *mut _MSV1_0_CHANGEPASSWORD_REQUEST = core::ptr::null_mut();
    let mut ResponseBuffer: *mut _MSV1_0_CHANGEPASSWORD_RESPONSE = core::ptr::null_mut();
    let mut RequestBufferSize: u32;
    let mut ResponseBufferSize: u32 = 0u32;
    let mut Ptr: *mut u16;
    let mut res: u8 = 0u8;
    let mut ProtocolStatus: NTSTATUS;
    let mut Status: NTSTATUS;

    GetDlgItemTextW(hwndDlg, IDC_CHANGEPWD_USERNAME, UserName.as_mut_ptr(), UserName.len() as i32);
    GetDlgItemTextW(hwndDlg, IDC_CHANGEPWD_DOMAIN, Domain.as_mut_ptr(), Domain.len() as i32);
    GetDlgItemTextW(hwndDlg, IDC_CHANGEPWD_OLDPWD, OldPassword.as_mut_ptr(), OldPassword.len() as i32);
    GetDlgItemTextW(hwndDlg, IDC_CHANGEPWD_NEWPWD1, NewPassword1.as_mut_ptr(), NewPassword1.len() as i32);
    GetDlgItemTextW(hwndDlg, IDC_CHANGEPWD_NEWPWD2, NewPassword2.as_mut_ptr(), NewPassword2.len() as i32);

    if wcscmp(NewPassword1.as_ptr(), NewPassword2.as_ptr()) != 0 {
        ResourceMessageBox(pgContext,
                           hwndDlg,
                           MB_OK | MB_ICONEXCLAMATION,
                           IDS_CHANGEPWDTITLE,
                           IDS_NONMATCHINGPASSWORDS);
        'done: {
            SecureZeroMemory(OldPassword.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
            SecureZeroMemory(NewPassword1.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
            SecureZeroMemory(NewPassword2.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
            if !RequestBuffer.is_null() {
                SecureZeroMemory((*RequestBuffer).OldPassword.Buffer as *mut core::ffi::c_void,
                                 (*RequestBuffer).OldPassword.MaximumLength as usize);
                SecureZeroMemory(core::ptr::addr_of_mut!((*RequestBuffer).OldPassword) as *mut core::ffi::c_void,
                                 core::mem::size_of::<_UNICODE_STRING>());
                SecureZeroMemory((*RequestBuffer).NewPassword.Buffer as *mut core::ffi::c_void,
                                 (*RequestBuffer).NewPassword.MaximumLength as usize);
                SecureZeroMemory(core::ptr::addr_of_mut!((*RequestBuffer).NewPassword) as *mut core::ffi::c_void,
                                 core::mem::size_of::<_UNICODE_STRING>());
                HeapFree(GetProcessHeap(), 0, RequestBuffer as LPVOID);
            }
            if !ResponseBuffer.is_null() {
                LsaFreeReturnBuffer(ResponseBuffer);
            }
            return res;
        }
    }

    RequestBufferSize = core::mem::size_of::<_MSV1_0_CHANGEPASSWORD_REQUEST>() as u32
        + ((wcslen(Domain.as_ptr()) + 1) * core::mem::size_of::<u16>() as u32)
        + ((wcslen(UserName.as_ptr()) + 1) * core::mem::size_of::<u16>() as u32)
        + ((wcslen(OldPassword.as_ptr()) + 1) * core::mem::size_of::<u16>() as u32)
        + ((wcslen(NewPassword1.as_ptr()) + 1) * core::mem::size_of::<u16>() as u32);

    RequestBuffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, RequestBufferSize as usize) as *mut _MSV1_0_CHANGEPASSWORD_REQUEST;
    if RequestBuffer.is_null() {
        ERR!(b"HeapAlloc failed\n\0".as_ptr() as PCHAR);
        'done2: {
            SecureZeroMemory(OldPassword.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
            SecureZeroMemory(NewPassword1.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
            SecureZeroMemory(NewPassword2.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
            if !RequestBuffer.is_null() {
                SecureZeroMemory((*RequestBuffer).OldPassword.Buffer as *mut core::ffi::c_void,
                                 (*RequestBuffer).OldPassword.MaximumLength as usize);
                SecureZeroMemory(core::ptr::addr_of_mut!((*RequestBuffer).OldPassword) as *mut core::ffi::c_void,
                                 core::mem::size_of::<_UNICODE_STRING>());
                SecureZeroMemory((*RequestBuffer).NewPassword.Buffer as *mut core::ffi::c_void,
                                 (*RequestBuffer).NewPassword.MaximumLength as usize);
                SecureZeroMemory(core::ptr::addr_of_mut!((*RequestBuffer).NewPassword) as *mut core::ffi::c_void,
                                 core::mem::size_of::<_UNICODE_STRING>());
                HeapFree(GetProcessHeap(), 0, RequestBuffer as LPVOID);
            }
            if !ResponseBuffer.is_null() {
                LsaFreeReturnBuffer(ResponseBuffer);
            }
            return res;
        }
    }

    (*RequestBuffer).MessageType = MsV1_0ChangePassword;
    (*RequestBuffer).Impersonating = 1u8;

    Ptr = (RequestBuffer as usize + core::mem::size_of::<_MSV1_0_CHANGEPASSWORD_REQUEST>()) as *mut u16;

    // Pack domain
    (*RequestBuffer).DomainName.Length = (wcslen(Domain.as_ptr()) * core::mem::size_of::<u16>() as u32) as u16;
    (*RequestBuffer).DomainName.MaximumLength = (*RequestBuffer).DomainName.Length + core::mem::size_of::<u16>() as u16;
    (*RequestBuffer).DomainName.Buffer = Ptr;
    RtlCopyMemory((*RequestBuffer).DomainName.Buffer as *mut core::ffi::c_void,
                  Domain.as_ptr() as *const core::ffi::c_void,
                  (*RequestBuffer).DomainName.MaximumLength as usize);
    Ptr = (Ptr as usize + (*RequestBuffer).DomainName.MaximumLength as usize) as *mut u16;

    // Pack username
    (*RequestBuffer).AccountName.Length = (wcslen(UserName.as_ptr()) * core::mem::size_of::<u16>() as u32) as u16;
    (*RequestBuffer).AccountName.MaximumLength = (*RequestBuffer).AccountName.Length + core::mem::size_of::<u16>() as u16;
    (*RequestBuffer).AccountName.Buffer = Ptr;
    RtlCopyMemory((*RequestBuffer).AccountName.Buffer as *mut core::ffi::c_void,
                  UserName.as_ptr() as *const core::ffi::c_void,
                  (*RequestBuffer).AccountName.MaximumLength as usize);
    Ptr = (Ptr as usize + (*RequestBuffer).AccountName.MaximumLength as usize) as *mut u16;

    // Pack old password
    (*RequestBuffer).OldPassword.Length = (wcslen(OldPassword.as_ptr()) * core::mem::size_of::<u16>() as u32) as u16;
    (*RequestBuffer).OldPassword.MaximumLength = (*RequestBuffer).OldPassword.Length + core::mem::size_of::<u16>() as u16;
    (*RequestBuffer).OldPassword.Buffer = Ptr;
    RtlCopyMemory((*RequestBuffer).OldPassword.Buffer as *mut core::ffi::c_void,
                  OldPassword.as_ptr() as *const core::ffi::c_void,
                  (*RequestBuffer).OldPassword.MaximumLength as usize);
    Ptr = (Ptr as usize + (*RequestBuffer).OldPassword.MaximumLength as usize) as *mut u16;

    // Pack new password
    (*RequestBuffer).NewPassword.Length = (wcslen(NewPassword1.as_ptr()) * core::mem::size_of::<u16>() as u32) as u16;
    (*RequestBuffer).NewPassword.MaximumLength = (*RequestBuffer).NewPassword.Length + core::mem::size_of::<u16>() as u16;
    (*RequestBuffer).NewPassword.Buffer = Ptr;
    RtlCopyMemory((*RequestBuffer).NewPassword.Buffer as *mut core::ffi::c_void,
                  NewPassword1.as_ptr() as *const core::ffi::c_void,
                  (*RequestBuffer).NewPassword.MaximumLength as usize);

    if ConnectToLsa(pgContext) != ERROR_SUCCESS {
        ERR!(b"ConnectToLsa() failed\n\0".as_ptr() as PCHAR);
        'done3: {
            SecureZeroMemory(OldPassword.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
            SecureZeroMemory(NewPassword1.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
            SecureZeroMemory(NewPassword2.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
            if !RequestBuffer.is_null() {
                SecureZeroMemory((*RequestBuffer).OldPassword.Buffer as *mut core::ffi::c_void,
                                 (*RequestBuffer).OldPassword.MaximumLength as usize);
                SecureZeroMemory(core::ptr::addr_of_mut!((*RequestBuffer).OldPassword) as *mut core::ffi::c_void,
                                 core::mem::size_of::<_UNICODE_STRING>());
                SecureZeroMemory((*RequestBuffer).NewPassword.Buffer as *mut core::ffi::c_void,
                                 (*RequestBuffer).NewPassword.MaximumLength as usize);
                SecureZeroMemory(core::ptr::addr_of_mut!((*RequestBuffer).NewPassword) as *mut core::ffi::c_void,
                                 core::mem::size_of::<_UNICODE_STRING>());
                HeapFree(GetProcessHeap(), 0, RequestBuffer as LPVOID);
            }
            if !ResponseBuffer.is_null() {
                LsaFreeReturnBuffer(ResponseBuffer);
            }
            return res;
        }
    }

    Status = LsaCallAuthenticationPackage((*pgContext).LsaHandle,
                                          (*pgContext).AuthenticationPackage,
                                          RequestBuffer as *mut core::ffi::c_void,
                                          RequestBufferSize,
                                          core::ptr::addr_of_mut!(ResponseBuffer) as *mut core::ffi::c_void,
                                          &mut ResponseBufferSize,
                                          &mut ProtocolStatus);
    if !NT_SUCCESS(Status) {
        ERR!(b"LsaCallAuthenticationPackage failed (Status 0x%08lx)\n\0".as_ptr() as PCHAR, Status);
        'done4: {
            SecureZeroMemory(OldPassword.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
            SecureZeroMemory(NewPassword1.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
            SecureZeroMemory(NewPassword2.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
            if !RequestBuffer.is_null() {
                SecureZeroMemory((*RequestBuffer).OldPassword.Buffer as *mut core::ffi::c_void,
                                 (*RequestBuffer).OldPassword.MaximumLength as usize);
                SecureZeroMemory(core::ptr::addr_of_mut!((*RequestBuffer).OldPassword) as *mut core::ffi::c_void,
                                 core::mem::size_of::<_UNICODE_STRING>());
                SecureZeroMemory((*RequestBuffer).NewPassword.Buffer as *mut core::ffi::c_void,
                                 (*RequestBuffer).NewPassword.MaximumLength as usize);
                SecureZeroMemory(core::ptr::addr_of_mut!((*RequestBuffer).NewPassword) as *mut core::ffi::c_void,
                                 core::mem::size_of::<_UNICODE_STRING>());
                HeapFree(GetProcessHeap(), 0, RequestBuffer as LPVOID);
            }
            if !ResponseBuffer.is_null() {
                LsaFreeReturnBuffer(ResponseBuffer);
            }
            return res;
        }
    }

    if !NT_SUCCESS(ProtocolStatus) {
        TRACE!(b"LsaCallAuthenticationPackage failed (ProtocolStatus 0x%08lx)\n\0".as_ptr() as PCHAR, ProtocolStatus);
        'done5: {
            SecureZeroMemory(OldPassword.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
            SecureZeroMemory(NewPassword1.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
            SecureZeroMemory(NewPassword2.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
            if !RequestBuffer.is_null() {
                SecureZeroMemory((*RequestBuffer).OldPassword.Buffer as *mut core::ffi::c_void,
                                 (*RequestBuffer).OldPassword.MaximumLength as usize);
                SecureZeroMemory(core::ptr::addr_of_mut!((*RequestBuffer).OldPassword) as *mut core::ffi::c_void,
                                 core::mem::size_of::<_UNICODE_STRING>());
                SecureZeroMemory((*RequestBuffer).NewPassword.Buffer as *mut core::ffi::c_void,
                                 (*RequestBuffer).NewPassword.MaximumLength as usize);
                SecureZeroMemory(core::ptr::addr_of_mut!((*RequestBuffer).NewPassword) as *mut core::ffi::c_void,
                                 core::mem::size_of::<_UNICODE_STRING>());
                HeapFree(GetProcessHeap(), 0, RequestBuffer as LPVOID);
            }
            if !ResponseBuffer.is_null() {
                LsaFreeReturnBuffer(ResponseBuffer);
            }
            return res;
        }
    }

    res = 1u8;

    ResourceMessageBox(pgContext,
                       hwndDlg,
                       MB_OK | MB_ICONINFORMATION,
                       IDS_CHANGEPWDTITLE,
                       IDS_PASSWORDCHANGED);

    if wcscmp(UserName.as_ptr(), (*pgContext).UserName.as_ptr()) == 0
        && wcscmp(Domain.as_ptr(), (*pgContext).DomainName.as_ptr()) == 0
        && wcscmp(OldPassword.as_ptr(), (*pgContext).Password.as_ptr()) == 0
    {
        SecureZeroMemory(core::ptr::addr_of_mut!((*pgContext).Password) as *mut core::ffi::c_void,
                         core::mem::size_of_val(&(*pgContext).Password));
        wcscpy((*pgContext).Password.as_mut_ptr(), NewPassword1.as_ptr());
    }

    // done:
    SecureZeroMemory(OldPassword.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
    SecureZeroMemory(NewPassword1.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
    SecureZeroMemory(NewPassword2.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of::<[u16; 256]>());
    if !RequestBuffer.is_null() {
        SecureZeroMemory((*RequestBuffer).OldPassword.Buffer as *mut core::ffi::c_void,
                         (*RequestBuffer).OldPassword.MaximumLength as usize);
        SecureZeroMemory(core::ptr::addr_of_mut!((*RequestBuffer).OldPassword) as *mut core::ffi::c_void,
                         core::mem::size_of::<_UNICODE_STRING>());
        SecureZeroMemory((*RequestBuffer).NewPassword.Buffer as *mut core::ffi::c_void,
                         (*RequestBuffer).NewPassword.MaximumLength as usize);
        SecureZeroMemory(core::ptr::addr_of_mut!((*RequestBuffer).NewPassword) as *mut core::ffi::c_void,
                         core::mem::size_of::<_UNICODE_STRING>());
        HeapFree(GetProcessHeap(), 0, RequestBuffer as LPVOID);
    }
    if !ResponseBuffer.is_null() {
        LsaFreeReturnBuffer(ResponseBuffer);
    }

    res
}
unsafe fn ChangePasswordDialogProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> isize {
    let pgContext = GetWindowLongPtrW(hwndDlg, GWLP_USERDATA) as *mut _GINA_CONTEXT;
    match uMsg {
        WM_INITDIALOG => {
            let pgContext = lParam as *mut _GINA_CONTEXT;
            SetWindowLongPtrW(hwndDlg, GWLP_USERDATA, pgContext as isize);
            SetDlgItemTextW(hwndDlg, IDC_CHANGEPWD_USERNAME, (*pgContext).UserName);
            SendDlgItemMessageW(
                hwndDlg,
                IDC_CHANGEPWD_DOMAIN,
                CB_ADDSTRING,
                0,
                (*pgContext).DomainName as LPARAM,
            );
            SendDlgItemMessageW(
                hwndDlg,
                IDC_CHANGEPWD_DOMAIN,
                CB_SETCURSEL,
                0,
                0,
            );
            SetFocus(GetDlgItem(hwndDlg, IDC_CHANGEPWD_OLDPWD));
            return 0;
        }
        WM_COMMAND => {
            let loword = (wParam & 0xFFFF) as u32;
            match loword {
                IDOK => {
                    if DoChangePassword(pgContext, hwndDlg) != 0 {
                        EndDialog(hwndDlg, 1);
                    } else {
                        SetDlgItemTextW(hwndDlg, IDC_CHANGEPWD_OLDPWD, core::ptr::null_mut());
                        SetDlgItemTextW(hwndDlg, IDC_CHANGEPWD_NEWPWD1, core::ptr::null_mut());
                        SetDlgItemTextW(hwndDlg, IDC_CHANGEPWD_NEWPWD2, core::ptr::null_mut());
                        SendMessageW(
                            hwndDlg,
                            WM_NEXTDLGCTL,
                            GetDlgItem(hwndDlg, IDC_CHANGEPWD_OLDPWD) as WPARAM,
                            1 as LPARAM,
                        );
                    }
                    return 1;
                }
                IDCANCEL => {
                    EndDialog(hwndDlg, 0);
                    return 1;
                }
                _ => {}
            }
        }
        WM_CLOSE => {
            EndDialog(hwndDlg, 0);
            return 1;
        }
        _ => {}
    }
    0
}

unsafe fn OnInitSecurityDlg(hwnd: HWND, pgContext: PGINA_CONTEXT) {
    unsafe extern "system" {
        fn LoadStringW(hInstance: HINSTANCE, uID: u32, lpBuffer: *mut u16, cchBufferMax: i32) -> i32;
        fn wsprintfW(buf: *mut u16, fmt: *const u16, ...) -> i32;
        fn SetDlgItemTextW(hDlg: HWND, nIDDlgItem: i32, lpString: *const u16) -> i32;
        fn GetDateFormatW(Locale: u32, dwFlags: u32, lpDate: *const SYSTEMTIME, lpFormat: *const u16, lpDateStr: *mut u16, cchDate: i32) -> i32;
        fn GetTimeFormatW(Locale: u32, dwFlags: u32, lpTime: *const SYSTEMTIME, lpFormat: *const u16, lpTimeStr: *mut u16, cchTime: i32) -> i32;
        fn RegOpenKeyExW(hKey: HKEY, lpSubKey: *const u16, ulOptions: u32, samDesired: u32, phkResult: *mut HKEY) -> LONG;
        fn ReadRegDwordValue(hKey: HKEY, lpValueName: *const u16, pdwValue: *mut u32) -> LONG;
        fn EnableWindow(hWnd: HWND, bEnable: u8) -> i32;
        fn GetDlgItem(hDlg: HWND, nIDDlgItem: i32) -> HWND;
        fn RegCloseKey(hKey: HKEY) -> LONG;
        fn RegOpenLoggedOnHKCU(token: HANDLE, samDesired: u32, phkResult: *mut HKEY) -> LONG;
        fn TestTokenPrivilege(token: HANDLE, privilege: u32) -> i32;
    }

    const FMT_SS: &[u16] = &[0x25, 0x73, 0x5C, 0x25, 0x73, 0x00]; // "%s\%s\0"
    const SUBKEY_WINLOGON: &[u16] = &[
        b'S' as u16, b'O' as u16, b'F' as u16, b'T' as u16, b'W' as u16, b'A' as u16, b'R' as u16, b'E' as u16, b'\\' as u16,
        b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16,
        b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16, b' ' as u16, b'N' as u16, b'T' as u16, b'\\' as u16,
        b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'V' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16, b'o' as u16, b'n' as u16, b'\\' as u16,
        b'W' as u16, b'i' as u16, b'n' as u16, b'l' as u16, b'o' as u16, b'g' as u16, b'o' as u16, b'n' as u16, 0
    ];
    const VALUE_DISABLE_LOCK: &[u16] = &[
        b'D' as u16, b'i' as u16, b's' as u16, b'a' as u16, b'b' as u16, b'l' as u16, b'e' as u16,
        b'L' as u16, b'o' as u16, b'c' as u16, b'k' as u16,
        b'W' as u16, b'o' as u16, b'r' as u16, b'k' as u16, b's' as u16, b't' as u16, b'a' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16,
        0
    ];
    const SUBKEY_SYSTEM: &[u16] = &[
        b'S' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'w' as u16, b'a' as u16, b'r' as u16, b'e' as u16, b'\\' as u16,
        b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16,
        b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16, b'\\' as u16,
        b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'V' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16, b'o' as u16, b'n' as u16, b'\\' as u16,
        b'P' as u16, b'o' as u16, b'l' as u16, b'i' as u16, b'c' as u16, b'i' as u16, b'e' as u16, b's' as u16, b'\\' as u16,
        b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, 0
    ];
    const VALUE_DISABLE_CHANGE_PWD: &[u16] = &[
        b'D' as u16, b'i' as u16, b's' as u16, b'a' as u16, b'b' as u16, b'l' as u16, b'e' as u16,
        b'C' as u16, b'h' as u16, b'a' as u16, b'n' as u16, b'g' as u16, b'e' as u16,
        b'P' as u16, b'a' as u16, b's' as u16, b's' as u16, b'w' as u16, b'o' as u16, b'r' as u16, b'd' as u16,
        0
    ];
    const VALUE_DISABLE_TASKMGR: &[u16] = &[
        b'D' as u16, b'i' as u16, b's' as u16, b'a' as u16, b'b' as u16, b'l' as u16, b'e' as u16,
        b'T' as u16, b'a' as u16, b's' as u16, b'k' as u16,
        b'M' as u16, b'g' as u16, b'r' as u16, 0
    ];
    const SUBKEY_EXPLORER: &[u16] = &[
        b'S' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'w' as u16, b'a' as u16, b'r' as u16, b'e' as u16, b'\\' as u16,
        b'M' as u16, b'i' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'\\' as u16,
        b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b's' as u16, b'\\' as u16,
        b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16, b'V' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16, b'o' as u16, b'n' as u16, b'\\' as u16,
        b'P' as u16, b'o' as u16, b'l' as u16, b'i' as u16, b'c' as u16, b'i' as u16, b'e' as u16, b's' as u16, b'\\' as u16,
        b'E' as u16, b'x' as u16, b'p' as u16, b'l' as u16, b'o' as u16, b'r' as u16, b'e' as u16, b'r' as u16, 0
    ];
    const VALUE_NO_LOGOFF: &[u16] = &[
        b'N' as u16, b'o' as u16, b'L' as u16, b'o' as u16, b'g' as u16, b'o' as u16, b'f' as u16, b'f' as u16, 0
    ];
    const VALUE_NO_CLOSE: &[u16] = &[
        b'N' as u16, b'o' as u16, b'C' as u16, b'l' as u16, b'o' as u16, b's' as u16, b'e' as u16, 0
    ];

    let mut hKeyCurrentUser: HKEY = core::ptr::null_mut();
    let mut hKey: HKEY;
    let mut dwValue: u32 = 0;
    let mut lRet: LONG;

    let mut buffer1: [u16; 256] = [0u16; 256];
    let mut buffer2: [u16; 256] = [0u16; 256];
    let mut buffer3: [u16; 256] = [0u16; 256];
    let mut buffer4: [u16; 512] = [0u16; 512];

    LoadStringW((*pgContext).hDllInstance, IDS_LOGONMSG, buffer1.as_mut_ptr(), buffer1.len() as i32);

    wsprintfW(buffer2.as_mut_ptr(), FMT_SS.as_ptr(), (*pgContext).DomainName, (*pgContext).UserName);
    wsprintfW(buffer4.as_mut_ptr(), buffer1.as_ptr(), buffer2.as_ptr());
    SetDlgItemTextW(hwnd, IDC_SECURITY_MESSAGE, buffer4.as_ptr());

    LoadStringW((*pgContext).hDllInstance, IDS_LOGONDATE, buffer1.as_mut_ptr(), buffer1.len() as i32);

    let logon_time_ptr = core::ptr::addr_of!((*pgContext).LogonTime) as *const SYSTEMTIME;
    GetDateFormatW(LOCALE_USER_DEFAULT, DATE_SHORTDATE, logon_time_ptr, core::ptr::null(), buffer2.as_mut_ptr(), buffer2.len() as i32);
    GetTimeFormatW(LOCALE_USER_DEFAULT, 0, logon_time_ptr, core::ptr::null(), buffer3.as_mut_ptr(), buffer3.len() as i32);

    wsprintfW(buffer4.as_mut_ptr(), buffer1.as_ptr(), buffer2.as_ptr(), buffer3.as_ptr());
    SetDlgItemTextW(hwnd, IDC_SECURITY_LOGONDATE, buffer4.as_ptr());

    lRet = RegOpenKeyExW(HKEY_LOCAL_MACHINE, SUBKEY_WINLOGON.as_ptr(), 0, KEY_QUERY_VALUE, &mut hKey);
    if lRet == ERROR_SUCCESS {
        lRet = ReadRegDwordValue(hKey, VALUE_DISABLE_LOCK.as_ptr(), &mut dwValue);
        if lRet == ERROR_SUCCESS && dwValue != 0 {
            EnableWindow(GetDlgItem(hwnd, IDC_SECURITY_LOCK), 0u8);
        }
        RegCloseKey(hKey);
    }

    lRet = RegOpenLoggedOnHKCU((*pgContext).UserToken, KEY_QUERY_VALUE, &mut hKeyCurrentUser);
    if lRet != ERROR_SUCCESS {
        return;
    }

    lRet = RegOpenKeyExW(hKeyCurrentUser, SUBKEY_SYSTEM.as_ptr(), 0, KEY_QUERY_VALUE, &mut hKey);
    if lRet == ERROR_SUCCESS {
        lRet = ReadRegDwordValue(hKey, VALUE_DISABLE_LOCK.as_ptr(), &mut dwValue);
        if lRet == ERROR_SUCCESS && dwValue != 0 {
            EnableWindow(GetDlgItem(hwnd, IDC_SECURITY_LOCK), 0u8);
        }
        lRet = ReadRegDwordValue(hKey, VALUE_DISABLE_CHANGE_PWD.as_ptr(), &mut dwValue);
        if lRet == ERROR_SUCCESS && dwValue != 0 {
            EnableWindow(GetDlgItem(hwnd, IDC_SECURITY_CHANGEPWD), 0u8);
        }
        lRet = ReadRegDwordValue(hKey, VALUE_DISABLE_TASKMGR.as_ptr(), &mut dwValue);
        if lRet == ERROR_SUCCESS && dwValue != 0 {
            EnableWindow(GetDlgItem(hwnd, IDC_SECURITY_TASKMGR), 0u8);
        }
        RegCloseKey(hKey);
    }

    lRet = RegOpenKeyExW(hKeyCurrentUser, SUBKEY_EXPLORER.as_ptr(), 0, KEY_QUERY_VALUE, &mut hKey);
    if lRet == ERROR_SUCCESS {
        lRet = ReadRegDwordValue(hKey, VALUE_NO_LOGOFF.as_ptr(), &mut dwValue);
        if lRet == ERROR_SUCCESS && dwValue != 0 {
            EnableWindow(GetDlgItem(hwnd, IDC_SECURITY_LOGOFF), 0u8);
        }
        lRet = ReadRegDwordValue(hKey, VALUE_NO_CLOSE.as_ptr(), &mut dwValue);
        if (lRet == ERROR_SUCCESS && dwValue != 0) || 
           (TestTokenPrivilege((*pgContext).UserToken, SE_SHUTDOWN_PRIVILEGE) == 0)
        {
            EnableWindow(GetDlgItem(hwnd, IDC_SECURITY_SHUTDOWN), 0u8);
        }
        RegCloseKey(hKey);
    }

    RegCloseKey(hKeyCurrentUser);
}

unsafe fn OnChangePassword(hwnd: HWND, pgContext: PGINA_CONTEXT) -> BOOL {
    let res: i32;
    TRACE!(b"OnChangePassword()\n\0".as_ptr().cast::<core::ffi::c_char>());
    res = (*(*pgContext).pWlxFuncs).WlxDialogBoxParam(
        (*pgContext).hWlx,
        (*pgContext).hDllInstance,
        MAKEINTRESOURCEW(IDD_CHANGEPWD),
        hwnd,
        Some(ChangePasswordDialogProc as unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> isize),
        pgContext as LPARAM,
    );
    TRACE!(b"Result: %x\n\0".as_ptr().cast::<core::ffi::c_char>(), res as core::ffi::c_int);
    0i32
}
unsafe fn LogOffDialogProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => return 1isize,
        WM_COMMAND => {
            let cmd = ((wParam as u32) & 0xFFFF) as INT_PTR;
            if cmd == IDOK || cmd == IDCANCEL {
                EndDialog(hwndDlg, cmd);
                return 1isize;
            }
        }
        WM_CLOSE => {
            EndDialog(hwndDlg, IDCANCEL);
            return 1isize;
        }
        _ => {}
    }
    0isize
}

unsafe fn OnLogOff(hwndDlg: HWND, pgContext: PGINA_CONTEXT) -> INT {
    (*(*pgContext).pWlxFuncs).WlxDialogBoxParam(
        (*pgContext).hWlx,
        (*pgContext).hDllInstance,
        MAKEINTRESOURCEW(IDD_LOGOFF),
        hwndDlg,
        LogOffDialogProc,
        pgContext as LPARAM,
    )
}

unsafe fn OnShutDown(
    hwndDlg: HWND,
    pgContext: PGINA_CONTEXT,
    dwExcludeOptions: u32,
) -> i32 {
    let mut hKeyCurrentUser: HKEY = core::ptr::null_mut();
    let mut shutdown_options: u32 = 0;
    let ret: i32;

    if RegOpenLoggedOnHKCU(
        (*pgContext).UserToken,
        KEY_QUERY_VALUE | KEY_CREATE_SUB_KEY | KEY_SET_VALUE,
        &mut hKeyCurrentUser as *mut _,
    ) != ERROR_SUCCESS
    {
        ERR!(
            b"RegOpenLoggedOnHKCU() failed with error %ld\n\0".as_ptr() as *const i8,
            GetLastError(),
        );
    }

    (*pgContext).nShutdownAction = 0;

    if !hKeyCurrentUser.is_null() {
        shutdown_options =
            GetAllowedShutdownOptions(hKeyCurrentUser, (*pgContext).UserToken);
        (*pgContext).nShutdownAction = LoadShutdownSelState(hKeyCurrentUser);
    }

    shutdown_options &= !dwExcludeOptions;

    ret = ShutdownDialog(hwndDlg, shutdown_options, pgContext) as i32;

    if ret == IDOK as i32 && !hKeyCurrentUser.is_null() {
        SaveShutdownSelState(hKeyCurrentUser, (*pgContext).nShutdownAction);
    }

    if !hKeyCurrentUser.is_null() {
        RegCloseKey(hKeyCurrentUser);
    }

    ret
}
unsafe fn SecurityDialogProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let pgContext = GetWindowLongPtrW(hwndDlg, GWLP_USERDATA) as PGINA_CONTEXT;

    match uMsg {
        WM_INITDIALOG => {
            let pgContext = lParam as PGINA_CONTEXT;
            SetWindowLongPtrW(hwndDlg, GWLP_USERDATA, pgContext as isize);
            SetWelcomeText(hwndDlg);
            OnInitSecurityDlg(hwndDlg, lParam as PGINA_CONTEXT);
            1
        }
        WM_COMMAND => {
            match (wParam & 0xFFFF) as u16 {
                IDC_SECURITY_LOCK => {
                    EndDialog(hwndDlg, WLX_SAS_ACTION_LOCK_WKSTA as isize);
                    1
                }
                IDC_SECURITY_LOGOFF => {
                    if GetKeyState(0x11) & 0x8000 != 0 {
                        if ResourceMessageBox(
                            pgContext,
                            hwndDlg,
                            MB_OKCANCEL | MB_ICONSTOP,
                            IDS_EMERGENCYLOGOFFTITLE,
                            IDS_EMERGENCYLOGOFF,
                        ) == IDOK
                        {
                            EndDialog(hwndDlg, WLX_SAS_ACTION_FORCE_LOGOFF as isize);
                        }
                    } else if OnLogOff(hwndDlg, pgContext) == IDOK {
                        EndDialog(hwndDlg, WLX_SAS_ACTION_LOGOFF as isize);
                    }
                    1
                }
                IDC_SECURITY_SHUTDOWN => {
                    if GetKeyState(0x11) & 0x8000 != 0 {
                        if ResourceMessageBox(
                            pgContext,
                            hwndDlg,
                            MB_OKCANCEL | MB_ICONSTOP,
                            IDS_EMERGENCYRESTARTTITLE,
                            IDS_EMERGENCYRESTART,
                        ) == IDOK
                        {
                            let mut Old: BOOLEAN = 0;
                            ERR!(b"Emergency restarting NT...\n\0".as_ptr() as *const i8);
                            RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE, 1u8, 0u8, &mut Old);
                            NtShutdownSystem(ShutdownReboot);
                            RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE, Old, 0u8, &mut Old);
                        }
                    } else if OnShutDown(hwndDlg, pgContext, 0) == IDOK {
                        EndDialog(hwndDlg, (*pgContext).nShutdownAction as isize);
                    }
                    1
                }
                IDC_SECURITY_CHANGEPWD => {
                    if OnChangePassword(hwndDlg, pgContext) != 0 {
                        EndDialog(hwndDlg, WLX_SAS_ACTION_PWD_CHANGED as isize);
                    }
                    1
                }
                IDC_SECURITY_TASKMGR => {
                    EndDialog(hwndDlg, WLX_SAS_ACTION_TASKLIST as isize);
                    1
                }
                IDCANCEL => {
                    EndDialog(hwndDlg, WLX_SAS_ACTION_NONE as isize);
                    1
                }
                _ => 0,
            }
        }
        WM_CLOSE => {
            EndDialog(hwndDlg, WLX_SAS_ACTION_NONE as isize);
            1
        }
        _ => 0,
    }
}

unsafe fn GUILoggedOnSAS(
    pgContext: PGINA_CONTEXT,
    dwSasType: u32,
) -> i32 {
    if dwSasType != WLX_SAS_TYPE_CTRL_ALT_DEL {
        return WLX_SAS_ACTION_NONE;
    }

    let mut result = (*(*pgContext).pWlxFuncs).WlxDialogBoxParam(
        (*pgContext).hWlx,
        (*pgContext).hDllInstance,
        MAKINTRESOURCEW(IDD_SECURITY),
        GetDesktopWindow(),
        SecurityDialogProc,
        pgContext as isize,
    );

    if result < WLX_SAS_ACTION_LOGON || result > WLX_SAS_ACTION_SWITCH_CONSOLE {
        result = WLX_SAS_ACTION_NONE;
    }

    result
}

unsafe fn DoLogon(
    hwndDlg: HWND,
    pgContext: PGINA_CONTEXT,
) -> BOOL {
    let mut result: BOOL = 0;
    let mut UserName: LPWSTR = core::ptr::null_mut();
    let mut Password: LPWSTR = core::ptr::null_mut();
    let mut Domain: LPWSTR = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut SubStatus: NTSTATUS = STATUS_SUCCESS;

    result = 'main: {
        if GetTextboxText(hwndDlg, IDC_LOGON_USERNAME, &mut UserName as *mut LPWSTR) != 0
            && *UserName == 0
        {
            break 'main 0;
        }
        if GetTextboxText(hwndDlg, IDC_LOGON_DOMAIN, &mut Domain as *mut LPWSTR) != 0
            && *Domain == 0
        {
            break 'main 0;
        }
        if GetTextboxText(hwndDlg, IDC_LOGON_PASSWORD, &mut Password as *mut LPWSTR) == 0 {
            break 'main 0;
        }

        status = DoLoginTasks(pgContext, UserName, Domain, Password, &mut SubStatus as *mut NTSTATUS);
        if status == STATUS_LOGON_FAILURE {
            ResourceMessageBox(
                pgContext,
                hwndDlg,
                MB_OK | MB_ICONEXCLAMATION,
                IDS_LOGONTITLE,
                IDS_LOGONWRONGUSERORPWD,
            );
            break 'main 0;
        } else if status == STATUS_ACCOUNT_RESTRICTION {
            TRACE!(
                "DoLoginTasks failed! Status 0x%08lx  SubStatus 0x%08lx\n\0".as_ptr() as *const i8,
                status,
                SubStatus,
            );
            if SubStatus == STATUS_ACCOUNT_DISABLED {
                ResourceMessageBox(
                    pgContext,
                    hwndDlg,
                    MB_OK | MB_ICONEXCLAMATION,
                    IDS_LOGONTITLE,
                    IDS_LOGONUSERDISABLED,
                );
                break 'main 0;
            } else if SubStatus == STATUS_ACCOUNT_LOCKED_OUT {
                TRACE!("Account locked!\n\0".as_ptr() as *const i8);
                ResourceMessageBox(
                    pgContext,
                    hwndDlg,
                    MB_OK | MB_ICONERROR,
                    IDS_LOGONTITLE,
                    IDS_ACCOUNTLOCKED,
                );
                break 'main 0;
            } else if SubStatus == STATUS_PASSWORD_MUST_CHANGE
                || SubStatus == STATUS_PASSWORD_EXPIRED
            {
                if SubStatus == STATUS_PASSWORD_MUST_CHANGE {
                    ResourceMessageBox(
                        pgContext,
                        hwndDlg,
                        MB_OK | MB_ICONSTOP,
                        IDS_LOGONTITLE,
                        IDS_PASSWORDMUSTCHANGE,
                    );
                } else {
                    ResourceMessageBox(
                        pgContext,
                        hwndDlg,
                        MB_OK | MB_ICONSTOP,
                        IDS_LOGONTITLE,
                        IDS_PASSWORDEXPIRED,
                    );
                }
                if OnChangePassword(hwndDlg, pgContext) == 0 {
                    break 'main 0;
                }
                status = DoLoginTasks(
                    pgContext,
                    (*pgContext).UserName,
                    (*pgContext).DomainName,
                    (*pgContext).Password,
                    &mut SubStatus as *mut NTSTATUS,
                );
                if NT_SUCCESS(status) == 0 {
                    TRACE!(
                        "Login after password change failed! (Status 0x%08lx)\n\0".as_ptr()
                            as *const i8,
                        status,
                    );
                    break 'main 0;
                }
            } else if SubStatus == STATUS_ACCOUNT_EXPIRED {
                ResourceMessageBox(
                    pgContext,
                    hwndDlg,
                    MB_OK | MB_ICONEXCLAMATION,
                    IDS_LOGONTITLE,
                    IDS_ACCOUNTEXPIRED,
                );
            } else if SubStatus == STATUS_INVALID_LOGON_HOURS {
                ResourceMessageBox(
                    pgContext,
                    hwndDlg,
                    MB_OK | MB_ICONERROR,
                    IDS_LOGONTITLE,
                    IDS_INVALIDLOGONHOURS,
                );
                break 'main 0;
            } else if SubStatus == STATUS_INVALID_WORKSTATION {
                ResourceMessageBox(
                    pgContext,
                    hwndDlg,
                    MB_OK | MB_ICONERROR,
                    IDS_LOGONTITLE,
                    IDS_INVALIDWORKSTATION,
                );
                break 'main 0;
            } else {
                TRACE!("Other error!\n\0".as_ptr() as *const i8);
                ResourceMessageBox(
                    pgContext,
                    hwndDlg,
                    MB_OK | MB_ICONERROR,
                    IDS_LOGONTITLE,
                    IDS_ACCOUNTRESTRICTION,
                );
                break 'main 0;
            }
        } else if NT_SUCCESS(status) == 0 {
            TRACE!(
                "DoLoginTasks failed! Status 0x%08lx\n\0".as_ptr() as *const i8,
                status,
            );
            break 'main 0;
        }

        if CreateProfile(pgContext, UserName, Domain, Password) == 0 {
            ERR!("Failed to create the profile!\n\0".as_ptr() as *const i8);
            break 'main 0;
        }

        SecureZeroMemory(
            (*pgContext).Password.as_mut_ptr() as *mut _,
            core::mem::size_of_val(&(*pgContext).Password),
        );
        wcscpy((*pgContext).Password.as_mut_ptr(), Password);

        break 'main 1;
    };

    (*pgContext).bAutoAdminLogon = 0;

    if !UserName.is_null() {
        HeapFree(GetProcessHeap(), 0, UserName as *mut _);
    }
    if !Password.is_null() {
        let pwdLen = (wcslen(Password) + 1) * core::mem::size_of::<u16>();
        SecureZeroMemory(Password as *mut _, pwdLen);
        HeapFree(GetProcessHeap(), 0, Password as *mut _);
    }
    if !Domain.is_null() {
        HeapFree(GetProcessHeap(), 0, Domain as *mut _);
    }

    result
}

unsafe fn SetDomainComboBox(
    hwndDomainComboBox: HWND,
    pgContext: PGINA_CONTEXT,
) {

    let mut szComputerName: [u16; MAX_COMPUTERNAME_LENGTH as usize + 1] =
        [0; MAX_COMPUTERNAME_LENGTH as usize + 1];
    let mut dwComputerNameLength: DWORD = szComputerName.len() as DWORD;
    let mut lIndex: isize = 0;
    let lFindIndex: isize;

    SendMessageW(hwndDomainComboBox, CB_RESETCONTENT, 0 as WPARAM, 0 as LPARAM);

    if GetComputerNameW(
        szComputerName.as_mut_ptr() as PWSTR,
        &mut dwComputerNameLength,
    ) != 0
    {
        lIndex = SendMessageW(
            hwndDomainComboBox,
            CB_ADDSTRING,
            0 as WPARAM,
            szComputerName.as_ptr() as LPARAM,
        );
    }

    if (*pgContext).DomainName[0] != 0 {
        lFindIndex = SendMessageW(
            hwndDomainComboBox,
            CB_FINDSTRINGEXACT,
            (-1isize) as WPARAM,
            core::ptr::addr_of!((*pgContext).DomainName[0]) as LPARAM,
        );
        if lFindIndex == CB_ERR as isize {
            lIndex = SendMessageW(
                hwndDomainComboBox,
                CB_ADDSTRING,
                0 as WPARAM,
                core::ptr::addr_of!((*pgContext).DomainName[0]) as LPARAM,
            );
        } else {
            lIndex = lFindIndex;
        }
    }

    SendMessageW(
        hwndDomainComboBox,
        CB_SETCURSEL,
        lIndex as WPARAM,
        0 as LPARAM,
    );
}
unsafe fn LogonDialogProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: usize,
    lParam: isize,
) -> isize {

    let mut pDlgData: PDLG_DATA = GetWindowLongPtrW(hwndDlg, GWLP_USERDATA) as PDLG_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pDlgData = DlgData_Create(hwndDlg, lParam as PGINA_CONTEXT);
            if pDlgData.is_null() {
                return 0;
            }
            DlgData_LoadBitmaps(pDlgData);
            SetWelcomeText(hwndDlg);

            let pgContext = (*pDlgData).pgContext;
            if (*pgContext).bAutoAdminLogon != 0 || (*pgContext).bDontDisplayLastUserName == 0 {
                SetDlgItemTextW(
                    hwndDlg,
                    IDC_LOGON_USERNAME,
                    (*pgContext).UserName as *const u16,
                );
            }
            if (*pgContext).bAutoAdminLogon != 0 {
                SetDlgItemTextW(
                    hwndDlg,
                    IDC_LOGON_PASSWORD,
                    (*pgContext).Password as *const u16,
                );
            }

            SetDomainComboBox(
                GetDlgItem(hwndDlg, IDC_LOGON_DOMAIN),
                pgContext,
            );

            if (*pgContext).bDisableCAD != 0 {
                EnableWindow(GetDlgItem(hwndDlg, IDCANCEL), 0);
            }
            if (*pgContext).bShutdownWithoutLogon == 0 {
                EnableWindow(GetDlgItem(hwndDlg, IDC_LOGON_SHUTDOWN), 0);
            }

            let focusId = if (*pgContext).bDontDisplayLastUserName != 0 {
                IDC_LOGON_USERNAME
            } else {
                IDC_LOGON_PASSWORD
            };
            SetFocus(GetDlgItem(hwndDlg, focusId));

            if (*pgContext).bAutoAdminLogon != 0 {
                PostMessageW(GetDlgItem(hwndDlg, IDOK), BM_CLICK, 0, 0);
            }

            return 0; // FALSE – focus changed
        }
        WM_PAINT => {
            if !pDlgData.is_null() && !(*pDlgData).hLogoBitmap.is_null() {
                let mut ps: PAINTSTRUCT = core::mem::zeroed();
                BeginPaint(hwndDlg, &mut ps);
                DrawStateW(
                    ps.hdc,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    (*pDlgData).hLogoBitmap as isize,
                    0,
                    0,
                    0,
                    0,
                    0,
                    DST_BITMAP,
                );
                EndPaint(hwndDlg, &ps);
            }
            return 1;
        }
        WM_DESTROY => {
            DlgData_Destroy(hwndDlg);
            return 1;
        }
        WM_COMMAND => {
            let command = (wParam & 0xFFFF) as u32;
            match command {
                IDOK => {
                    let pgContext = (*pDlgData).pgContext;
                    if DoLogon(hwndDlg, pgContext) != 0 {
                        EndDialog(hwndDlg, WLX_SAS_ACTION_LOGON);
                    } else {
                        SetDlgItemTextW(hwndDlg, IDC_LOGON_PASSWORD, core::ptr::null());
                        SendMessageW(
                            hwndDlg,
                            WM_NEXTDLGCTL,
                            GetDlgItem(hwndDlg, IDC_LOGON_PASSWORD) as usize,
                            1,
                        );
                    }
                    return 1;
                }
                IDCANCEL => {
                    EndDialog(hwndDlg, WLX_SAS_ACTION_NONE);
                    return 1;
                }
                IDC_LOGON_SHUTDOWN => {
                    let pgContext = (*pDlgData).pgContext;
                    if OnShutDown(
                        hwndDlg,
                        pgContext,
                        WLX_SHUTDOWN_STATE_DISCONNECT | WLX_SHUTDOWN_STATE_LOGOFF,
                    ) == IDOK
                    {
                        EndDialog(hwndDlg, (*pgContext).nShutdownAction);
                    }
                    return 1;
                }
                _ => {}
            }
        }
        _ => {}
    }

    0
}
unsafe fn LegalNoticeDialogProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> isize {

    const WM_INITDIALOG: u32 = 0x0110;
    const WM_COMMAND: u32 = 0x0111;
    const IDOK: u16 = 1;
    const IDCANCEL: u16 = 2;

    match uMsg {
        WM_INITDIALOG => {
            let pLegalNotice = lParam as PLEGALNOTICEDATA;
            SetWindowTextW(hwndDlg, (*pLegalNotice).pszCaption);
            SetDlgItemTextW(hwndDlg, IDC_LEGALNOTICE_TEXT, (*pLegalNotice).pszText);
            return 1isize;
        }
        WM_COMMAND => {
            match (wParam as usize & 0xFFFF) as u16 {
                IDOK | IDCANCEL => {
                    EndDialog(hwndDlg, 0);
                    return 1isize;
                }
                _ => {}
            }
        }
        _ => {}
    }

    0isize
}

unsafe fn GUILoggedOutSAS(pgContext: PGINA_CONTEXT) -> i32 {
    let mut LegalNotice = LEGALNOTICEDATA {
        pszCaption: core::ptr::null_mut(),
        pszText: core::ptr::null_mut(),
    };
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut rc: i32;
    let result: i32;

    TRACE!(b"GUILoggedOutSAS()\n\0".as_ptr() as *const i8);

    let subkey: [u16; 54] = [
        0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045,
        0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F,
        0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F,
        0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C, 0x0043, 0x0075,
        0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
        0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0057, 0x0069, 0x006E,
        0x006C, 0x006F, 0x0067, 0x006F, 0x006E, 0x0000,
    ];
    rc = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        subkey.as_ptr() as *const u16,
        0,
        KEY_QUERY_VALUE,
        &mut hKey as *mut HKEY,
    );
    if rc == ERROR_SUCCESS {
        let caption_val: [u16; 19] = [
            0x004C, 0x0065, 0x0067, 0x0061, 0x006C, 0x004E, 0x006F, 0x0074,
            0x0069, 0x0063, 0x0065, 0x0043, 0x0061, 0x0070, 0x0074, 0x0069,
            0x006F, 0x006E, 0x0000,
        ];
        ReadRegSzValue(
            hKey,
            caption_val.as_ptr() as *const u16,
            core::ptr::addr_of_mut!(LegalNotice.pszCaption),
        );

        let text_val: [u16; 16] = [
            0x004C, 0x0065, 0x0067, 0x0061, 0x006C, 0x004E, 0x006F, 0x0074,
            0x0069, 0x0063, 0x0065, 0x0054, 0x0065, 0x0078, 0x0074, 0x0000,
        ];
        ReadRegSzValue(
            hKey,
            text_val.as_ptr() as *const u16,
            core::ptr::addr_of_mut!(LegalNotice.pszText),
        );

        RegCloseKey(hKey);
    }

    if !LegalNotice.pszCaption.is_null()
        && *LegalNotice.pszCaption != 0
        && !LegalNotice.pszText.is_null()
        && *LegalNotice.pszText != 0
    {
        (*(*pgContext).pWlxFuncs).WlxDialogBoxParam(
            (*pgContext).hWlx,
            (*pgContext).hDllInstance,
            MAKEINTRESOURCEW(IDD_LEGALNOTICE),
            GetDesktopWindow(),
            LegalNoticeDialogProc,
            &LegalNotice as *const LEGALNOTICEDATA as LPARAM,
        );
    }

    if !LegalNotice.pszCaption.is_null() {
        HeapFree(GetProcessHeap(), 0, LegalNotice.pszCaption as LPVOID);
    }
    if !LegalNotice.pszText.is_null() {
        HeapFree(GetProcessHeap(), 0, LegalNotice.pszText as LPVOID);
    }

    result = (*(*pgContext).pWlxFuncs).WlxDialogBoxParam(
        (*pgContext).hWlx,
        (*pgContext).hDllInstance,
        MAKEINTRESOURCEW(IDD_LOGON),
        GetDesktopWindow(),
        LogonDialogProc,
        pgContext as LPARAM,
    );
    if result >= WLX_SAS_ACTION_LOGON && result <= WLX_SAS_ACTION_SWITCH_CONSOLE {
        WARN!(b"WlxLoggedOutSAS() returns 0x%x\n\0".as_ptr() as *const i8, result);
        return result;
    }

    WARN!(b"WlxDialogBoxParam() failed (0x%x)\n\0".as_ptr() as *const i8, result);
    return WLX_SAS_ACTION_NONE;
}

unsafe fn SetLockMessage(
    hwnd: HWND,
    nDlgItem: i32,
    pgContext: PGINA_CONTEXT,
) {
    let mut buffer1: [u16; 256] = [0u16; 256];
    let mut buffer2: [u16; 256] = [0u16; 256];
    let mut buffer3: [u16; 512] = [0u16; 512];

    LoadStringW(
        (*pgContext).hDllInstance,
        IDS_LOCKMSG,
        buffer1.as_mut_ptr(),
        256,
    );

    let format: &[u16] = &[0x25, 0x73, 0x5c, 0x25, 0x73, 0]; // "%s\%s"
    wsprintfW(
        buffer2.as_mut_ptr(),
        format.as_ptr(),
        (*pgContext).DomainName,
        (*pgContext).UserName,
    );

    wsprintfW(
        buffer3.as_mut_ptr(),
        buffer1.as_ptr(),
        buffer2.as_ptr(),
    );

    SetDlgItemTextW(hwnd, nDlgItem, buffer3.as_ptr());
}

unsafe fn DoUnlock(
    hwndDlg: HWND,
    pgContext: *mut _GINA_CONTEXT,
    action: *mut i32,
) -> i32 {
    let mut buffer1: [u16; 256] = [0u16; 256];
    let mut buffer2: [u16; 256] = [0u16; 256];
    let mut user_name: LPWSTR = core::ptr::null_mut();
    let mut password: LPWSTR = core::ptr::null_mut();
    let mut res: i32 = 0;

    if GetTextboxText(hwndDlg, IDC_UNLOCK_USERNAME, &mut user_name) != 0
        && *user_name == 0u16
    {
        HeapFree(GetProcessHeap(), 0, user_name as _);
        return 0;
    }

    if GetTextboxText(hwndDlg, IDC_UNLOCK_PASSWORD, &mut password) != 0 {
        if !user_name.is_null()
            && !password.is_null()
            && wcscmp(user_name, (*pgContext).UserName) == 0
            && wcscmp(password, (*pgContext).Password) == 0
        {
            *action = WLX_SAS_ACTION_UNLOCK_WKSTA;
            res = 1;
        } else if wcscmp(user_name, (*pgContext).UserName) == 0
            && wcscmp(password, (*pgContext).Password) != 0
        {
            LoadStringW(
                (*pgContext).hDllInstance,
                IDS_LOCKEDWRONGPASSWORD,
                buffer2.as_mut_ptr(),
                256,
            );
            LoadStringW(
                (*pgContext).hDllInstance,
                IDS_COMPUTERLOCKED,
                buffer1.as_mut_ptr(),
                256,
            );
            MessageBoxW(
                hwndDlg,
                buffer2.as_mut_ptr(),
                buffer1.as_mut_ptr(),
                MB_OK | MB_ICONERROR,
            );
        } else {
            if DoAdminUnlock(
                pgContext,
                user_name,
                core::ptr::null_mut(),
                password,
            ) != 0
            {
                *action = WLX_SAS_ACTION_UNLOCK_WKSTA;
                res = 1;
            } else {
                LoadStringW(
                    (*pgContext).hDllInstance,
                    IDS_LOCKEDWRONGUSER,
                    buffer1.as_mut_ptr(),
                    256,
                );
                wsprintfW(
                    buffer2.as_mut_ptr(),
                    buffer1.as_ptr(),
                    (*pgContext).DomainName,
                    (*pgContext).UserName,
                );
                LoadStringW(
                    (*pgContext).hDllInstance,
                    IDS_COMPUTERLOCKED,
                    buffer1.as_mut_ptr(),
                    256,
                );
                MessageBoxW(
                    hwndDlg,
                    buffer2.as_mut_ptr(),
                    buffer1.as_mut_ptr(),
                    MB_OK | MB_ICONERROR,
                );
            }
        }
    }

    if !user_name.is_null() {
        HeapFree(GetProcessHeap(), 0, user_name as _);
    }

    if !password.is_null() {
        let pwd_len: usize = (wcslen(password) + 1) * core::mem::size_of::<u16>();
        core::ptr::write_bytes(password as *mut u8, 0, pwd_len);
        HeapFree(GetProcessHeap(), 0, password as _);
    }

    res
}
unsafe fn UnlockDialogProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pDlgData: PDLG_DATA = GetWindowLongPtrW(hwndDlg, GWLP_USERDATA) as PDLG_DATA;

    match uMsg {
        WM_INITDIALOG => {
            let pgContext = lParam as PGINA_CONTEXT;
            pDlgData = DlgData_Create(hwndDlg, pgContext);
            if pDlgData.is_null() {
                return 0;
            }

            DlgData_LoadBitmaps(pDlgData);
            SetWelcomeText(hwndDlg);
            SetLockMessage(hwndDlg, IDC_UNLOCK_MESSAGE, (*pDlgData).pgContext);
            SetDlgItemTextW(
                hwndDlg,
                IDC_UNLOCK_USERNAME,
                (*(*pDlgData).pgContext).UserName,
            );

            if (*(*pDlgData).pgContext).bDisableCAD != 0 {
                EnableWindow(GetDlgItem(hwndDlg, IDCANCEL), 0);
            }

            SetFocus(GetDlgItem(hwndDlg, IDC_UNLOCK_PASSWORD));
            return 0;
        }

        WM_PAINT => {
            if !pDlgData.is_null() && !(*pDlgData).hLogoBitmap.is_null() {
                let mut ps = core::mem::zeroed::<PAINTSTRUCT>();
                BeginPaint(hwndDlg, &mut ps);
                DrawStateW(
                    ps.hdc,
                    core::ptr::null_mut(),
                    None,
                    (*pDlgData).hLogoBitmap as LPARAM,
                    0,
                    0,
                    0,
                    0,
                    0,
                    DST_BITMAP,
                );
                EndPaint(hwndDlg, &ps);
            }
            return 1;
        }

        WM_DESTROY => {
            DlgData_Destroy(hwndDlg);
            return 1;
        }

        WM_COMMAND => {
            match LOWORD(wParam) {
                IDOK => {
                    let mut result: i32 = WLX_SAS_ACTION_NONE;
                    if DoUnlock(hwndDlg, (*pDlgData).pgContext, &mut result) != 0 {
                        EndDialog(hwndDlg, result as INT_PTR);
                    } else {
                        SetDlgItemTextW(
                            hwndDlg,
                            IDC_UNLOCK_PASSWORD,
                            core::ptr::null_mut(),
                        );
                        SendMessageW(
                            hwndDlg,
                            WM_NEXTDLGCTL,
                            GetDlgItem(hwndDlg, IDC_UNLOCK_PASSWORD) as WPARAM,
                            1,
                        );
                    }
                    return 1;
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, WLX_SAS_ACTION_NONE as INT_PTR);
                    return 1;
                }

                _ => {}
            }
        }

        _ => {}
    }

    0
}

unsafe fn GUILockedSAS(pgContext: *mut _GINA_CONTEXT) -> i32 {
    TRACE!(b"GUILockedSAS()\n\0".as_ptr().cast());
    let pWlxFuncs = (*pgContext).pWlxFuncs;
    let result = ((*pWlxFuncs).WlxDialogBoxParam)(
        (*pgContext).hWlx,
        (*pgContext).hDllInstance,
        (IDD_UNLOCK as u16 as usize) as PWSTR,
        GetDesktopWindow(),
        UnlockDialogProc,
        pgContext as LPARAM,
    );
    if result >= WLX_SAS_ACTION_LOGON && result <= WLX_SAS_ACTION_SWITCH_CONSOLE {
        WARN!(
            b"GUILockedSAS() returns 0x%x\n\0".as_ptr().cast(),
            result,
        );
        return result;
    }
    WARN!(
        b"GUILockedSAS() failed (0x%x)\n\0".as_ptr().cast(),
        result,
    );
    WLX_SAS_ACTION_NONE
}

unsafe fn LockedDialogProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> isize {
    let pDlgData = GetWindowLongPtrW(hwndDlg, GWLP_USERDATA) as PDLG_DATA;

    match uMsg {
        WM_INITDIALOG => {
            let pDlgData = DlgData_Create(hwndDlg, lParam as PGINA_CONTEXT);
            if pDlgData.is_null() {
                return 0;
            }
            DlgData_LoadBitmaps(pDlgData);
            SetWelcomeText(hwndDlg);
            SetLockMessage(hwndDlg, IDC_LOCKED_MESSAGE, (*pDlgData).pgContext);
            return 1;
        }
        WM_PAINT => {
            if !pDlgData.is_null() && !(*pDlgData).hLogoBitmap.is_null() {
                let mut ps: PAINTSTRUCT = core::mem::zeroed();
                BeginPaint(hwndDlg, &mut ps);
                DrawStateW(
                    ps.hdc,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    (*pDlgData).hLogoBitmap as LPARAM,
                    0,
                    0,
                    0,
                    0,
                    0,
                    DST_BITMAP,
                );
                EndPaint(hwndDlg, &ps);
            }
            return 1;
        }
        WM_DESTROY => {
            DlgData_Destroy(hwndDlg);
            return 1;
        }
        _ => return 0,
    }
}

unsafe fn GUIDisplayLockedNotice(pgContext: *mut _GINA_CONTEXT) {
    let func = (*((*pgContext).pWlxFuncs)).WlxDialogBoxParam;
    func(
        (*pgContext).hWlx,
        (*pgContext).hDllInstance,
        MAKEINTRESOURCEW(IDD_LOCKED),
        GetDesktopWindow(),
        LockedDialogProc as unsafe extern "system" fn(_, _, _, _) -> _,
        pgContext as isize,
    );
}
