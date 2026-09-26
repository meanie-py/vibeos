use win32k_sys::win32k::*;
#[allow(unused_imports)]
use core::ffi::c_void;

// LOWORD/HIWORD are macros; ValidateHwnd is the fastcall-split internal helper.
use crate::internal::{ValidateHwnd, HIWORD, LOWORD};
// gdi32 glue available from crate::winbase (CreatePen/DeleteObject/SelectObject + GetTextMetricsW).
use crate::winbase::{CreatePen, DeleteObject, GetTextMetricsW, SelectObject};
// NtUserx* multiplexers for the user32 syscall-forwarder exports.
use crate::ntwrapper::{


    BeginPaint, DestroyWindow, EndPaint, GetDC, InvalidateRect, MoveWindow, NtUserxReleaseCapture,
    NtUserxReleaseDC, NtUserxSwitchToThisWindow, SetCapture, SetWindowPos, ShowWindowAsync,
    WaitMessage,
};

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


// ── No-op debug macro (TRACE compiles to nothing; consume args so they type-check).

// WC_SWITCH = MAKEINTATOM(0x8003) (undocuser.h) — atom encoded as a class-name pointer.
const WC_SWITCH: PCWSTR = 0x8003 as PCWSTR;

// winuser.h / winreg.h literals absent from the win32k bindgen.
#[allow(non_camel_case_types)]
type REGSAM = u32;
const CW_USEDEFAULT: INT = 0x8000_0000u32 as INT;
const HWND_TOP: HWND = core::ptr::null_mut();
const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as HKEY;

// ReleaseCapture/ReleaseDC/SwitchToThisWindow forward to their NtUserx* multiplexers
// (keeps the C call sites — and the 1u8/1 fAltTab args — intact via BOOL coercion).
#[inline]
unsafe fn ReleaseCapture() -> BOOL {
    NtUserxReleaseCapture()
}
#[inline]
unsafe fn ReleaseDC(_hWnd: HWND, hDC: HDC) -> INT {
    NtUserxReleaseDC(hDC)
}
#[inline]
unsafe fn SwitchToThisWindow(hWnd: HWND, fAltTab: BOOL) {
    NtUserxSwitchToThisWindow(hWnd, fAltTab)
}

extern "system" {
    fn DestroyIcon(hIcon: HICON) -> i32;

    // ── user32 own-exports (sibling object files / export table within this DLL).
    fn CreateWindowExW(dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, x: INT, y: INT, nWidth: INT, nHeight: INT, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID) -> HWND;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR;
    fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR;
    fn GetWindowLongW(hWnd: HWND, nIndex: INT) -> LONG;
    fn SendMessageTimeoutW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM, fuFlags: UINT, uTimeout: UINT, lpdwResult: PDWORD_PTR) -> LRESULT;
    fn GetClassLongPtrW(hWnd: HWND, nIndex: INT) -> ULONG_PTR;
    fn CopyIcon(hIcon: HICON) -> HICON;
    fn GetWindow(hWnd: HWND, uCmd: UINT) -> HWND;
    fn IsWindowVisible(hWnd: HWND) -> BOOL;
    fn GetWindowRect(hWnd: HWND, lpRect: LPRECT) -> BOOL;
    fn IsRectEmpty(lprc: *const RECT) -> BOOL;
    fn GetClassNameW(hWnd: HWND, lpClassName: LPWSTR, nMaxCount: INT) -> INT;
    fn GetLastActivePopup(hWnd: HWND) -> HWND;
    fn EnumWindows(lpEnumFunc: WNDENUMPROC, lParam: LPARAM) -> BOOL;
    fn GetWindowTextW(hWnd: HWND, lpString: LPWSTR, nMaxCount: INT) -> INT;
    fn GetSystemMetrics(nIndex: INT) -> INT;
    fn SetRect(lprc: LPRECT, xLeft: INT, yTop: INT, xRight: INT, yBottom: INT) -> BOOL;
    fn AdjustWindowRectEx(lpRect: LPRECT, dwStyle: DWORD, bMenu: BOOL, dwExStyle: DWORD) -> BOOL;
    fn GetClientRect(hWnd: HWND, lpRect: LPRECT) -> BOOL;
    fn GetActiveWindow() -> HWND;
    fn GetAsyncKeyState(vKey: INT) -> SHORT;
    fn GetKeyState(nVirtKey: INT) -> SHORT;
    fn PeekMessageW(lpMsg: LPMSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT, wRemoveMsg: UINT) -> BOOL;
    fn CallMsgFilterW(lpMsg: LPMSG, nCode: INT) -> BOOL;
    fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;

    // ── gdi32 exports.
    fn GetStockObject(i: INT) -> HGDIOBJ;
    fn GetSysColor(nIndex: INT) -> COLORREF;
    fn Rectangle(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT) -> BOOL;
    fn DrawIconEx(hdc: HDC, xLeft: INT, yTop: INT, hIcon: HICON, cxWidth: INT, cyWidth: INT, istepIfAniCur: UINT, hbrFlickerFreeDraw: HBRUSH, diFlags: UINT) -> BOOL;
    fn DrawFrameControl(hdc: HDC, lprc: LPRECT, uType: UINT, uState: UINT) -> BOOL;
    fn FillRect(hDC: HDC, lprc: *const RECT, hbr: HBRUSH) -> INT;
    fn DrawTextW(hdc: HDC, lpchText: LPCWSTR, cchText: INT, lprc: LPRECT, format: UINT) -> INT;
    fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn SetBkMode(hdc: HDC, mode: INT) -> INT;

    // ── advapi32 registry exports.
    fn RegOpenKeyExW(hKey: HKEY, lpSubKey: LPCWSTR, ulOptions: DWORD, samDesired: REGSAM, phkResult: PHKEY) -> LONG;
    fn RegQueryValueExW(hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD) -> LONG;
    fn RegCloseKey(hKey: HKEY) -> LONG;
}

