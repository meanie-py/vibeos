use crate::internal::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::dialog::CreateFontIndirectW;
use crate::windows::message::GlobalAlloc;
// The crate's own functions: the definition is the declaration.
use crate::windows::font::DrawTextW;
use crate::ntwrapper::*;
use win32k_sys::win32k::*;
use core::ffi::c_void;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// SetWindowLongPtrW is a winuser.h macro for SetWindowLongW on x86; class.rs only
// defines a real (isize-typed) SetWindowLongPtrW for non-x86. cfg-split.
#[cfg(not(target_arch = "x86"))]
#[inline]
unsafe fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR {
    crate::windows::class::SetWindowLongPtrW(hWnd, nIndex, dwNewLong as isize) as LONG_PTR
}
#[cfg(target_arch = "x86")]
#[inline]
unsafe fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR {
    crate::windows::class::SetWindowLongW(hWnd, nIndex, dwNewLong as core::ffi::c_long) as LONG_PTR
}

// ─────────────────────────────────────────────────────────────────────────────
// Local externs not provided by the substrate.
// ─────────────────────────────────────────────────────────────────────────────

// CRT + WINAPIV (cdecl). wsprintfW is __cdecl (variadic) — `_wsprintfW`, no @N.
extern "C" {
    fn wcslen(s: *const u16) -> usize;
    fn wsprintfW(buf: LPWSTR, fmt: LPCWSTR, ...) -> INT;
}