// kernel32 heap + memory glue.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: usize) -> PVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: PVOID) -> BOOL;
    fn RtlMoveMemory(Destination: PVOID, Source: *const core::ffi::c_void, Length: usize);
}

// MoveMemory is the kernel32 macro alias for RtlMoveMemory.
#[inline]
unsafe fn MoveMemory(Destination: *mut core::ffi::c_void, Source: *const core::ffi::c_void, Length: usize) {
    RtlMoveMemory(Destination as PVOID, Source, Length)
}

// CRT (msvcrt): wide string helpers.
extern "C" {
    fn wcscmp(string1: *const WCHAR, string2: *const WCHAR) -> INT;
    fn wcstol(strSource: *const WCHAR, endptr: *mut *mut WCHAR, base: INT) -> core::ffi::c_long;
}

// GetTextMetrics → GetTextMetricsW (gdi32, winbase glue).
#[inline]
unsafe fn GetTextMetrics(hdc: HDC, lptm: LPTEXTMETRICW) -> BOOL {
    GetTextMetricsW(hdc, lptm)
}

// User32Instance global (dllmain.rs).
extern "C" {
    static User32Instance: HINSTANCE;
}

// ---- file-scope preamble (translated) ----
pub const DIALOG_MARGIN: i32 = 8;
pub const CX_ICON: i32 = 32;
pub const CY_ICON: i32 = 32;
pub const ICON_MARGIN: i32 = 4;
pub const ITEM_MARGIN: i32 = 4;
pub const CY_TEXT_MARGIN: i32 = 4;
pub const MAX_WINDOWS: usize = 120;

pub static mut switchdialog: HWND = core::ptr::null_mut();
pub static mut dialogFont: HFONT = core::ptr::null_mut();
pub static mut selectedWindow: i32 = 0;
pub static mut isOpen: BOOL = 0;
pub static mut fontHeight: i32 = 0;
pub static mut windowText: [WCHAR; 1024] = [0; 1024];
pub static mut windowList: [HWND; MAX_WINDOWS] = [core::ptr::null_mut(); MAX_WINDOWS];
pub static mut iconList: [HICON; MAX_WINDOWS] = [core::ptr::null_mut(); MAX_WINDOWS];
pub static mut windowCount: i32 = 0;
pub static mut cxBorder: i32 = 0;
pub static mut cyBorder: i32 = 0;
pub static mut nItems: i32 = 0;
pub static mut nCols: i32 = 0;
pub static mut nRows: i32 = 0;
pub static mut itemsW: i32 = 0;
pub static mut itemsH: i32 = 0;
pub static mut totalW: i32 = 0;
pub static mut totalH: i32 = 0;
pub static mut xOffset: i32 = 0;
pub static mut yOffset: i32 = 0;
pub static mut ptStart: POINT = POINT { x: 0, y: 0 };
pub static mut nShift: i32 = 0;
pub static mut Esc: BOOL = 0;

pub const DefSwitchRows: i32 = 3;
pub const DefSwitchColumns: i32 = 7;

pub static mut CoolSwitch: BOOL = 1;
pub static mut CoolSwitchRows: i32 = DefSwitchRows;
pub static mut CoolSwitchColumns: i32 = DefSwitchColumns;
pub static mut SettingsLoaded: BOOL = 0;
pub static Style: DWORD = WS_POPUP | WS_BORDER | WS_DISABLED;
pub static ExStyle: DWORD = WS_EX_TOPMOST | WS_EX_DLGMODALFRAME | WS_EX_TOOLWINDOW;

unsafe fn GetRegInt(hKey: HKEY, Name: PCWSTR, DefVal: i32) -> i32 {
    let mut buf: [u16; 12] = [0; 12];
    let mut cb: u32 = core::mem::size_of_val(&buf) as u32;
    let mut type_: u32 = 0;
    let err = RegQueryValueExW(
        hKey,
        Name,
        core::ptr::null_mut(),
        &mut type_,
        buf.as_mut_ptr() as *mut u8,
        &mut cb,
    );
    if err as u32 == ERROR_SUCCESS
        && (cb as usize) <= core::mem::size_of_val(&buf) - core::mem::size_of::<u16>()
    {
        buf[(cb as usize) / core::mem::size_of::<u16>()] = 0;
        if type_ == REG_SZ || type_ == REG_EXPAND_SZ {
            let mut pszEnd: *mut u16 = core::ptr::null_mut();
            let Value = wcstol(buf.as_ptr(), &mut pszEnd, 10);
            return if pszEnd > buf.as_ptr() as *mut u16 {
                Value as i32
            } else {
                DefVal
            };
        }
        if (type_ == REG_DWORD || type_ == REG_BINARY)
            && cb as usize == core::mem::size_of::<u32>()
        {
            return *(buf.as_ptr() as *const u32) as i32;
        }
    }
    DefVal
}

unsafe fn LoadCoolSwitchSettings() {
    let mut hKey: HKEY = core::ptr::null_mut();

    if SettingsLoaded != 0
        && (GetKeyState(VK_SCROLL as i32) & 1) == 0
    {
        return;
    }

    SettingsLoaded = 1;

    if RegOpenKeyExW(
        HKEY_CURRENT_USER,
        w!("Control Panel\\Desktop").as_ptr() as PCWSTR,
        0,
        KEY_READ,
        &mut hKey,
    ) == 0
    {
        CoolSwitch = GetRegInt(
            hKey,
            w!("CoolSwitch").as_ptr() as PCWSTR,
            1,
        );
        CoolSwitchRows = GetRegInt(
            hKey,
            w!("CoolSwitchRows").as_ptr() as PCWSTR,
            DefSwitchRows as i32,
        );
        CoolSwitchColumns = GetRegInt(
            hKey,
            w!("CoolSwitchColumns").as_ptr() as PCWSTR,
            DefSwitchColumns as i32,
        );
        RegCloseKey(hKey);
    }

    if CoolSwitchRows * CoolSwitchColumns < 3 {
        CoolSwitchRows = DefSwitchRows as i32;
        CoolSwitchColumns = DefSwitchColumns as i32;
    }

    TRACE!(b"CoolSwitch: %d\n\0".as_ptr() as *const i8, CoolSwitch);
    TRACE!(b"CoolSwitchRows: %d\n\0".as_ptr() as *const i8, CoolSwitchRows);
    TRACE!(b"CoolSwitchColumns: %d\n\0".as_ptr() as *const i8, CoolSwitchColumns);
}

unsafe fn ResizeAndCenter(hwnd: HWND, mut width: i32, mut height: i32) {
    let mut x: i32;
    let mut y: i32;
    let mut Rect: RECT = core::mem::zeroed();

    let screenwidth = GetSystemMetrics(SM_CXSCREEN as i32);
    let screenheight = GetSystemMetrics(SM_CYSCREEN as i32);

    x = (screenwidth - width) / 2;
    y = (screenheight - height) / 2;

    SetRect(core::ptr::addr_of_mut!(Rect), x, y, x + width, y + height);
    AdjustWindowRectEx(core::ptr::addr_of_mut!(Rect), Style, 0, ExStyle);

    x = Rect.left;
    y = Rect.top;
    width = Rect.right - Rect.left;
    height = Rect.bottom - Rect.top;
    MoveWindow(hwnd, x, y, width, height, 0);

    ptStart.x = x;
    ptStart.y = y;
}

unsafe fn CompleteSwitch(doSwitch: BOOL) {
    if isOpen == 0 {
        return;
    }

    isOpen = 0;

    TRACE!(b"[ATbot] CompleteSwitch Hiding Window.\n\0".as_ptr());
    ShowWindowAsync(switchdialog, SW_HIDE as i32);

    if doSwitch != 0 {
        if selectedWindow >= windowCount {
            return;
        }

        {
            let hwnd = windowList[selectedWindow as usize];

            GetWindowTextW(hwnd, (*(&raw mut windowText)).as_mut_ptr(), ((*(&raw const windowText)).len()) as i32);

            TRACE!(b"[ATbot] CompleteSwitch Switching to 0x%08x (%ls)\n\0".as_ptr(), hwnd, (*(&raw const windowText)).as_ptr());

            SwitchToThisWindow(hwnd, 1);
        }
    }

    windowCount = 0;
}