// gdi32 / user32 / kernel32 public WINAPI (stdcall) exports — resolve via the
// respective import libs (gdi32/kernel32) or within user32.dll (still-C TUs).
extern "system" {
    fn DeleteObject(ho: *mut core::ffi::c_void) -> BOOL;
    fn DialogBoxIndirectParamW( hInstance: HINSTANCE, hDialogTemplate: *mut DLGTEMPLATE, hWndParent: HWND, lpDialogFunc: DLGPROC, dwInitParam: LPARAM, ) -> INT_PTR;
    fn GetDialogBaseUnits() -> core::ffi::c_long;
    fn GetLastError() -> DWORD;
    fn GetSystemMetrics(nIndex: INT) -> INT;
    fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON;
    fn MulDiv(nNumber: INT, nNumerator: INT, nDenominator: INT) -> INT;
    fn ReleaseDC(hWnd: HWND, hDC: HDC) -> INT;
    fn SelectObject(hdc: HDC, h: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn SystemParametersInfoW(uiAction: UINT, uiParam: UINT, pvParam: *mut core::ffi::c_void, fWinIni: UINT) -> BOOL;

    fn LoadStringW(hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: core::ffi::c_int) -> core::ffi::c_int;

    // ntdll export (NtRaiseHardError is NTAPI).
    fn NtRaiseHardError(
        ErrorStatus: NTSTATUS,
        NumberOfParameters: ULONG,
        UnicodeStringParameterMask: ULONG,
        Parameters: *mut ULONG_PTR,
        ValidResponseOptions: ULONG,
        Response: PULONG,
    ) -> NTSTATUS;

    // user32 public exports living in still-C TUs (resolve within user32.dll).
    fn GetPropW(hWnd: HWND, lpString: LPCWSTR) -> HANDLE;
    fn SetPropW(hWnd: HWND, lpString: LPCWSTR, hData: HANDLE) -> BOOL;
    fn GetDlgItem(hDlg: HWND, nIDDlgItem: INT) -> HWND;
    fn GetDlgItemTextW(hDlg: HWND, nIDDlgItem: INT, lpString: LPWSTR, cchMax: INT) -> UINT;
    fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SendDlgItemMessageW(hDlg: HWND, nIDDlgItem: INT, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn EndDialog(hDlg: HWND, nResult: INT_PTR) -> BOOL;
    fn GetSystemMenu(hWnd: HWND, bRevert: BOOL) -> HMENU;
    fn SetWindowContextHelpId(hWnd: HWND, dwContextHelpId: DWORD) -> BOOL;
    fn GetWindowContextHelpId(hWnd: HWND) -> DWORD;
    fn GetWindow(hWnd: HWND, uCmd: UINT) -> HWND;
    fn GetWindowTextW(hWnd: HWND, lpString: LPWSTR, nMaxCount: INT) -> INT;
    fn GetWindowTextLengthW(hWnd: HWND) -> INT;
    fn GetCursorPos(lpPoint: LPPOINT) -> BOOL;
    fn NotifyWinEvent(event: DWORD, hwnd: HWND, idObject: LONG, idChild: LONG);
    fn lstrlenW(lpString: LPCWSTR) -> INT;

    // kernel32 (winbase.h) public exports.
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn GlobalLock(hMem: HGLOBAL) -> LPVOID;
    fn GlobalUnlock(hMem: HGLOBAL) -> BOOL;
    fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;

    // user32 clipboard (winuser.h) public exports.
    fn OpenClipboard(hWndNewOwner: HWND) -> BOOL;
    fn SetClipboardData(uFormat: UINT, hMem: HANDLE) -> HANDLE;
}

// user32-internal globals living in still-C TUs (resolve within user32.dll).
extern "C" {
    static User32Instance: HINSTANCE; // dllmain.c / user32p.h
}

// ─── Local file-private helpers (per-file convention in this crate) ───────────

// strsafe.h RtlStringCchCopyNW (inline in C; no ntdll export) — bounded copy + NUL-terminate.
#[inline]
unsafe fn RtlStringCchCopyNW(dest: PWSTR, cch_dest: usize, src: PCWSTR, cch_src: usize) -> i32 {
    if cch_dest == 0 { return -2147024809; } // STRSAFE_E_INVALID_PARAMETER (0x80070057)
    let mut n = 0usize;
    while n < cch_src && n < cch_dest - 1 && *src.add(n) != 0 {
        *dest.add(n) = *src.add(n);
        n += 1;
    }
    *dest.add(n) = 0;
    0 // S_OK
}

// strlenW(s) == wcslen(s) (wine/unicode.h).
#[inline]
unsafe fn strlenW(s: *const u16) -> i32 {
    wcslen(s) as i32
}

// IS_INTRESOURCE — wine/winuser.h macro: a pointer with the high bits clear
// (value < 0x10000) is an int-resource. Faithful to the C macro.
#[inline]
unsafe fn IS_INTRESOURCE(p: *const c_void) -> bool {
    (p as usize) >> 16 == 0
}

// ntdef.h NT_SUCCESS(s) := s >= 0.
#[inline]
fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// winnt.h PtrToUlong — truncating pointer→u32.
#[inline]
fn PtrToUlong(p: *mut c_void) -> u32 {
    (p as usize) as u32
}

// L"..." wide-string literal → a const-built, statically-stored UTF-16 array
// (ASCII-only literals, which is all this file uses). Returns a stable pointer.
macro_rules! ucs2 {
    ($s:literal) => {{
        const B: &[u8] = $s.as_bytes();
        const N: usize = B.len() + 1;
        static W: [u16; N] = {
            let mut a = [0u16; N];
            let mut i = 0;
            while i < B.len() {
                a[i] = B[i] as u16;
                i += 1;
            }
            a
        };
        &W
    }};
}

// ---- file-scope preamble (translated) ----
pub const MSGBOX_IDICON: u32 = 1088;

pub const MSGBOX_IDTEXT: u32 = 0xffff;

pub const BTN_CX: i32 = 75;

pub const BTN_CY: i32 = 23;

pub const MSGBOXEX_SPACING: i32 = 16;

pub const MSGBOXEX_BUTTONSPACING: i32 = 6;

pub const MSGBOXEX_MARGIN: i32 = 12;

pub const MSGBOXEX_MAXBTNSTR: i32 = 32;

pub const MSGBOXEX_MAXBTNS: usize = 4;

// user32-private string-resource IDs (win32ss/user/user32/include/resource.h) —
// not in the win32k-sys bindgen surface; reproduced here verbatim.
pub const IDS_OK: u32 = 800;
pub const IDS_CANCEL: u32 = 801;
pub const IDS_ABORT: u32 = 802;
pub const IDS_RETRY: u32 = 803;
pub const IDS_IGNORE: u32 = 804;
pub const IDS_YES: u32 = 805;
pub const IDS_NO: u32 = 806;
pub const IDS_CLOSE: u32 = 807;
pub const IDS_HELP: u32 = 808;
pub const IDS_TRYAGAIN: u32 = 809;
pub const IDS_CONTINUE: u32 = 810;
pub const IDS_ERROR: u32 = 2; // resource.h IDS_ERROR

// winbase.h GHND = GMEM_MOVEABLE | GMEM_ZEROINIT (0x0042); ntstatus.h
// STATUS_SERVICE_NOTIFICATION — not in the bindgen surface.
pub const GHND: UINT = 0x0042;
pub const STATUS_SERVICE_NOTIFICATION: NTSTATUS = 0x40000018u32 as NTSTATUS;

#[repr(C)]
pub struct _MSGBTNINFO {
    pub btnCnt: u32,
    pub btnIdx: [i32; MSGBOXEX_MAXBTNS],
    pub btnIds: [u32; MSGBOXEX_MAXBTNS],
}
pub type MSGBTNINFO = _MSGBTNINFO;
pub type PMSGBTNINFO = *mut _MSGBTNINFO;

pub static MsgBtnInfo: [MSGBTNINFO; 7] = [
    /* MB_OK (0) */
    _MSGBTNINFO { btnCnt: 1, btnIdx: [IDOK as i32, 0, 0, 0], btnIds: [IDS_OK, 0, 0, 0] },
    /* MB_OKCANCEL (1) */
    _MSGBTNINFO { btnCnt: 2, btnIdx: [IDOK as i32, IDCANCEL as i32, 0, 0], btnIds: [IDS_OK, IDS_CANCEL, 0, 0] },
    /* MB_ABORTRETRYIGNORE (2) */
    _MSGBTNINFO { btnCnt: 3, btnIdx: [IDABORT as i32, IDRETRY as i32, IDIGNORE as i32, 0], btnIds: [IDS_ABORT, IDS_RETRY, IDS_IGNORE, 0] },
    /* MB_YESNOCANCEL (3) */
    _MSGBTNINFO { btnCnt: 3, btnIdx: [IDYES as i32, IDNO as i32, IDCANCEL as i32, 0], btnIds: [IDS_YES, IDS_NO, IDS_CANCEL, 0] },
    /* MB_YESNO (4) */
    _MSGBTNINFO { btnCnt: 2, btnIdx: [IDYES as i32, IDNO as i32, 0, 0], btnIds: [IDS_YES, IDS_NO, 0, 0] },
    /* MB_RETRYCANCEL (5) */
    _MSGBTNINFO { btnCnt: 2, btnIdx: [IDRETRY as i32, IDCANCEL as i32, 0, 0], btnIds: [IDS_RETRY, IDS_CANCEL, 0, 0] },
    /* MB_CANCELTRYCONTINUE (6) */
    _MSGBTNINFO { btnCnt: 3, btnIdx: [IDCANCEL as i32, IDTRYAGAIN as i32, IDCONTINUE as i32, 0], btnIds: [IDS_CANCEL, IDS_TRYAGAIN, IDS_CONTINUE, 0] },
];

unsafe fn LoadAllocStringW(
    hInstance: HINSTANCE,
    uID: UINT,
    pString: *mut PWSTR,
    pDefaultString: PCWSTR,
) -> UINT {
    let mut Length: UINT;
    let mut pStr: PCWSTR;

    pStr = core::ptr::null_mut();

    Length = LoadStringW(hInstance, uID, core::ptr::addr_of_mut!(pStr) as LPWSTR, 0) as UINT;
    if Length == 0 {
        if pDefaultString.is_null() {
            *pString = core::ptr::null_mut();
            return 0;
        }

        pStr = pDefaultString;
        Length = wcslen(pStr) as UINT;
    }

    *pString = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        ((Length + 1) as usize * core::mem::size_of::<u16>()) as SIZE_T,
    ) as PWSTR;
    if (*pString).is_null() {
        return 0;
    }

    RtlStringCchCopyNW(*pString, (Length + 1) as usize, pStr, Length as usize);
    Length
}

unsafe fn MessageBoxTextToClipboard(DialogWindow: HWND) {
    let hwndText: HWND;
    let mbd: PMSGBOXDATA;
    let cchTotal: i32;
    let cchTitle: i32;
    let cchText: i32;
    let mut cchButton: i32;
    let mut i: i32;
    let mut n: i32;
    let cchBuffer: i32;
    let pszBuffer: LPWSTR;
    let mut pszBufferPos: LPWSTR;
    let pMessageBoxText: LPWSTR;
    let pszTitle: LPWSTR;
    let pszText: LPWSTR;
    let mut pszButton: LPWSTR;
    let mut szButton: [u16; MSGBOXEX_MAXBTNSTR as usize] = [0; MSGBOXEX_MAXBTNSTR as usize];
    let hGlobal: HGLOBAL;

    static szLine: [u16; 29] = [
        b'-' as u16, b'-' as u16, b'-' as u16, b'-' as u16, b'-' as u16, b'-' as u16,
        b'-' as u16, b'-' as u16, b'-' as u16, b'-' as u16, b'-' as u16, b'-' as u16,
        b'-' as u16, b'-' as u16, b'-' as u16, b'-' as u16, b'-' as u16, b'-' as u16,
        b'-' as u16, b'-' as u16, b'-' as u16, b'-' as u16, b'-' as u16, b'-' as u16,
        b'-' as u16, b'-' as u16, b'-' as u16, b'\r' as u16, b'\n' as u16,
    ];

    let ros_msgbox: [u16; 11] = [
        b'R' as u16, b'O' as u16, b'S' as u16, b'_' as u16, b'M' as u16, b'S' as u16,
        b'G' as u16, b'B' as u16, b'O' as u16, b'X' as u16, 0,
    ];

    mbd = GetPropW(DialogWindow, ros_msgbox.as_ptr()) as PMSGBOXDATA;
    hwndText = GetDlgItem(DialogWindow, MSGBOX_IDTEXT as i32);
    cchTitle = GetWindowTextLengthW(DialogWindow) + 1;
    cchText = GetWindowTextLengthW(hwndText) + 1;

    if mbd.is_null() {
        return;
    }

    pMessageBoxText = RtlAllocateHeap(
        GetProcessHeap(),
        0,
        (((cchTitle + cchText) as usize) * core::mem::size_of::<u16>()) as SIZE_T,
    ) as LPWSTR;

    if pMessageBoxText.is_null() {
        RtlFreeHeap(GetProcessHeap(), 0, pMessageBoxText as *mut _);
        return;
    }

    pszTitle = pMessageBoxText;
    pszText = pMessageBoxText.offset(cchTitle as isize);

    if GetWindowTextW(DialogWindow, pszTitle, cchTitle) == 0
        || GetWindowTextW(hwndText, pszText, cchText) == 0
    {
        RtlFreeHeap(GetProcessHeap(), 0, pMessageBoxText as *mut _);
        return;
    }

    cchTotal = 6 + cchTitle + cchText + (lstrlenW(szLine.as_ptr()) * 4)
        + ((*mbd).dwButtons as i32 * MSGBOXEX_MAXBTNSTR + 3);

    hGlobal = GlobalAlloc(GHND, ((cchTotal as usize) * core::mem::size_of::<u16>()) as SIZE_T);

    pszBuffer = GlobalLock(hGlobal) as LPWSTR;

    if pszBuffer.is_null() {
        RtlFreeHeap(GetProcessHeap(), 0, pMessageBoxText as *mut _);
        GlobalFree(hGlobal);
        return;
    }

    let fmt1: [u16; 13] = [
        b'%' as u16, b's' as u16, b'%' as u16, b's' as u16, b'\r' as u16, b'\n' as u16,
        b'%' as u16, b's' as u16, b'%' as u16, b's' as u16, b'\r' as u16, b'\n' as u16,
        b'%' as u16,
    ];
    let fmt1_full: [u16; 14] = [
        b'%' as u16, b's' as u16, b'%' as u16, b's' as u16, b'\r' as u16, b'\n' as u16,
        b'%' as u16, b's' as u16, b'%' as u16, b's' as u16, b'\r' as u16, b'\n' as u16,
        b'%' as u16, b's' as u16,
    ];
    let _ = fmt1;
    let fmt1_z: [u16; 15] = {
        let mut a = [0u16; 15];
        a[..14].copy_from_slice(&fmt1_full);
        a
    };

    cchBuffer = wsprintfW(
        pszBuffer,
        fmt1_z.as_ptr(),
        szLine.as_ptr(),
        pszTitle,
        szLine.as_ptr(),
        pszText,
        szLine.as_ptr(),
    );
    pszBufferPos = pszBuffer.offset(cchBuffer as isize);

    i = 0;
    while i < (*mbd).dwButtons as INT {
        GetDlgItemTextW(
            DialogWindow,
            *(*mbd).pidButton.offset(i as isize),
            szButton.as_mut_ptr(),
            MSGBOXEX_MAXBTNSTR as i32,
        );

        cchButton = strlenW(szButton.as_ptr());
        pszButton = szButton.as_mut_ptr();

        if szButton[0] == b'&' as u16 {
            pszButton = pszButton.offset(1);
            cchButton = cchButton - 1;
        }

        n = 0;
        while n < cchButton {
            *pszBufferPos = *pszButton.offset(n as isize);
            pszBufferPos = pszBufferPos.offset(1);
            n += 1;
        }

        *pszBufferPos = b' ' as u16;
        pszBufferPos = pszBufferPos.offset(1);
        *pszBufferPos = b' ' as u16;
        pszBufferPos = pszBufferPos.offset(1);
        *pszBufferPos = b' ' as u16;
        pszBufferPos = pszBufferPos.offset(1);

        i += 1;
    }

    let fmt2: [u16; 5] = [
        b'\r' as u16, b'\n' as u16, b'%' as u16, b's' as u16, 0,
    ];
    wsprintfW(pszBufferPos, fmt2.as_ptr(), szLine.as_ptr());

    GlobalUnlock(hGlobal);

    if OpenClipboard(DialogWindow) != 0 {
        EmptyClipboard();
        SetClipboardData(CF_UNICODETEXT as u32, hGlobal);
        CloseClipboard();
    } else {
        GlobalFree(hGlobal);
    }
    RtlFreeHeap(GetProcessHeap(), 0, pMessageBoxText as *mut _);
}

unsafe extern "system" fn MessageBoxProc(
    hwnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    // C declares `PMSGBOXDATA mbd;` at function scope; each arm shadows it with
    // its own GetPropW lookup. Init to null so the trailing `let _ = mbd;`
    // (translator scaffolding) is not a use-of-uninitialized.
    let mbd: PMSGBOXDATA = core::ptr::null_mut();

    match message {
        WM_INITDIALOG => {
            let Alert: i32;

            let mbd = lParam as PMSGBOXDATA;

            SetWindowLongPtrW(hwnd, GWLP_USERDATA as i32, mbd as LONG_PTR);
            NtUserxSetMessageBox(hwnd);

            if GetPropW(hwnd, ucs2!("ROS_MSGBOX").as_ptr()).is_null() {
                SetPropW(hwnd, ucs2!("ROS_MSGBOX").as_ptr(), lParam as HANDLE);

                if (*mbd).mbp.dwContextHelpId != 0 {
                    SetWindowContextHelpId(hwnd, (*mbd).mbp.dwContextHelpId as DWORD);
                }

                if !(*mbd).mbp.lpszIcon.is_null() {
                    SendDlgItemMessageW(hwnd, 1088, STM_SETICON, (*mbd).mbp.lpszIcon as HICON as WPARAM, 0);
                    Alert = ALERT_SYSTEM_WARNING as i32;
                } else {
                    match (*mbd).mbp.dwStyle & MB_ICONMASK {
                        x if x == MB_ICONWARNING => {
                            Alert = ALERT_SYSTEM_WARNING as i32;
                        }
                        x if x == MB_ICONERROR => {
                            Alert = ALERT_SYSTEM_ERROR as i32;
                        }
                        x if x == MB_ICONQUESTION => {
                            Alert = ALERT_SYSTEM_QUERY as i32;
                        }
                        _ => {
                            Alert = ALERT_SYSTEM_INFORMATIONAL as i32;
                        }
                    }
                }

                NotifyWinEvent(EVENT_SYSTEM_ALERT, hwnd, OBJID_ALERT as LONG, Alert);

                if (*mbd).uCancelId == 0 {
                    let hSysMenu = GetSystemMenu(hwnd, 0u8 as _);
                    if !hSysMenu.is_null() {
                        DeleteMenu(hSysMenu, SC_CLOSE, MF_BYCOMMAND);
                    }
                }

                if (*mbd).dwButtons > 0 {
                    SetFocus(GetDlgItem(hwnd, *(*mbd).pidButton.offset((*mbd).uDefButton as isize)));
                }

                if (*mbd).dwTimeout != 0 && (*mbd).dwTimeout != (-1i32) as UINT {
                    SetTimer(hwnd, 0, (*mbd).dwTimeout, None);
                }
            }
            return 0;
        }

        WM_COMMAND => {
            let mut i: UINT;
            let mut iCtrlId: INT_PTR = LOWORD(wParam as _) as INT_PTR;

            match iCtrlId as u32 {
                x if x == IDOK || x == IDCANCEL => {
                    'fallthrough: {
                        if iCtrlId as u32 == IDCANCEL {
                            let mbd = GetPropW(hwnd, ucs2!("ROS_MSGBOX").as_ptr()) as PMSGBOXDATA;
                            if mbd.is_null() {
                                return 0;
                            }

                            if (*mbd).uCancelId == 0 {
                                return 1;
                            }
                            iCtrlId = (*mbd).uCancelId as INT_PTR;
                        }
                        if GetDlgItem(hwnd, iCtrlId as i32).is_null() {
                            return 0;
                        }
                        break 'fallthrough;
                    }
                    EndDialog(hwnd, iCtrlId as INT_PTR);
                    return 1;
                }
                x if x == IDABORT || x == IDRETRY || x == IDIGNORE
                    || x == IDYES || x == IDNO || x == IDTRYAGAIN || x == IDCONTINUE =>
                {
                    EndDialog(hwnd, iCtrlId as INT_PTR);
                    return 1;
                }
                x if x == IDCLOSE => {
                    return 0;
                }
                x if x == IDHELP => {
                    let mut hi: HELPINFO = core::mem::zeroed();
                    hi.cbSize = core::mem::size_of::<HELPINFO>() as UINT;
                    hi.iContextType = HELPINFO_WINDOW as INT;
                    hi.iCtrlId = iCtrlId as i32;
                    hi.hItemHandle = lParam as HANDLE;
                    hi.dwContextId = 0;
                    GetCursorPos(core::ptr::addr_of_mut!(hi.MousePos) as LPPOINT);
                    SendMessageW(hwnd, WM_HELP, 0, core::ptr::addr_of_mut!(hi) as LPARAM);
                    return 1;
                }
                _ => {}
            }

            let mbd = GetPropW(hwnd, ucs2!("ROS_MSGBOX").as_ptr()) as PMSGBOXDATA;
            if mbd.is_null() {
                return 0;
            }

            i = 0;
            while i < (*mbd).dwButtons {
                if iCtrlId == *(*mbd).pidButton.offset(i as isize) as INT_PTR {
                    EndDialog(hwnd, iCtrlId as INT_PTR);
                    return 1;
                }
                i += 1;
            }

            return 0;
        }

        WM_COPY => {
            MessageBoxTextToClipboard(hwnd);
            return 1;
        }

        WM_HELP => {
            let phi = lParam as LPHELPINFO;
            let mbd = GetPropW(hwnd, ucs2!("ROS_MSGBOX").as_ptr()) as PMSGBOXDATA;
            if mbd.is_null() {
                return 0;
            }
            (*phi).dwContextId = GetWindowContextHelpId(hwnd) as DWORD_PTR;

            if let Some(cb) = (*mbd).mbp.lpfnMsgBoxCallback {
                cb(phi);
            } else {
                let hwndOwner = GetWindow(hwnd, GW_OWNER);
                if !hwndOwner.is_null() {
                    SendMessageW(hwndOwner, WM_HELP, 0, lParam);
                }
            }
            return 1;
        }

        WM_CLOSE => {
            let mbd = GetPropW(hwnd, ucs2!("ROS_MSGBOX").as_ptr()) as PMSGBOXDATA;
            if mbd.is_null() {
                return 0;
            }

            if (*mbd).uCancelId == 0 {
                return 1;
            }
            EndDialog(hwnd, (*mbd).uCancelId as INT_PTR);
            return 1;
        }

        WM_TIMER => {
            if wParam == 0 {
                EndDialog(hwnd, IDTIMEOUT as INT_PTR);
            }
            return 0;
        }

        _ => {}
    }

    let _ = mbd;
    0
}