#[no_mangle]
pub unsafe extern "system" fn EnumerateCallback(window: HWND, lParam: LPARAM) -> BOOL {
    let mut hIcon: HICON = core::ptr::null_mut();
    let bAlive: LRESULT;

    let _ = lParam;

    const ICON_TIMEOUT: u32 = 100;
    bAlive = SendMessageTimeoutW(
        window,
        WM_GETICON,
        ICON_BIG as WPARAM,
        0,
        SMTO_ABORTIFHUNG | SMTO_BLOCK,
        ICON_TIMEOUT,
        core::ptr::addr_of_mut!(hIcon) as PDWORD_PTR,
    );
    if hIcon.is_null() {
        hIcon = GetClassLongPtrW(window, GCL_HICON) as HICON;
        if hIcon.is_null() {
            if bAlive != 0 {
                SendMessageTimeoutW(
                    window,
                    WM_GETICON,
                    ICON_SMALL2 as WPARAM,
                    0,
                    SMTO_ABORTIFHUNG | SMTO_BLOCK,
                    ICON_TIMEOUT,
                    core::ptr::addr_of_mut!(hIcon) as PDWORD_PTR,
                );
            }
            if hIcon.is_null() {
                hIcon = if !crate::misc::dllmain::gpsi.is_null() {
                    crate::wow64::psi::hicon_windows()
                } else {
                    core::ptr::null_mut()
                };
                if hIcon.is_null() {
                    return 1;
                }
            }
        }
    }

    windowList[windowCount as usize] = window;
    iconList[windowCount as usize] = CopyIcon(hIcon);
    windowCount += 1;

    ((windowCount as usize) < MAX_WINDOWS) as BOOL
}

unsafe fn GetNiceRootOwner(mut hwnd: HWND) -> HWND {
    let mut hwndOwner: HWND;
    let mut exStyle: DWORD;
    let mut OwnerExStyle: DWORD;

    loop {
        exStyle = GetWindowLongW(hwnd, GWL_EXSTYLE as i32) as DWORD;
        if exStyle & WS_EX_APPWINDOW != 0 {
            break;
        }

        hwndOwner = GetWindow(hwnd, GW_OWNER as u32);
        OwnerExStyle = GetWindowLongW(hwndOwner, GWL_EXSTYLE as i32) as DWORD;
        if IsWindowVisible(hwndOwner) == 0 || (OwnerExStyle & WS_EX_TOOLWINDOW != 0) {
            break;
        }

        hwnd = hwndOwner;
    }

    hwnd
}

unsafe extern "system" fn IsAltTabWindow(hwnd: HWND) -> BOOL {
    let mut exStyle: LONG_PTR;
    let ClassStyle: LONG_PTR;
    let mut rc: RECT = core::mem::zeroed();
    let mut hwndTry: HWND;
    let mut hwndWalk: HWND;
    let mut szClass: [WCHAR; 64] = [0; 64];

    if IsWindowVisible(hwnd) == 0 {
        return 0;
    }

    exStyle = GetWindowLongPtrW(hwnd, GWL_EXSTYLE as _);
    if exStyle & ((WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE) as LONG_PTR) != 0 {
        return 0;
    }

    GetWindowRect(hwnd, &mut rc);
    if IsRectEmpty(&rc) != 0 {
        return 0;
    }

    if GetClassNameW(hwnd, szClass.as_mut_ptr(), szClass.len() as _) == 0
        || wcscmp(szClass.as_ptr(), b"S\0h\0e\0l\0l\0_\0T\0r\0a\0y\0W\0n\0d\0\0\0".as_ptr() as *const WCHAR) == 0
        || wcscmp(szClass.as_ptr(), b"P\0r\0o\0g\0m\0a\0n\0\0\0".as_ptr() as *const WCHAR) == 0
    {
        return 1;
    }

    ClassStyle = GetClassLongPtrW(hwnd, GCL_STYLE as _) as LONG_PTR;
    if ClassStyle & (CS_IME as LONG_PTR) != 0 {
        return 0;
    }

    hwndWalk = GetNiceRootOwner(hwnd);

    loop {
        hwndTry = GetLastActivePopup(hwndWalk);
        if hwndTry == hwndWalk {
            break;
        }

        exStyle = GetWindowLongW(hwndTry, GWL_EXSTYLE as _) as LONG_PTR;
        if IsWindowVisible(hwndTry) != 0 && exStyle & (WS_EX_TOOLWINDOW as LONG_PTR) == 0 {
            break;
        }

        hwndWalk = hwndTry;
    }

    (hwnd == hwndTry) as BOOL
}

unsafe extern "system" fn EnumWindowsProc(hwnd: HWND, lParam: LPARAM) -> BOOL {
    if IsAltTabWindow(hwnd) != 0 {
        if EnumerateCallback(hwnd, lParam) == 0 {
            return 0;
        }
    }
    1
}