unsafe fn MessageBoxTimeoutIndirectW(
    lpMsgBoxParams: *const MSGBOXPARAMSW,
    dwTimeout: UINT,
) -> INT {
    let mut ret: INT = 0;
    let mut i: UINT;
    let mut defCaption: LPWSTR = core::ptr::null_mut();
    let mut mbd: MSGBOXDATA = core::mem::zeroed();
    let mut Buttons: MSGBTNINFO;
    let mut ButtonText: [LPCWSTR; MSGBOXEX_MAXBTNS as usize] = [core::ptr::null(); MSGBOXEX_MAXBTNS as usize];

    if (*lpMsgBoxParams).dwStyle & MB_SERVICE_NOTIFICATION != 0 {
        let Status: NTSTATUS;
        let mut CaptionU: _UNICODE_STRING = core::mem::zeroed();
        let mut TextU: _UNICODE_STRING = core::mem::zeroed();
        let mut Response: ULONG = ResponseNotHandled as ULONG;
        let mut MsgBoxParams: [ULONG_PTR; 4] = [
            core::ptr::addr_of_mut!(TextU) as ULONG_PTR,
            core::ptr::addr_of_mut!(CaptionU) as ULONG_PTR,
            ((*lpMsgBoxParams).dwStyle & !MB_SERVICE_NOTIFICATION) as ULONG_PTR,
            dwTimeout as ULONG_PTR,
        ];

        if !(*lpMsgBoxParams).hwndOwner.is_null() {
            return 0;
        }

        RtlInitUnicodeString(&mut CaptionU, (*lpMsgBoxParams).lpszCaption);
        RtlInitUnicodeString(&mut TextU, (*lpMsgBoxParams).lpszText);

        Status = NtRaiseHardError(
            (STATUS_SERVICE_NOTIFICATION | HARDERROR_OVERRIDE_ERRORMODE as NTSTATUS) as NTSTATUS,
            MsgBoxParams.len() as ULONG,
            (1 | 2),
            MsgBoxParams.as_mut_ptr(),
            OptionOk as u32,
            &mut Response,
        );
        if !NT_SUCCESS(Status) {
            return 0;
        }

        match Response {
            x if x == ResponseReturnToCaller as ULONG => {}
            x if x == ResponseNotHandled as ULONG => {}
            x if x == ResponseAbort as ULONG => return IDABORT as INT,
            x if x == ResponseCancel as ULONG => return IDCANCEL as INT,
            x if x == ResponseIgnore as ULONG => return IDIGNORE as INT,
            x if x == ResponseNo as ULONG => return IDNO as INT,
            x if x == ResponseOk as ULONG => return IDOK as INT,
            x if x == ResponseRetry as ULONG => return IDRETRY as INT,
            x if x == ResponseYes as ULONG => return IDYES as INT,
            x if x == ResponseTryAgain as ULONG => return IDTRYAGAIN as INT,
            x if x == ResponseContinue as ULONG => return IDCONTINUE as INT,
            _ => {}
        }
        return 0;
    }

    core::ptr::write_bytes(&mut mbd, 0, 1);
    core::ptr::copy_nonoverlapping(
        lpMsgBoxParams,
        &mut mbd.mbp,
        1,
    );
    mbd.wLanguageId = (*lpMsgBoxParams).dwLanguageId as WORD;
    mbd.dwTimeout = dwTimeout;

    if mbd.mbp.lpszCaption.is_null() {
        LoadAllocStringW(
            User32Instance,
            IDS_ERROR,
            &mut defCaption,
            ucs2!("Error").as_ptr(),
        );
        mbd.mbp.lpszCaption = if !defCaption.is_null() {
            defCaption
        } else {
            ucs2!("Error").as_ptr() as LPWSTR
        };
    }

    i = (*lpMsgBoxParams).dwStyle & MB_TYPEMASK;
    if i as usize >= MsgBtnInfo.len() {
        i = MB_OK;
    }

    // C does a by-value struct copy; _MSGBTNINFO is POD (u32 + two arrays), so a
    // bitwise read of the read-only static element is the faithful equivalent.
    Buttons = core::ptr::read(&MsgBtnInfo[i as usize]);

    if (*lpMsgBoxParams).dwStyle & MB_HELP != 0 {
        Buttons.btnIdx[Buttons.btnCnt as usize] = IDHELP as INT;
        Buttons.btnIds[Buttons.btnCnt as usize] = IDS_HELP;
        Buttons.btnCnt += 1;
    }

    mbd.uCancelId = if i == MB_OK { IDOK as UINT } else { 0 };
    i = 0;
    while i < Buttons.btnCnt {
        ButtonText[i as usize] = MB_GetString(Buttons.btnIds[i as usize] - IDS_OK);
        if Buttons.btnIdx[i as usize] == IDCANCEL as INT {
            mbd.uCancelId = IDCANCEL as UINT;
        }
        i += 1;
    }

    mbd.pidButton = Buttons.btnIdx.as_mut_ptr();
    mbd.ppszButtonText = ButtonText.as_mut_ptr();
    mbd.dwButtons = Buttons.btnCnt;

    mbd.uDefButton = ((*lpMsgBoxParams).dwStyle & MB_DEFMASK) >> 8;
    if mbd.uDefButton >= mbd.dwButtons {
        mbd.uDefButton = 0;
    }

    ret = SoftModalMessageBox(&mut mbd);

    if !defCaption.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, defCaption as PVOID);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn SoftModalMessageBox(lpMsgBoxData: LPMSGBOXDATA) -> INT {
    let mut ret: INT = 0;
    let mut mbd: MSGBOXDATA = core::mem::zeroed();
    let lpMsgBoxParams: LPMSGBOXPARAMSW = core::ptr::addr_of_mut!(mbd.mbp);
    let tpl: *mut DLGTEMPLATE;
    let mut iico: *mut DLGITEMTEMPLATE;
    let itxt: *mut DLGITEMTEMPLATE;
    let mut ibtn: *mut DLGITEMTEMPLATE;
    let mut nclm: NONCLIENTMETRICSW = core::mem::zeroed();
    let mut buf: LPVOID = core::ptr::null_mut();
    let mut dest: *mut u8;
    let mut caption: LPWSTR = core::ptr::null_mut();
    let mut text: LPWSTR = core::ptr::null_mut();
    let mut hFont: HFONT;
    let hOldFont: HFONT;
    let hIcon: HICON;
    let mut hDCWnd: HWND;
    let mut hDC: HDC;
    let mut units: SIZE = core::mem::zeroed();
    let mut bufsize: INT;
    let mut caplen: INT = 0;
    let mut textlen: INT = 0;
    let mut i: INT;
    let mut btnleft: INT;
    let mut btntop: INT;
    let mut ButtonLen: usize;
    let mut btnrect: RECT = core::mem::zeroed();
    let mut txtrect: RECT = core::mem::zeroed();
    let mut rc: RECT = core::mem::zeroed();
    let mut btnsize: SIZE = core::mem::zeroed();
    let mut iconPos: POINT = core::mem::zeroed();
    let mut iconSize: SIZE = core::mem::zeroed();

    type DWORD = u32;
    type UINT = u32;
    type BOOL = i32;
    type DLGPROC = Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR>;

    let is_intresource = |p: *const u16| -> bool { (p as usize) >> 16 == 0 };
    let align_up_u32 = |x: i32| -> i32 { ((x as u32 + 3) & !3u32) as i32 };
    let align_up_ptr = |p: *mut u8| -> *mut u8 { ((p as usize + 3) & !3usize) as *mut u8 };
    let rescale_x = |x: i32, u: &SIZE| -> i16 { (((x * 4 + u.cx - 1) / u.cx) as i16) };
    let rescale_y = |y: i32, u: &SIZE| -> i16 { (((y * 8 + u.cy - 1) / u.cy) as i16) };

    core::ptr::copy_nonoverlapping(
        lpMsgBoxData as *const u8,
        core::ptr::addr_of_mut!(mbd) as *mut u8,
        core::mem::size_of::<MSGBOXDATA>(),
    );

    'quit: {
        /* Load the caption */
        caption = core::ptr::null_mut();
        if !(*lpMsgBoxParams).lpszCaption.is_null() && is_intresource((*lpMsgBoxParams).lpszCaption) {
            caplen = LoadAllocStringW(
                (*lpMsgBoxParams).hInstance,
                PtrToUlong((*lpMsgBoxParams).lpszCaption as *mut core::ffi::c_void),
                core::ptr::addr_of_mut!(caption),
                core::ptr::null(),
            ) as INT;
            (*lpMsgBoxParams).lpszCaption = caption;
        } else if !(*lpMsgBoxParams).lpszCaption.is_null() {
            caplen = wcslen((*lpMsgBoxParams).lpszCaption) as INT;
        }
        if (*lpMsgBoxParams).lpszCaption.is_null() {
            (*lpMsgBoxParams).lpszCaption = w!("").as_ptr() as LPCWSTR;
            caplen = 0;
        }

        /* Load the text */
        text = core::ptr::null_mut();
        if !(*lpMsgBoxParams).lpszText.is_null() && is_intresource((*lpMsgBoxParams).lpszText) {
            textlen = LoadAllocStringW(
                (*lpMsgBoxParams).hInstance,
                PtrToUlong((*lpMsgBoxParams).lpszText as *mut core::ffi::c_void),
                core::ptr::addr_of_mut!(text),
                core::ptr::null(),
            ) as INT;
            (*lpMsgBoxParams).lpszText = text;
        } else if !(*lpMsgBoxParams).lpszText.is_null() {
            textlen = wcslen((*lpMsgBoxParams).lpszText) as INT;
        }
        if (*lpMsgBoxParams).lpszText.is_null() {
            (*lpMsgBoxParams).lpszText = w!("").as_ptr() as LPCWSTR;
            textlen = 0;
        }

        const IDI_HANDW: LPCWSTR = 32513 as LPCWSTR;
        const IDI_QUESTIONW: LPCWSTR = 32514 as LPCWSTR;
        const IDI_EXCLAMATIONW: LPCWSTR = 32515 as LPCWSTR;
        const IDI_ASTERISKW: LPCWSTR = 32516 as LPCWSTR;

        match (*lpMsgBoxParams).dwStyle & MB_ICONMASK {
            x if x == MB_ICONEXCLAMATION => {
                hIcon = LoadIconW(core::ptr::null_mut(), IDI_EXCLAMATIONW);
                MessageBeep(MB_ICONEXCLAMATION);
            }
            x if x == MB_ICONQUESTION => {
                hIcon = LoadIconW(core::ptr::null_mut(), IDI_QUESTIONW);
                MessageBeep(MB_ICONQUESTION);
            }
            x if x == MB_ICONASTERISK => {
                hIcon = LoadIconW(core::ptr::null_mut(), IDI_ASTERISKW);
                MessageBeep(MB_ICONASTERISK);
            }
            x if x == MB_ICONHAND => {
                hIcon = LoadIconW(core::ptr::null_mut(), IDI_HANDW);
                MessageBeep(MB_ICONHAND);
            }
            x if x == MB_USERICON => {
                hIcon = LoadIconW((*lpMsgBoxParams).hInstance, (*lpMsgBoxParams).lpszIcon);
                MessageBeep(MB_OK);
            }
            _ => {
                hIcon = core::ptr::null_mut();
                MessageBeep(MB_OK);
            }
        }
        (*lpMsgBoxParams).lpszIcon = hIcon as LPCWSTR;

        /* Basic space */
        bufsize = core::mem::size_of::<DLGTEMPLATE>() as INT
            + 2 * core::mem::size_of::<u16>() as INT
            + (caplen + 1) * core::mem::size_of::<u16>() as INT
            + core::mem::size_of::<u16>() as INT;

        if !hIcon.is_null() {
            bufsize = align_up_u32(bufsize);
            bufsize += core::mem::size_of::<DLGITEMTEMPLATE>() as INT
                + 4 * core::mem::size_of::<u16>() as INT
                + core::mem::size_of::<u16>() as INT;
        }

        bufsize = align_up_u32(bufsize);
        bufsize += core::mem::size_of::<DLGITEMTEMPLATE>() as INT
            + 3 * core::mem::size_of::<u16>() as INT
            + (textlen + 1) * core::mem::size_of::<u16>() as INT;

        i = 0;
        while i < mbd.dwButtons as INT {
            if (*mbd.ppszButtonText.add(i as usize)).is_null() || **mbd.ppszButtonText.add(i as usize) == 0 {
                *mbd.ppszButtonText.add(i as usize) = w!("").as_ptr() as PWSTR;
                ButtonLen = 0;
            } else {
                ButtonLen = wcslen(*mbd.ppszButtonText.add(i as usize));
            }

            bufsize = align_up_u32(bufsize);
            bufsize += core::mem::size_of::<DLGITEMTEMPLATE>() as INT
                + 3 * core::mem::size_of::<u16>() as INT
                + (ButtonLen as INT + 1) * core::mem::size_of::<u16>() as INT;
            i += 1;
        }

        buf = RtlAllocateHeap(RtlGetProcessHeap(), 0, bufsize as SIZE_T);
        if buf.is_null() {
            break 'quit;
        }

        iico = core::ptr::null_mut();

        nclm.cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as u32;
        SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            core::mem::size_of::<NONCLIENTMETRICSW>() as UINT,
            core::ptr::addr_of_mut!(nclm) as *mut core::ffi::c_void,
            0,
        );
        hFont = CreateFontIndirectW(core::ptr::addr_of!(nclm.lfMessageFont).cast());
        if hFont.is_null() {
            break 'quit;
        }

        hDCWnd = core::ptr::null_mut();
        hDC = GetDCEx(hDCWnd, core::ptr::null_mut(), DCX_WINDOW | DCX_CACHE);
        if hDC.is_null() {
            hDCWnd = (*lpMsgBoxParams).hwndOwner;
            hDC = GetDCEx(hDCWnd, core::ptr::null_mut(), DCX_WINDOW | DCX_CACHE);
        }
        if hDC.is_null() {
            break 'quit;
        }
        hOldFont = SelectObject(hDC, hFont as *mut core::ffi::c_void) as HFONT;

        units.cx = GdiGetCharDimensions(hDC, core::ptr::null_mut(), core::ptr::addr_of_mut!(units.cy));
        if units.cx == 0 {
            let defUnits = GetDialogBaseUnits() as u32;
            units.cx = (defUnits & 0xFFFF) as INT;
            units.cy = ((defUnits >> 16) & 0xFFFF) as INT;
        }

        /* Calculate the caption rectangle */
        txtrect.right = MulDiv(GetSystemMetrics(SM_CXSCREEN as INT), 4, 5);
        if !hIcon.is_null() {
            txtrect.right -= GetSystemMetrics(SM_CXICON as INT) + MSGBOXEX_SPACING as INT;
        }
        txtrect.top = 0;
        txtrect.left = 0;
        txtrect.bottom = 0;
        if textlen != 0 {
            DrawTextW(
                hDC,
                (*lpMsgBoxParams).lpszText,
                textlen,
                core::ptr::addr_of_mut!(txtrect),
                DT_LEFT | DT_NOPREFIX | DT_WORDBREAK | DT_EXPANDTABS | DT_EXTERNALLEADING | DT_EDITCONTROL | DT_CALCRECT,
            );
        } else {
            txtrect.right = txtrect.left + 1;
            txtrect.bottom = txtrect.top + 1;
        }
        txtrect.right += 1;

        /* Calculate the maximum buttons size */
        btnsize.cx = 75;
        btnsize.cy = 23;
        btnrect.left = 0;
        btnrect.top = 0;
        i = 0;
        while i < mbd.dwButtons as INT {
            DrawTextW(
                hDC,
                *mbd.ppszButtonText.add(i as usize),
                wcslen(*mbd.ppszButtonText.add(i as usize)) as INT,
                core::ptr::addr_of_mut!(btnrect),
                DT_LEFT | DT_SINGLELINE | DT_CALCRECT,
            );
            btnsize.cx = core::cmp::max(btnsize.cx, btnrect.right);
            btnsize.cy = core::cmp::max(btnsize.cy, btnrect.bottom);
            i += 1;
        }

        if !hOldFont.is_null() {
            SelectObject(hDC, hOldFont as *mut core::ffi::c_void);
        }

        ReleaseDC(hDCWnd, hDC);

        if !hFont.is_null() {
            DeleteObject(hFont as *mut core::ffi::c_void);
        }

        /* Calculate position and size of the icon */
        rc.left = 0;
        rc.bottom = 0;
        rc.right = 0;
        btntop = 0;
        if !hIcon.is_null() {
            rc.right = GetSystemMetrics(SM_CXICON as INT);
            rc.bottom = GetSystemMetrics(SM_CYICON as INT);
            rc.top = MSGBOXEX_MARGIN as INT;
            btnleft = (mbd.dwButtons as INT * (btnsize.cx + MSGBOXEX_BUTTONSPACING as INT)) - MSGBOXEX_BUTTONSPACING as INT;
            if btnleft > txtrect.right + rc.right + MSGBOXEX_SPACING as INT {
                rc.left = MSGBOXEX_MARGIN as INT;
                btnleft = MSGBOXEX_MARGIN as INT;
            } else {
                rc.left = MSGBOXEX_MARGIN as INT;
                btnleft = MSGBOXEX_MARGIN as INT
                    + ((txtrect.right + rc.right + MSGBOXEX_SPACING as INT - btnleft) / 2);
            }

            iconPos.x = rescale_x(rc.left, &units) as INT;
            iconPos.y = rescale_y(rc.top, &units) as INT;
            iconSize.cx = rescale_x(rc.right, &units) as INT;
            iconSize.cy = rescale_y(rc.bottom, &units) as INT;

            btntop = rc.top + rc.bottom + MSGBOXEX_SPACING as INT;
            rc.left += rc.right + MSGBOXEX_SPACING as INT;
        } else {
            btnleft = (mbd.dwButtons as INT * (btnsize.cx + MSGBOXEX_BUTTONSPACING as INT)) - MSGBOXEX_BUTTONSPACING as INT;
            if btnleft > txtrect.right {
                rc.left = MSGBOXEX_MARGIN as INT;
                btnleft = MSGBOXEX_MARGIN as INT;
            } else {
                rc.left = MSGBOXEX_MARGIN as INT;
                btnleft = MSGBOXEX_MARGIN as INT + ((txtrect.right - btnleft) / 2);
            }
        }

        /* Initialize the dialog template */
        tpl = buf as *mut DLGTEMPLATE;

        (*tpl).style = WS_CAPTION | WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS | WS_SYSMENU
            | DS_CENTER | DS_SETFONT | DS_MODALFRAME | DS_NOIDLEMSG;
        (*tpl).dwExtendedStyle = WS_EX_DLGMODALFRAME | WS_EX_WINDOWEDGE | WS_EX_CONTROLPARENT;
        if (*lpMsgBoxParams).dwStyle & MB_TOPMOST != 0 {
            (*tpl).dwExtendedStyle |= WS_EX_TOPMOST;
        }
        if (*lpMsgBoxParams).dwStyle & MB_RIGHT != 0 {
            (*tpl).dwExtendedStyle |= WS_EX_RIGHT;
        }
        (*tpl).x = 100;
        (*tpl).y = 100;
        (*tpl).cdit = (mbd.dwButtons + if !hIcon.is_null() { 1 } else { 0 } + 1) as u16;

        dest = (tpl as *mut DLGTEMPLATE).add(1) as *mut u8;

        *(dest as *mut DWORD) = 0;
        dest = dest.add(2 * core::mem::size_of::<u16>());
        core::ptr::copy_nonoverlapping(
            (*lpMsgBoxParams).lpszCaption as *const u8,
            dest,
            caplen as usize * core::mem::size_of::<u16>(),
        );
        dest = dest.add(caplen as usize * core::mem::size_of::<u16>());
        *(dest as *mut u16) = 0;
        dest = dest.add(core::mem::size_of::<u16>());

        *(dest as *mut u16) = 0x7FFF;
        dest = dest.add(core::mem::size_of::<u16>());

        /* Create the icon */
        if !hIcon.is_null() {
            dest = align_up_ptr(dest);
            iico = dest as *mut DLGITEMTEMPLATE;
            (*iico).style = WS_CHILD | WS_VISIBLE | SS_ICON;
            (*iico).dwExtendedStyle = 0;
            (*iico).id = MSGBOX_IDICON as u16;

            (*iico).x = iconPos.x as i16;
            (*iico).y = iconPos.y as i16;
            (*iico).cx = iconSize.cx as i16;
            (*iico).cy = iconSize.cy as i16;

            dest = dest.add(core::mem::size_of::<DLGITEMTEMPLATE>());
            *(dest as *mut u16) = 0xFFFF;
            dest = dest.add(core::mem::size_of::<u16>());
            *(dest as *mut u16) = 0x0082;
            dest = dest.add(core::mem::size_of::<u16>());
            *(dest as *mut u16) = 0xFFFF;
            dest = dest.add(core::mem::size_of::<u16>());
            *(dest as *mut u16) = 0;
            dest = dest.add(core::mem::size_of::<u16>());
            *(dest as *mut u16) = 0;
            dest = dest.add(core::mem::size_of::<u16>());
        }

        /* Create static for text */
        dest = align_up_ptr(dest);
        itxt = dest as *mut DLGITEMTEMPLATE;
        (*itxt).style = WS_CHILD | WS_VISIBLE | SS_NOPREFIX;
        if (*lpMsgBoxParams).dwStyle & MB_RIGHT != 0 {
            (*itxt).style |= SS_RIGHT;
        } else {
            (*itxt).style |= SS_LEFT;
        }
        (*itxt).dwExtendedStyle = 0;
        (*itxt).id = 0xffff;
        dest = dest.add(core::mem::size_of::<DLGITEMTEMPLATE>());
        *(dest as *mut u16) = 0xFFFF;
        dest = dest.add(core::mem::size_of::<u16>());
        *(dest as *mut u16) = 0x0082;
        dest = dest.add(core::mem::size_of::<u16>());
        core::ptr::copy_nonoverlapping(
            (*lpMsgBoxParams).lpszText as *const u8,
            dest,
            textlen as usize * core::mem::size_of::<u16>(),
        );
        dest = dest.add(textlen as usize * core::mem::size_of::<u16>());
        *(dest as *mut u16) = 0;
        dest = dest.add(core::mem::size_of::<u16>());
        *(dest as *mut u16) = 0;
        dest = dest.add(core::mem::size_of::<u16>());

        /* Calculate position of the text */
        rc.top = MSGBOXEX_MARGIN as INT + ((rc.bottom - txtrect.bottom) / 2);
        rc.top = core::cmp::max(rc.top, MSGBOXEX_MARGIN as INT);

        if mbd.uDefButton >= mbd.dwButtons {
            mbd.uDefButton = 0;
        }

        btntop = core::cmp::max(rc.top + txtrect.bottom + MSGBOXEX_SPACING as INT, btntop);
        i = 0;
        while i < mbd.dwButtons as INT {
            ButtonLen = wcslen(*mbd.ppszButtonText.add(i as usize));

            dest = align_up_ptr(dest);
            ibtn = dest as *mut DLGITEMTEMPLATE;

            (*ibtn).style = WS_CHILD | WS_VISIBLE | WS_TABSTOP;
            if i as u32 == mbd.uDefButton {
                (*ibtn).style |= BS_DEFPUSHBUTTON;
            } else {
                (*ibtn).style |= BS_PUSHBUTTON;
            }

            (*ibtn).dwExtendedStyle = 0;
            (*ibtn).id = *mbd.pidButton.add(i as usize) as u16;
            dest = dest.add(core::mem::size_of::<DLGITEMTEMPLATE>());
            *(dest as *mut u16) = 0xFFFF;
            dest = dest.add(core::mem::size_of::<u16>());
            *(dest as *mut u16) = 0x0080;
            dest = dest.add(core::mem::size_of::<u16>());
            core::ptr::copy_nonoverlapping(
                *mbd.ppszButtonText.add(i as usize) as *const u8,
                dest,
                ButtonLen * core::mem::size_of::<u16>(),
            );
            dest = dest.add(ButtonLen * core::mem::size_of::<u16>());
            *(dest as *mut u16) = 0;
            dest = dest.add(core::mem::size_of::<u16>());
            *(dest as *mut u16) = 0;
            dest = dest.add(core::mem::size_of::<u16>());

            (*ibtn).x = rescale_x(btnleft, &units);
            (*ibtn).y = rescale_y(btntop, &units);
            (*ibtn).cx = rescale_x(btnsize.cx, &units);
            (*ibtn).cy = rescale_y(btnsize.cy, &units);
            btnleft += btnsize.cx + MSGBOXEX_BUTTONSPACING as INT;
            i += 1;
        }

        /* Calculate the size and position of the message-box window */
        btnleft = core::cmp::max(btnleft - MSGBOXEX_BUTTONSPACING as INT, rc.left + txtrect.right);
        btnleft += MSGBOXEX_MARGIN as INT;
        if mbd.dwButtons > 0 {
            btntop += btnsize.cy + MSGBOXEX_MARGIN as INT;
        }

        /* Set the size and position of the static message */
        (*itxt).x = rescale_x(rc.left, &units);
        (*itxt).y = rescale_y(rc.top, &units);
        (*itxt).cx = rescale_x(btnleft - rc.left - MSGBOXEX_MARGIN as INT, &units);
        (*itxt).cy = rescale_y(txtrect.bottom, &units);

        /* Set the size of the window */
        (*tpl).cx = rescale_x(btnleft, &units);
        (*tpl).cy = rescale_y(btntop, &units);

        ret = DialogBoxIndirectParamW(
            (*lpMsgBoxParams).hInstance,
            tpl,
            (*lpMsgBoxParams).hwndOwner,
            core::mem::transmute::<usize, win32k_sys::win32k::DLGPROC>(MessageBoxProc as usize),
            core::ptr::addr_of_mut!(mbd) as LPARAM,
        ) as INT;
    }

    if !buf.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, buf);
    }
    if !text.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, text as *mut core::ffi::c_void);
    }
    if !caption.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, caption as *mut core::ffi::c_void);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn MessageBoxA(
    hWnd: HWND,
    lpText: LPCSTR,
    lpCaption: LPCSTR,
    uType: UINT,
) -> core::ffi::c_int {
    MessageBoxExA(hWnd, lpText, lpCaption, uType, LANG_NEUTRAL as WORD)
}

#[no_mangle]
pub unsafe extern "system" fn MessageBoxW(
    hWnd: HWND,
    lpText: LPCWSTR,
    lpCaption: LPCWSTR,
    uType: UINT,
) -> core::ffi::c_int {
    MessageBoxExW(hWnd, lpText, lpCaption, uType, LANG_NEUTRAL as WORD)
}

#[no_mangle]
pub unsafe extern "system" fn MessageBoxExA(
    hWnd: HWND,
    lpText: LPCSTR,
    lpCaption: LPCSTR,
    uType: UINT,
    wLanguageId: WORD,
) -> core::ffi::c_int {
    let mut msgbox: MSGBOXPARAMSA = core::mem::zeroed();

    msgbox.cbSize = core::mem::size_of::<MSGBOXPARAMSA>() as UINT;
    msgbox.hwndOwner = hWnd;
    msgbox.hInstance = 0 as HINSTANCE;
    msgbox.lpszText = lpText;
    msgbox.lpszCaption = lpCaption;
    msgbox.dwStyle = uType;
    msgbox.lpszIcon = core::ptr::null_mut();
    msgbox.dwContextHelpId = 0;
    msgbox.lpfnMsgBoxCallback = None;
    msgbox.dwLanguageId = wLanguageId as DWORD;

    MessageBoxIndirectA(&msgbox)
}