unsafe fn ProcessMouseMessage(message: UINT, lParam: LPARAM) {
    let xPos = LOWORD(lParam as u32) as i32;
    let yPos = HIWORD(lParam as u32) as i32;

    let xIndex = (xPos - DIALOG_MARGIN) / (CX_ICON + 2 * ICON_MARGIN + 2 * ITEM_MARGIN);
    let yIndex = (yPos - DIALOG_MARGIN) / (CY_ICON + 2 * ICON_MARGIN + 2 * ITEM_MARGIN);

    if xIndex < 0 || nCols <= xIndex || yIndex < 0 || nRows <= yIndex {
        return;
    }

    selectedWindow = (yIndex * nCols) + xIndex;
    if message == WM_MOUSEMOVE {
        InvalidateRect(switchdialog, core::ptr::null_mut(), 1);
    } else {
        selectedWindow = (yIndex * nCols) + xIndex;
        CompleteSwitch(1);
    }
}

unsafe fn OnPaint(hWnd: HWND) {
    let dialogDC: HDC;
    let mut paint: PAINTSTRUCT = core::mem::zeroed();
    let mut cRC: RECT = core::mem::zeroed();
    let mut textRC: RECT = core::mem::zeroed();
    let mut i: i32;
    let mut xPos: i32;
    let mut yPos: i32;
    let CharCount: i32;
    let dcFont: HFONT;
    let mut hIcon: HICON;
    let mut hPen: HPEN;
    let mut Color: COLORREF;

    const DIALOG_MARGIN: i32 = 8;
    const CX_ICON: i32 = 32;
    const CY_ICON: i32 = 32;
    const ICON_MARGIN: i32 = 4;
    const CX_ITEM: i32 = CX_ICON + 2 * ICON_MARGIN;
    const CY_ITEM: i32 = CY_ICON + 2 * ICON_MARGIN;
    const ITEM_MARGIN: i32 = 4;
    const CX_ITEM_SPACE: i32 = CX_ITEM + 2 * ITEM_MARGIN;
    const CY_ITEM_SPACE: i32 = CY_ITEM + 2 * ITEM_MARGIN;

    // check
    if nCols == 0 || nItems == 0 {
        return;
    }

    // begin painting
    dialogDC = BeginPaint(hWnd, core::ptr::addr_of_mut!(paint));
    if dialogDC.is_null() {
        return;
    }

    // fill the client area
    GetClientRect(hWnd, core::ptr::addr_of_mut!(cRC));
    FillRect(dialogDC, core::ptr::addr_of_mut!(cRC), (COLOR_3DFACE + 1) as HBRUSH);

    // if the selection index exceeded the display items, then
    // do display item shifting
    if selectedWindow >= nItems {
        nShift = selectedWindow - nItems + 1;
    } else {
        nShift = 0;
    }

    i = 0;
    while i < nItems {
        // get the icon to display
        hIcon = iconList[(i + nShift) as usize];

        // calculate the position where we start drawing
        xPos = DIALOG_MARGIN + CX_ITEM_SPACE * (i % nCols) + ITEM_MARGIN;
        yPos = DIALOG_MARGIN + CY_ITEM_SPACE * (i / nCols) + ITEM_MARGIN;

        // centering
        if nItems < CoolSwitchColumns {
            xPos += (itemsW - nItems * CX_ITEM_SPACE) / 2;
        }

        // if this position is selected,
        if selectedWindow == i + nShift {
            // create a solid pen
            Color = GetSysColor(COLOR_HIGHLIGHT as i32);
            hPen = CreatePen(PS_SOLID as i32, 1, Color);

            // draw a rectangle with using the pen
            SelectObject(dialogDC, hPen as _);
            SelectObject(dialogDC, GetStockObject(NULL_BRUSH as i32));
            Rectangle(dialogDC, xPos, yPos, xPos + CX_ITEM, yPos + CY_ITEM);
            Rectangle(dialogDC, xPos + 1, yPos + 1,
                                xPos + CX_ITEM - 1, yPos + CY_ITEM - 1);

            // delete the pen
            DeleteObject(hPen as _);
        }

        // draw icon
        DrawIconEx(dialogDC, xPos + ICON_MARGIN, yPos + ICON_MARGIN,
                   hIcon, CX_ICON, CY_ICON, 0, core::ptr::null_mut(), DI_NORMAL);

        i += 1;
    }

    // set the text rectangle
    SetRect(core::ptr::addr_of_mut!(textRC), DIALOG_MARGIN, DIALOG_MARGIN + itemsH,
            totalW - DIALOG_MARGIN, totalH - DIALOG_MARGIN);

    // draw the sunken button around text
    DrawFrameControl(dialogDC, core::ptr::addr_of_mut!(textRC), DFC_BUTTON,
                     DFCS_BUTTONPUSH | DFCS_PUSHED);

    // get text
    CharCount = GetWindowTextW(windowList[selectedWindow as usize], (*(&raw mut windowText)).as_mut_ptr(),
                               ((*(&raw const windowText)).len()) as i32);

    // draw text
    dcFont = SelectObject(dialogDC, dialogFont as _) as HFONT;
    SetTextColor(dialogDC, GetSysColor(COLOR_BTNTEXT as i32));
    SetBkMode(dialogDC, TRANSPARENT as i32);
    DrawTextW(dialogDC, (*(&raw const windowText)).as_ptr(), CharCount, core::ptr::addr_of_mut!(textRC),
              DT_CENTER | DT_VCENTER | DT_END_ELLIPSIS | DT_SINGLELINE);
    SelectObject(dialogDC, dcFont as _);

    // end painting
    EndPaint(hWnd, core::ptr::addr_of_mut!(paint));
}

unsafe fn CreateSwitcherWindow(hInstance: HINSTANCE) -> DWORD {
    switchdialog = CreateWindowExW(
        (WS_EX_TOPMOST | WS_EX_DLGMODALFRAME | WS_EX_TOOLWINDOW) as u32,
        WC_SWITCH,
        w!("").as_ptr() as PCWSTR,
        (WS_POPUP | WS_BORDER | WS_DISABLED) as u32,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        400,
        150,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        hInstance,
        core::ptr::null_mut(),
    );

    if switchdialog.is_null() {
        TRACE!(b"[ATbot] Task Switcher Window failed to create.\n\0".as_ptr() as PCHAR);
        return 0;
    }

    isOpen = 0;
    1
}

unsafe extern "system" fn GetDialogFont() -> DWORD {
    let tDC: HDC;
    let mut tm: TEXTMETRICW = core::mem::zeroed();

    dialogFont = GetStockObject(DEFAULT_GUI_FONT as i32) as HFONT;

    tDC = GetDC(0 as HWND);
    GetTextMetrics(tDC, &mut tm);
    fontHeight = tm.tmHeight;
    ReleaseDC(0 as HWND, tDC);

    1
}

unsafe fn PrepareWindow() {
    nItems = windowCount;

    nCols = CoolSwitchColumns;
    nRows = (nItems + CoolSwitchColumns - 1) / CoolSwitchColumns;
    if nRows > CoolSwitchRows {
        nRows = CoolSwitchRows;
        nItems = nRows * nCols;
    }

    itemsW = nCols * (CX_ICON + 2 * ICON_MARGIN + 2 * ITEM_MARGIN);
    itemsH = nRows * (CY_ICON + 2 * ICON_MARGIN + 2 * ITEM_MARGIN);

    totalW = itemsW + 2 * DIALOG_MARGIN;
    totalH = itemsH + 2 * DIALOG_MARGIN;
    totalH += fontHeight + 2 * CY_TEXT_MARGIN;

    ResizeAndCenter(switchdialog, totalW, totalH);
}

unsafe extern "system" fn ProcessHotKey() -> BOOL {
    if isOpen == 0 {
        windowCount = 0;
        EnumWindows(core::mem::transmute::<_, WNDENUMPROC>(EnumWindowsProc as *const ()), 0);

        if windowCount == 0 {
            return 0;
        }

        if windowCount == 1 {
            SwitchToThisWindow(windowList[0], 1);
            return 0;
        }

        if CreateSwitcherWindow(User32Instance) == 0 {
            return 0;
        }

        selectedWindow = 1;

        TRACE!(b"[ATbot] HotKey Received. Opening window.\n\0".as_ptr());
        ShowWindowAsync(switchdialog, SW_SHOWNORMAL as i32);
        SwitchToThisWindow(switchdialog, 1);
        isOpen = 1;
    } else {
        TRACE!(b"[ATbot] HotKey Received  Rotating.\n\0".as_ptr());
        selectedWindow = (selectedWindow + 1) % windowCount;
        InvalidateRect(switchdialog, core::ptr::null_mut(), 1);
    }
    1
}

unsafe fn RotateTasks(bShift: BOOL) {
    let hwndFirst: HWND;
    let hwndLast: HWND;
    let Size: DWORD;

    if windowCount < 2 || Esc == 0 {
        return;
    }

    hwndFirst = windowList[0];
    hwndLast = windowList[(windowCount - 1) as usize];

    if bShift != 0 {
        SetWindowPos(hwndLast, HWND_TOP, 0, 0, 0, 0,
                     SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE |
                     SWP_NOOWNERZORDER | SWP_NOREPOSITION | SWP_ASYNCWINDOWPOS);

        SwitchToThisWindow(hwndLast, 1u8 as BOOL);

        Size = (windowCount - 1) as DWORD * core::mem::size_of::<HWND>() as DWORD;
        MoveMemory(core::ptr::addr_of_mut!(windowList[1]) as *mut core::ffi::c_void,
                   core::ptr::addr_of_mut!(windowList[0]) as *const core::ffi::c_void,
                   Size as usize);
        windowList[0] = hwndLast;
    } else {
        SetWindowPos(hwndFirst, hwndLast, 0, 0, 0, 0,
                     SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE |
                     SWP_NOOWNERZORDER | SWP_NOREPOSITION | SWP_ASYNCWINDOWPOS);

        SwitchToThisWindow(windowList[1], 1u8 as BOOL);

        Size = (windowCount - 1) as DWORD * core::mem::size_of::<HWND>() as DWORD;
        MoveMemory(core::ptr::addr_of_mut!(windowList[0]) as *mut core::ffi::c_void,
                   core::ptr::addr_of_mut!(windowList[1]) as *const core::ffi::c_void,
                   Size as usize);
        windowList[(windowCount - 1) as usize] = hwndFirst;
    }
}