#[no_mangle]
pub unsafe extern "system" fn MessageBoxExW(
    hWnd: HWND,
    lpText: LPCWSTR,
    lpCaption: LPCWSTR,
    uType: UINT,
    wLanguageId: WORD,
) -> core::ffi::c_int {
    let mut msgbox: MSGBOXPARAMSW = core::mem::zeroed();

    msgbox.cbSize = core::mem::size_of::<MSGBOXPARAMSW>() as UINT;
    msgbox.hwndOwner = hWnd;
    msgbox.hInstance = core::ptr::null_mut();
    msgbox.lpszText = lpText;
    msgbox.lpszCaption = lpCaption;
    msgbox.dwStyle = uType;
    msgbox.lpszIcon = core::ptr::null_mut();
    msgbox.dwContextHelpId = 0;
    msgbox.lpfnMsgBoxCallback = None;
    msgbox.dwLanguageId = wLanguageId as DWORD;

    MessageBoxTimeoutIndirectW(&msgbox, !0u32) as core::ffi::c_int
}

#[no_mangle]
pub unsafe extern "system" fn MessageBoxIndirectA(
    lpMsgBoxParams: *const MSGBOXPARAMSA,
) -> core::ffi::c_int {
    let mut msgboxW: MSGBOXPARAMSW = core::mem::zeroed();
    let mut textW: _UNICODE_STRING = core::mem::zeroed();
    let mut captionW: _UNICODE_STRING = core::mem::zeroed();
    let mut iconW: _UNICODE_STRING = core::mem::zeroed();
    let ret: core::ffi::c_int;

    if !IS_INTRESOURCE((*lpMsgBoxParams).lpszText as *const core::ffi::c_void) {
        RtlCreateUnicodeStringFromAsciiz(
            core::ptr::addr_of_mut!(textW),
            (*lpMsgBoxParams).lpszText as PCSZ,
        );
        *textW.Buffer.add((textW.Length as usize) / core::mem::size_of::<u16>()) = 0u16;
    } else {
        textW.Buffer = (*lpMsgBoxParams).lpszText as LPWSTR;
    }

    if !IS_INTRESOURCE((*lpMsgBoxParams).lpszCaption as *const core::ffi::c_void) {
        RtlCreateUnicodeStringFromAsciiz(
            core::ptr::addr_of_mut!(captionW),
            (*lpMsgBoxParams).lpszCaption as PCSZ,
        );
        *captionW.Buffer.add((captionW.Length as usize) / core::mem::size_of::<u16>()) = 0u16;
    } else {
        captionW.Buffer = (*lpMsgBoxParams).lpszCaption as LPWSTR;
    }

    if (*lpMsgBoxParams).dwStyle & MB_USERICON != 0 {
        if !IS_INTRESOURCE((*lpMsgBoxParams).lpszIcon as *const core::ffi::c_void) {
            RtlCreateUnicodeStringFromAsciiz(
                core::ptr::addr_of_mut!(iconW),
                (*lpMsgBoxParams).lpszIcon as PCSZ,
            );
            *iconW.Buffer.add((iconW.Length as usize) / core::mem::size_of::<u16>()) = 0u16;
        } else {
            iconW.Buffer = (*lpMsgBoxParams).lpszIcon as LPWSTR;
        }
    } else {
        iconW.Buffer = core::ptr::null_mut();
    }

    msgboxW.cbSize = core::mem::size_of::<MSGBOXPARAMSW>() as u32;
    msgboxW.hwndOwner = (*lpMsgBoxParams).hwndOwner;
    msgboxW.hInstance = (*lpMsgBoxParams).hInstance;
    msgboxW.lpszText = textW.Buffer;
    msgboxW.lpszCaption = captionW.Buffer;
    msgboxW.dwStyle = (*lpMsgBoxParams).dwStyle;
    msgboxW.lpszIcon = iconW.Buffer;
    msgboxW.dwContextHelpId = (*lpMsgBoxParams).dwContextHelpId;
    msgboxW.lpfnMsgBoxCallback = (*lpMsgBoxParams).lpfnMsgBoxCallback;
    msgboxW.dwLanguageId = (*lpMsgBoxParams).dwLanguageId;

    ret = MessageBoxTimeoutIndirectW(core::ptr::addr_of!(msgboxW), -1i32 as UINT);

    if !IS_INTRESOURCE((*lpMsgBoxParams).lpszText as *const core::ffi::c_void) {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(textW));
    }

    if !IS_INTRESOURCE((*lpMsgBoxParams).lpszCaption as *const core::ffi::c_void) {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(captionW));
    }

    if (*lpMsgBoxParams).dwStyle & MB_USERICON != 0
        && !IS_INTRESOURCE(iconW.Buffer as *const core::ffi::c_void)
    {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(iconW));
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn MessageBoxIndirectW(
    lpMsgBoxParams: *const MSGBOXPARAMSW,
) -> INT {
    MessageBoxTimeoutIndirectW(lpMsgBoxParams, u32::MAX)
}