unsafe fn MoveLeft() {
    selectedWindow = selectedWindow - 1;
    if selectedWindow < 0 {
        selectedWindow = windowCount - 1;
    }
    InvalidateRect(switchdialog, core::ptr::null_mut(), 1);
}

unsafe fn MoveRight() {
    selectedWindow = (selectedWindow + 1) % windowCount;
    InvalidateRect(switchdialog, core::ptr::null_mut(), 1u8 as _);
}

unsafe fn MoveUp() {
    let mut iRow: i32 = selectedWindow / nCols;
    let iCol: i32 = selectedWindow % nCols;

    iRow -= 1;
    if iRow < 0 {
        iRow = nRows - 1;
    }

    selectedWindow = iRow * nCols + iCol;
    if selectedWindow >= windowCount {
        selectedWindow = windowCount - 1;
    }
    InvalidateRect(switchdialog, core::ptr::null_mut(), 1);
}

unsafe fn MoveDown() {
    let mut iRow: INT = selectedWindow / nCols;
    let iCol: INT = selectedWindow % nCols;

    iRow += 1;
    if iRow >= nRows {
        iRow = 0;
    }

    selectedWindow = iRow * nCols + iCol;
    if selectedWindow >= windowCount {
        selectedWindow = windowCount - 1;
    }
    InvalidateRect(switchdialog, core::ptr::null_mut(), 1);
}

unsafe fn DestroyAppWindows() {
    let mut i: INT;
    i = 0;
    while i < windowCount {
        DestroyIcon(iconList[i as usize]);
        iconList[i as usize] = core::ptr::null_mut();
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn DoAppSwitch(wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut hwndActive: HWND;
    let mut msg: MSG = core::mem::zeroed();

    LoadCoolSwitchSettings();

    if CoolSwitch == 0 {
        return 0;
    }

    if !switchdialog.is_null() || Esc != 0 {
        return 0;
    }

    if lParam == VK_ESCAPE as LPARAM {
        Esc = 1;

        windowCount = 0;
        EnumWindows(core::mem::transmute::<_, WNDENUMPROC>(EnumWindowsProc as *const ()), 0);

        if windowCount < 2 {
            return 0;
        }

        RotateTasks((GetAsyncKeyState(VK_SHIFT as i32) < 0) as BOOL);

        hwndActive = GetActiveWindow();

        if hwndActive.is_null() {
            Esc = 0;
            return 0;
        }
    }

    hwndActive = GetActiveWindow();
    if !hwndActive.is_null() {
        SetCapture(hwndActive);
    }

    'exit: {
        match lParam {
            x if x == VK_TAB as LPARAM => {
                if GetDialogFont() == 0 || ProcessHotKey() == 0 {
                    break 'exit;
                }
            }
            x if x == VK_ESCAPE as LPARAM => {}
            _ => {
                break 'exit;
            }
        }

        if hwndActive.is_null() {
            break 'exit;
        }

        loop {
            loop {
                if PeekMessageW(&mut msg, core::ptr::null_mut(), 0, 0, PM_NOREMOVE) != 0 {
                    if CallMsgFilterW(&mut msg, MSGF_NEXTWINDOW as i32) == 0 {
                        break;
                    }
                    PeekMessageW(&mut msg, core::ptr::null_mut(), msg.message, msg.message, PM_REMOVE);
                } else {
                    WaitMessage();
                }
            }

            match msg.message {
                m if m == WM_KEYUP => {
                    PeekMessageW(&mut msg, core::ptr::null_mut(), msg.message, msg.message, PM_REMOVE);
                    if msg.wParam == VK_MENU as WPARAM {
                        CompleteSwitch(1);
                    } else if msg.wParam == VK_RETURN as WPARAM {
                        CompleteSwitch(1);
                    } else if msg.wParam == VK_ESCAPE as WPARAM {
                        CompleteSwitch(0);
                    }
                    break 'exit;
                }

                m if m == WM_SYSKEYDOWN => {
                    PeekMessageW(&mut msg, core::ptr::null_mut(), msg.message, msg.message, PM_REMOVE);
                    if (HIWORD(msg.lParam as u32) & KF_ALTDOWN as u16) != 0 {
                        if msg.wParam == VK_TAB as WPARAM {
                            if Esc != 0 {
                                continue;
                            }
                            if GetKeyState(VK_SHIFT as i32) < 0 {
                                MoveLeft();
                            } else {
                                MoveRight();
                            }
                        } else if msg.wParam == VK_ESCAPE as WPARAM {
                            if Esc == 0 {
                                continue;
                            }
                            RotateTasks((GetKeyState(VK_SHIFT as i32) < 0) as BOOL);
                        } else if msg.wParam == VK_LEFT as WPARAM {
                            MoveLeft();
                        } else if msg.wParam == VK_RIGHT as WPARAM {
                            MoveRight();
                        } else if msg.wParam == VK_UP as WPARAM {
                            MoveUp();
                        } else if msg.wParam == VK_DOWN as WPARAM {
                            MoveDown();
                        }
                    }
                }

                m if m == WM_LBUTTONUP => {
                    PeekMessageW(&mut msg, core::ptr::null_mut(), msg.message, msg.message, PM_REMOVE);
                    ProcessMouseMessage(msg.message, msg.lParam);
                    break 'exit;
                }

                _ => {
                    if PeekMessageW(&mut msg, core::ptr::null_mut(), msg.message, msg.message, PM_REMOVE) != 0 {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
            }
        }
    }

    ReleaseCapture();
    if !switchdialog.is_null() {
        DestroyWindow(switchdialog);
    }
    if Esc != 0 {
        DestroyAppWindows();
    }
    switchdialog = core::ptr::null_mut();
    selectedWindow = 0;
    windowCount = 0;
    Esc = 0;
    0
}

#[no_mangle]
pub unsafe extern "system" fn SwitchWndProc_common(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    unicode: BOOL,
) -> LRESULT {
    let pWnd: PWND = ValidateHwnd(hWnd);
    if !pWnd.is_null() {
        if (*pWnd).fnid == 0 {
            NtUserSetWindowFNID(hWnd, FNID_SWITCH as u16);
        }
    }

    match uMsg {
        WM_NCCREATE => {
            let ati = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<ALTTABINFO>(),
            ) as PALTTABINFO;
            if ati.is_null() {
                return 0;
            }
            SetWindowLongPtrW(hWnd, 0, ati as LONG_PTR);
            return TRUE as LRESULT;
        }

        WM_SHOWWINDOW => {
            if wParam != 0 {
                PrepareWindow();
                let ati = GetWindowLongPtrW(hWnd, 0) as PALTTABINFO;
                // Null-guard: the inlined GetWindowLongPtrW error paths return
                // 0, and unguarded derefs of a provably-null pointer fold the
                // whole path to ud2 (Â§9.103) â which is ALSO a real crash if
                // the window data was never set.
                if !ati.is_null() {
                    (*ati).cbSize = core::mem::size_of::<ALTTABINFO>() as u32;
                    (*ati).cItems = nItems;
                    (*ati).cColumns = nCols;
                    (*ati).cRows = nRows;
                    if nCols != 0 {
                        (*ati).iColFocus = (selectedWindow - nShift) % nCols;
                        (*ati).iRowFocus = (selectedWindow - nShift) / nCols;
                    } else {
                        (*ati).iColFocus = 0;
                        (*ati).iRowFocus = 0;
                    }
                    (*ati).cxItem = CX_ICON + 2 * ICON_MARGIN + 2 * ITEM_MARGIN;
                    (*ati).cyItem = CY_ICON + 2 * ICON_MARGIN + 2 * ITEM_MARGIN;
                    (*ati).ptStart.x = ptStart.x;
                    (*ati).ptStart.y = ptStart.y;
                }
            }
            return 0;
        }

        WM_MOUSEMOVE => {
            ProcessMouseMessage(uMsg, lParam);
            return 0;
        }

        WM_ACTIVATE => {
            if wParam == WA_INACTIVE as WPARAM {
                CompleteSwitch(FALSE as BOOL);
            }
            return 0;
        }

        WM_PAINT => {
            OnPaint(hWnd);
            return 0;
        }

        WM_DESTROY => {
            isOpen = FALSE as BOOL;
            let ati = GetWindowLongPtrW(hWnd, 0) as PALTTABINFO;
            HeapFree(GetProcessHeap(), 0, ati as *mut core::ffi::c_void);
            SetWindowLongPtrW(hWnd, 0, 0);
            DestroyAppWindows();
            NtUserSetWindowFNID(hWnd, FNID_DESTROY as u16);
            return 0;
        }

        _ => {}
    }
    DefWindowProcW(hWnd, uMsg, wParam, lParam)
}

#[no_mangle]
pub unsafe extern "system" fn SwitchWndProcA(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    SwitchWndProc_common(hWnd, uMsg, wParam, lParam, 0)
}

#[no_mangle]
pub unsafe extern "system" fn SwitchWndProcW(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    SwitchWndProc_common(hWnd, uMsg, wParam, lParam, 1)
}