#[no_mangle]
pub unsafe extern "system" fn MessageBoxTimeoutA(
    hWnd: HWND,
    lpText: LPCSTR,
    lpCaption: LPCSTR,
    uType: UINT,
    wLanguageId: WORD,
    dwTimeout: DWORD,
) -> INT {
    let mut msgboxW: MSGBOXPARAMSW = core::mem::zeroed();
    let mut textW: _UNICODE_STRING = core::mem::zeroed();
    let mut captionW: _UNICODE_STRING = core::mem::zeroed();
    let ret: INT;

    if !IS_INTRESOURCE(lpText as *const _) {
        RtlCreateUnicodeStringFromAsciiz(&mut textW, lpText as PCSZ);
    } else {
        textW.Buffer = lpText as LPWSTR;
    }

    if !IS_INTRESOURCE(lpCaption as *const _) {
        RtlCreateUnicodeStringFromAsciiz(&mut captionW, lpCaption as PCSZ);
    } else {
        captionW.Buffer = lpCaption as LPWSTR;
    }

    msgboxW.cbSize = core::mem::size_of::<MSGBOXPARAMSW>() as UINT;
    msgboxW.hwndOwner = hWnd;
    msgboxW.hInstance = 0 as HINSTANCE;
    msgboxW.lpszText = textW.Buffer;
    msgboxW.lpszCaption = captionW.Buffer;
    msgboxW.dwStyle = uType;
    msgboxW.lpszIcon = core::ptr::null_mut();
    msgboxW.dwContextHelpId = 0;
    msgboxW.lpfnMsgBoxCallback = None;
    msgboxW.dwLanguageId = wLanguageId as DWORD;

    ret = MessageBoxTimeoutIndirectW(&msgboxW, dwTimeout as UINT);

    if !IS_INTRESOURCE(textW.Buffer as *const _) {
        RtlFreeUnicodeString(&mut textW);
    }

    if !IS_INTRESOURCE(captionW.Buffer as *const _) {
        RtlFreeUnicodeString(&mut captionW);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn MessageBoxTimeoutW(
    hWnd: HWND,
    lpText: LPCWSTR,
    lpCaption: LPCWSTR,
    uType: UINT,
    wLanguageId: WORD,
    dwTimeout: DWORD,
) -> INT {
    let mut msgbox: MSGBOXPARAMSW = core::mem::zeroed();

    msgbox.cbSize = core::mem::size_of::<MSGBOXPARAMSW>() as UINT;
    msgbox.hwndOwner = hWnd;
    msgbox.hInstance = core::ptr::null_mut();
    msgbox.lpszText = lpText;
    msgbox.lpszCaption = lpCaption;
    msgbox.dwStyle = uType;
    msgbox.lpszIcon = core::ptr::null_mut();
    msgbox.dwContextHelpId = 0;
    msgbox.lpfnMsgBoxCallback = None;
    msgbox.dwLanguageId = wLanguageId as DWORD;

    MessageBoxTimeoutIndirectW(&msgbox, dwTimeout as UINT)
}

#[no_mangle]
pub unsafe extern "system" fn MessageBeep(uType: UINT) -> BOOL {
    NtUserxMessageBeep(uType)
}

#[no_mangle]
pub unsafe extern "system" fn MB_GetString(wBtn: UINT) -> LPCWSTR {
    static mut bCached: BOOL = 0;
    static mut MBStrings: [MBSTRING; MAX_MB_STRINGS as usize] =
        unsafe { core::mem::zeroed() };

    if bCached == 0 {
        let mut i: UINT = 0;
        while i < MAX_MB_STRINGS {
            MBStrings[i as usize].uID = (IDOK + i) as UINT;
            MBStrings[i as usize].uStr = (IDS_OK + i) as UINT;
            LoadStringW(
                User32Instance,
                MBStrings[i as usize].uStr,
                MBStrings[i as usize].szName.as_mut_ptr(),
                MBStrings[i as usize].szName.len() as i32,
            );
            i += 1;
        }
        bCached = 1;
    }

    if wBtn > (IDCONTINUE - 1) as UINT {
        return core::ptr::null_mut();
    }

    MBStrings[wBtn as usize].szName.as_ptr() as LPCWSTR
}
