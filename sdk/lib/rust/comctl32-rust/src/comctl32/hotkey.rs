use crate::comctl32::commctrl::COMCTL32_IsReflectedMessage;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_hotkey);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const HOTKEY_CLASSW: [::core::ffi::c_ushort; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_ushort; 16]>(
        *b"m\0s\0c\0t\0l\0s\0_\0h\0o\0t\0k\0e\0y\x003\x002\0\0\0",
    )
};

static mut HOTKEY_plussep: [WCHAR; 3] = [
    ' ' as i32 as WCHAR,
    '+' as i32 as WCHAR,
    ' ' as i32 as WCHAR,
];

#[repr(C)]
pub struct HOTKEY_INFO {
    pub hwndSelf: HWND,
    pub hwndNotify: HWND,
    pub hFont: HFONT,
    pub bFocus: BOOL,
    pub nHeight: INT,
    pub HotKey: WORD,
    pub InvComb: WORD,
    pub InvMod: WORD,
    pub CurrMod: BYTE,
    pub CaretPos: INT,
    pub ScanCode: DWORD,
    pub strNone: [WCHAR; 15], /* hope it's long enough ... */
}

unsafe extern "C" fn HOTKEY_IsCombInv(infoPtr: *const HOTKEY_INFO) -> BOOL {
    if ((*infoPtr).InvComb & (HKCOMB_NONE as u16)) != 0 && (*infoPtr).CurrMod == 0 {
        return TRUE;
    }
    if ((*infoPtr).InvComb & (HKCOMB_S as u16)) != 0 && IsOnlySet!(infoPtr, (HOTKEYF_SHIFT as u8)) {
        return TRUE;
    }
    if ((*infoPtr).InvComb & (HKCOMB_C as u16)) != 0 && IsOnlySet!(infoPtr, (HOTKEYF_CONTROL as u8)) {
        return TRUE;
    }
    if ((*infoPtr).InvComb & (HKCOMB_A as u16)) != 0 && IsOnlySet!(infoPtr, (HOTKEYF_ALT as u8)) {
        return TRUE;
    }
    if ((*infoPtr).InvComb & (HKCOMB_SC as u16)) != 0
        && IsOnlySet!(infoPtr, (HOTKEYF_SHIFT | HOTKEYF_CONTROL) as u8)
    {
        return TRUE;
    }
    if ((*infoPtr).InvComb & (HKCOMB_SA as u16)) != 0
        && IsOnlySet!(infoPtr, (HOTKEYF_SHIFT | HOTKEYF_ALT) as u8)
    {
        return TRUE;
    }
    if ((*infoPtr).InvComb & (HKCOMB_CA as u16)) != 0
        && IsOnlySet!(infoPtr, (HOTKEYF_CONTROL | HOTKEYF_ALT) as u8)
    {
        return TRUE;
    }
    if ((*infoPtr).InvComb & (HKCOMB_SCA as u16)) != 0
        && IsOnlySet!(infoPtr, (HOTKEYF_SHIFT | HOTKEYF_CONTROL | HOTKEYF_ALT) as u8)
    {
        return TRUE;
    }

    return FALSE;
}

unsafe extern "C" fn HOTKEY_DrawHotKey(infoPtr: *mut HOTKEY_INFO, hdc: HDC, KeyName: LPCWSTR, NameLen: WORD) {
    let mut TextSize: SIZE = core::mem::zeroed();
    let mut nXStart: INT;
    let mut nYStart: INT;
    let mut clrOldText: COLORREF;
    let mut clrOldBk: COLORREF;
    let mut hFontOld: HFONT;

    /* Make a gap from the frame */
    nXStart = GetSystemMetrics((SM_CXBORDER) as i32) as INT;
    nYStart = GetSystemMetrics((SM_CYBORDER) as i32) as INT;

    hFontOld = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;
    if GetWindowLongW((*infoPtr).hwndSelf, GWL_STYLE) as INT & WS_DISABLED as INT != 0
    {
        clrOldText = SetTextColor(hdc, comctl32_color.clrGrayText);
        clrOldBk = SetBkColor(hdc, comctl32_color.clrBtnFace);
    }
    else
    {
        clrOldText = SetTextColor(hdc, comctl32_color.clrWindowText);
        clrOldBk = SetBkColor(hdc, comctl32_color.clrWindow);
    }

    TextOutW(hdc, nXStart, nYStart, KeyName, NameLen as INT);

    /* Get the text width for the caret */
    GetTextExtentPoint32W(hdc, KeyName, NameLen as INT, core::ptr::addr_of_mut!(TextSize));
    (*infoPtr).CaretPos = nXStart + TextSize.cx;

    SetBkColor(hdc, clrOldBk);
    SetTextColor(hdc, clrOldText);
    SelectObject(hdc, hFontOld as HGDIOBJ);

    /* position the caret */
    SetCaretPos((*infoPtr).CaretPos, nYStart);
}

unsafe extern "C" fn HOTKEY_Refresh(infoPtr: *mut HOTKEY_INFO, hdc: HDC) {
    let mut KeyName: [WCHAR; 64] = [0; 64];
    let mut NameLen: WORD = 0;
    let mut Modifier: BYTE = 0;

    if (*infoPtr).CurrMod == 0 && (*infoPtr).HotKey == 0 {
        HOTKEY_DrawHotKey(
            infoPtr,
            hdc,
            core::ptr::addr_of_mut!((*infoPtr).strNone) as *mut WCHAR as LPCWSTR,
            lstrlenW(core::ptr::addr_of_mut!((*infoPtr).strNone) as *mut WCHAR as LPCWSTR) as WORD,
        );
        return;
    }

    if (*infoPtr).HotKey != 0 {
        Modifier = HIBYTE!((*infoPtr).HotKey) as BYTE;
    } else if HOTKEY_IsCombInv(infoPtr) != 0 {
        Modifier = (*infoPtr).InvMod as BYTE;
    } else {
        Modifier = (*infoPtr).CurrMod;
    }

    if (Modifier as i32) & (HOTKEYF_CONTROL as i32) != 0 {
        GetKeyNameTextW(
            MAKELPARAM!(
                0 as WORD,
                MapVirtualKeyW(VK_CONTROL as UINT, 0 as UINT) as WORD,
            ) as LONG,
            core::ptr::addr_of_mut!(KeyName) as *mut WCHAR as LPWSTR,
            64,
        );
        NameLen = lstrlenW(core::ptr::addr_of_mut!(KeyName) as *mut WCHAR as LPCWSTR) as WORD;
        memcpy(
            (core::ptr::addr_of_mut!(KeyName) as *mut WCHAR).add(NameLen as usize) as *mut core::ffi::c_void,
            core::ptr::addr_of!(HOTKEY_plussep) as *const core::ffi::c_void,
            core::mem::size_of::<[WCHAR; 3]>(),
        );
        NameLen += 3;
    }
    if (Modifier as i32) & (HOTKEYF_SHIFT as i32) != 0 {
        GetKeyNameTextW(
            MAKELPARAM!(
                0 as WORD,
                MapVirtualKeyW(VK_SHIFT as UINT, 0 as UINT) as WORD,
            ) as LONG,
            (core::ptr::addr_of_mut!(KeyName) as *mut WCHAR).add(NameLen as usize) as LPWSTR,
            64 - (NameLen as i32),
        );
        NameLen = lstrlenW(core::ptr::addr_of_mut!(KeyName) as *mut WCHAR as LPCWSTR) as WORD;
        memcpy(
            (core::ptr::addr_of_mut!(KeyName) as *mut WCHAR).add(NameLen as usize) as *mut core::ffi::c_void,
            core::ptr::addr_of!(HOTKEY_plussep) as *const core::ffi::c_void,
            core::mem::size_of::<[WCHAR; 3]>(),
        );
        NameLen += 3;
    }
    if (Modifier as i32) & (HOTKEYF_ALT as i32) != 0 {
        GetKeyNameTextW(
            MAKELPARAM!(
                0 as WORD,
                MapVirtualKeyW(VK_MENU as UINT, 0 as UINT) as WORD,
            ) as LONG,
            (core::ptr::addr_of_mut!(KeyName) as *mut WCHAR).add(NameLen as usize) as LPWSTR,
            64 - (NameLen as i32),
        );
        NameLen = lstrlenW(core::ptr::addr_of_mut!(KeyName) as *mut WCHAR as LPCWSTR) as WORD;
        memcpy(
            (core::ptr::addr_of_mut!(KeyName) as *mut WCHAR).add(NameLen as usize) as *mut core::ffi::c_void,
            core::ptr::addr_of!(HOTKEY_plussep) as *const core::ffi::c_void,
            core::mem::size_of::<[WCHAR; 3]>(),
        );
        NameLen += 3;
    }

    if (*infoPtr).HotKey != 0 {
        GetKeyNameTextW(
            (*infoPtr).ScanCode as LONG,
            (core::ptr::addr_of_mut!(KeyName) as *mut WCHAR).add(NameLen as usize) as LPWSTR,
            64 - (NameLen as i32),
        );
        NameLen = lstrlenW(core::ptr::addr_of_mut!(KeyName) as *mut WCHAR as LPCWSTR) as WORD;
    } else {
        KeyName[NameLen as usize] = 0;
    }

    HOTKEY_DrawHotKey(
        infoPtr,
        hdc,
        core::ptr::addr_of_mut!(KeyName) as *mut WCHAR as LPCWSTR,
        NameLen,
    );
}

unsafe extern "C" fn HOTKEY_Paint(infoPtr: *mut HOTKEY_INFO, mut hdc: HDC) {
    if !hdc.is_null() {
        HOTKEY_Refresh(infoPtr, hdc);
    } else {
        let mut ps: PAINTSTRUCT = core::mem::zeroed();
        hdc = BeginPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
        HOTKEY_Refresh(infoPtr, hdc);
        EndPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
    }
}

unsafe extern "C" fn HOTKEY_GetHotKey(infoPtr: *const HOTKEY_INFO) -> LRESULT {
    return (*infoPtr).HotKey as LRESULT;
}

unsafe extern "C" fn HOTKEY_SetHotKey(infoPtr: *mut HOTKEY_INFO, hotKey: WORD) {
    (*infoPtr).HotKey = hotKey;
    (*infoPtr).ScanCode = MAKELPARAM!(
        0,
        MapVirtualKeyW((LOBYTE!((*infoPtr).HotKey)) as UINT, 0),
    ) as DWORD;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/hotkey.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"HOTKEY_SetHotKey\0" as *const u8 as *const ::core::ffi::c_char,
            215 as ::core::ffi::c_int,
            b"(infoPtr=%p hotKey=%x) Modifiers: 0x%x, Virtual Key: %d\n\0" as *const u8
                as *const ::core::ffi::c_char,
            infoPtr,
            hotKey as ::core::ffi::c_int,
            ((*infoPtr).HotKey as DWORD_PTR >> 8 as ::core::ffi::c_int
                & 0xff as DWORD_PTR) as BYTE as ::core::ffi::c_int,
            ((*infoPtr).HotKey as DWORD_PTR & 0xff as DWORD_PTR) as BYTE
                as ::core::ffi::c_int,
        );
    }
    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE);
}

unsafe extern "C" fn HOTKEY_SetRules(
    infoPtr: *mut HOTKEY_INFO,
    invComb: WORD,
    invMod: WORD,
) {
    (*infoPtr).InvComb = invComb;
    (*infoPtr).InvMod = invMod;
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/hotkey.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"HOTKEY_SetRules\0" as *const u8 as *const ::core::ffi::c_char,
            225 as ::core::ffi::c_int,
            b"(infoPtr=%p) Invalid Modifiers: 0x%x, If Invalid: 0x%x\n\0" as *const u8
                as *const ::core::ffi::c_char,
            infoPtr,
            (*infoPtr).InvComb as ::core::ffi::c_int,
            (*infoPtr).InvMod as ::core::ffi::c_int,
        );
    }
}

unsafe extern "C" fn HOTKEY_Create(infoPtr: *mut HOTKEY_INFO, lpcs: *const CREATESTRUCTW) -> LRESULT {
    (*infoPtr).hwndNotify = (*lpcs).hwndParent;

    HOTKEY_SetFont(infoPtr, GetStockObject((SYSTEM_FONT as i32)) as HFONT, 0);

    return 0;
}

unsafe extern "C" fn HOTKEY_Destroy(mut infoPtr: *mut HOTKEY_INFO) -> LRESULT {
    SetWindowLongPtrW((*infoPtr).hwndSelf, 0, 0);
    heap_free(infoPtr as *mut core::ffi::c_void);
    return 0;
}

unsafe extern "C" fn HOTKEY_EraseBackground(infoPtr: *const HOTKEY_INFO, hdc: HDC) -> LRESULT {
    let mut hBrush: HBRUSH = core::ptr::null_mut();
    let mut hSolidBrush: HBRUSH = core::ptr::null_mut();
    let mut rc: RECT = core::mem::zeroed();
    if (GetWindowLongW((*infoPtr).hwndSelf, GWL_STYLE) as u32
        & WS_DISABLED)
        != 0
    {
        hSolidBrush = CreateSolidBrush(comctl32_color.clrBtnFace);
        hBrush = hSolidBrush;
    } else {
        hBrush = SendMessageW(
            (*infoPtr).hwndNotify,
            WM_CTLCOLOREDIT,
            hdc as WPARAM,
            (*infoPtr).hwndSelf as LPARAM,
        ) as HBRUSH;
        if hBrush.is_null() {
            hSolidBrush = CreateSolidBrush(comctl32_color.clrWindow);
            hBrush = hSolidBrush;
        }
    }

    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rc));

    FillRect(hdc, core::ptr::addr_of_mut!(rc), hBrush);

    if !hSolidBrush.is_null() {
        DeleteObject((hSolidBrush).cast());
    }

    return -(1 as ::core::ffi::c_int) as LRESULT;
}

unsafe extern "C" fn HOTKEY_GetFont(infoPtr: *const HOTKEY_INFO) -> LRESULT {
    return (*infoPtr).hFont as LRESULT;
}

unsafe extern "C" fn HOTKEY_KeyDown(infoPtr: *mut HOTKEY_INFO, key: DWORD, flags: DWORD) -> LRESULT {
    let mut wOldHotKey: WORD = 0;
    let mut bOldMod: BYTE = 0;

    if (GetWindowLongW((*infoPtr).hwndSelf, GWL_STYLE) as DWORD & WS_DISABLED as DWORD) != 0 {
        return 0 as LRESULT;
    }

    wOldHotKey = (*infoPtr).HotKey;
    bOldMod = (*infoPtr).CurrMod;

    /* If any key is Pressed, we have to reset the hotkey in the control */
    (*infoPtr).HotKey = 0 as WORD;

    if key == VK_RETURN as DWORD
        || key == VK_TAB as DWORD
        || key == VK_SPACE as DWORD
        || key == VK_DELETE as DWORD
        || key == VK_ESCAPE as DWORD
        || key == VK_BACK as DWORD
    {
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE as BOOL);
        return DefWindowProcW(
            (*infoPtr).hwndSelf,
            WM_KEYDOWN as UINT,
            key as WPARAM,
            flags as LPARAM,
        );
    } else if key == VK_SHIFT as DWORD {
        (*infoPtr).CurrMod |= HOTKEYF_SHIFT as BYTE;
    } else if key == VK_CONTROL as DWORD {
        (*infoPtr).CurrMod |= HOTKEYF_CONTROL as BYTE;
    } else if key == VK_MENU as DWORD {
        (*infoPtr).CurrMod |= HOTKEYF_ALT as BYTE;
    } else {
        if HOTKEY_IsCombInv(infoPtr) != 0 {
            (*infoPtr).HotKey = MAKEWORD!(key, (*infoPtr).InvMod as DWORD) as WORD;
        } else {
            (*infoPtr).HotKey = MAKEWORD!(key, (*infoPtr).CurrMod as DWORD) as WORD;
        }
        (*infoPtr).ScanCode = flags;
    }

    if (wOldHotKey as DWORD != (*infoPtr).HotKey as DWORD) || (bOldMod as DWORD != (*infoPtr).CurrMod as DWORD) {
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE as BOOL);

        /* send EN_CHANGE notification */
        SendMessageW(
            (*infoPtr).hwndNotify,
            WM_COMMAND as UINT,
            MAKEWPARAM!(
                GetDlgCtrlID((*infoPtr).hwndSelf) as WORD,
                EN_CHANGE as WORD,
            ) as WPARAM,
            (*infoPtr).hwndSelf as LPARAM,
        );
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn HOTKEY_KeyUp(infoPtr: *mut HOTKEY_INFO, key: DWORD) -> LRESULT {
    let mut bOldMod: BYTE;

    if (GetWindowLongW((*infoPtr).hwndSelf, GWL_STYLE) as DWORD & WS_DISABLED as DWORD) != 0 {
        return 0 as LRESULT;
    }

    bOldMod = (*infoPtr).CurrMod;

    match key {
        VK_SHIFT => {
            (*infoPtr).CurrMod &= !(HOTKEYF_SHIFT as BYTE);
        }
        VK_CONTROL => {
            (*infoPtr).CurrMod &= !(HOTKEYF_CONTROL as BYTE);
        }
        VK_MENU => {
            (*infoPtr).CurrMod &= !(HOTKEYF_ALT as BYTE);
        }
        _ => {
            return 1 as LRESULT;
        }
    }

    if bOldMod != (*infoPtr).CurrMod {
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), TRUE as BOOL);

        /* send EN_CHANGE notification */
        SendMessageW(
            (*infoPtr).hwndNotify,
            WM_COMMAND as UINT,
            MAKEWPARAM!(GetDlgCtrlID((*infoPtr).hwndSelf) as WORD, EN_CHANGE as WORD) as WPARAM,
            (*infoPtr).hwndSelf as LPARAM,
        );
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn HOTKEY_KillFocus(infoPtr: *mut HOTKEY_INFO) -> LRESULT {
    (*infoPtr).bFocus = FALSE;
    DestroyCaret();

    return 0;
}

unsafe extern "C" fn HOTKEY_LButtonDown(infoPtr: *const HOTKEY_INFO) -> LRESULT {
    if (GetWindowLongW((*infoPtr).hwndSelf, GWL_STYLE) as u32 & WS_DISABLED) == 0 {
        SetFocus((*infoPtr).hwndSelf);
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn HOTKEY_NCCreate(hwnd: HWND, lpcs: *const CREATESTRUCTW) -> LRESULT {
    let mut infoPtr: *mut HOTKEY_INFO = core::ptr::null_mut();
    let mut dwExStyle: DWORD = GetWindowLongW(hwnd, GWL_EXSTYLE) as DWORD;
    SetWindowLongW(
        hwnd,
        GWL_EXSTYLE,
        (dwExStyle | WS_EX_CLIENTEDGE as DWORD) as LONG,
    );

    /* allocate memory for info structure */
    infoPtr = heap_alloc_zero(core::mem::size_of::<HOTKEY_INFO>() as SIZE_T) as *mut HOTKEY_INFO;
    SetWindowLongPtrW(hwnd, 0, infoPtr as DWORD_PTR as LONG_PTR);

    /* initialize info structure */
    (*infoPtr).CurrMod = 0 as BYTE;
    (*infoPtr).InvMod = (*infoPtr).CurrMod as WORD;
    (*infoPtr).InvComb = (*infoPtr).InvMod;
    (*infoPtr).HotKey = (*infoPtr).InvComb;
    (*infoPtr).CaretPos = GetSystemMetrics((SM_CXBORDER) as i32) as INT;
    (*infoPtr).hwndSelf = hwnd;
    LoadStringW(
        COMCTL32_hModule as HINSTANCE,
        HKY_NONE as UINT,
        core::ptr::addr_of_mut!((*infoPtr).strNone) as LPWSTR,
        15 as ::core::ffi::c_int,
    );

    return DefWindowProcW(
        (*infoPtr).hwndSelf,
        WM_NCCREATE as UINT,
        0 as WPARAM,
        lpcs as LPARAM,
    );
}

unsafe extern "C" fn HOTKEY_SetFocus(infoPtr: *mut HOTKEY_INFO) -> LRESULT {
    (*infoPtr).bFocus = TRUE;

    CreateCaret((*infoPtr).hwndSelf, core::ptr::null_mut(), 1, (*infoPtr).nHeight);
    SetCaretPos((*infoPtr).CaretPos, GetSystemMetrics((SM_CYBORDER) as i32));
    ShowCaret((*infoPtr).hwndSelf);

    return 0;
}

unsafe extern "C" fn HOTKEY_SetFont(infoPtr: *mut HOTKEY_INFO, hFont: HFONT, redraw: BOOL) -> LRESULT {
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let mut hdc: HDC;
    let mut hOldFont: HFONT = core::ptr::null_mut();

    (*infoPtr).hFont = hFont;

    hdc = GetDC((*infoPtr).hwndSelf);
    if !(*infoPtr).hFont.is_null() {
        hOldFont = SelectObject(hdc, (*infoPtr).hFont as HGDIOBJ) as HFONT;
    }

    GetTextMetricsW(hdc, core::ptr::addr_of_mut!(tm));
    (*infoPtr).nHeight = tm.tmHeight as INT;

    if !(*infoPtr).hFont.is_null() {
        SelectObject(hdc, hOldFont as HGDIOBJ);
    }
    ReleaseDC((*infoPtr).hwndSelf, hdc);

    if redraw != 0 {
        InvalidateRect((*infoPtr).hwndSelf, core::ptr::null(), TRUE);
    }

    return 0;
}

unsafe extern "system" fn HOTKEY_WindowProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut infoPtr: *mut HOTKEY_INFO =
        GetWindowLongPtrW(hwnd, 0) as *mut HOTKEY_INFO;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/hotkey.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"HOTKEY_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            469 as ::core::ffi::c_int,
            b"hwnd=%p msg=%x wparam=%lx lparam=%lx\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hwnd,
            uMsg,
            wParam,
            lParam,
        );
    }
    if infoPtr.is_null() && (uMsg != WM_NCCREATE) {
        return DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }
    match uMsg {
        HKM_GETHOTKEY => {
            return HOTKEY_GetHotKey(infoPtr);
        }
        HKM_SETHOTKEY => {
            HOTKEY_SetHotKey(infoPtr, wParam as WORD);
        }
        HKM_SETRULES => {
            HOTKEY_SetRules(infoPtr, wParam as WORD, lParam as WORD);
        }
        WM_CHAR | WM_SYSCHAR => {
            return HOTKEY_KeyDown(
                infoPtr,
                MapVirtualKeyW((LOBYTE!(HIWORD!(lParam))) as UINT, 1) as DWORD,
                lParam as DWORD,
            );
        }
        WM_CREATE => {
            return HOTKEY_Create(infoPtr, lParam as LPCREATESTRUCTW as *const CREATESTRUCTW);
        }
        WM_DESTROY => {
            return HOTKEY_Destroy(infoPtr);
        }
        WM_ERASEBKGND => {
            return HOTKEY_EraseBackground(infoPtr, wParam as HDC);
        }
        WM_GETDLGCODE => {
            return (DLGC_WANTCHARS | DLGC_WANTARROWS) as LRESULT;
        }
        WM_GETFONT => {
            return HOTKEY_GetFont(infoPtr);
        }
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            return HOTKEY_KeyDown(infoPtr, wParam as DWORD, lParam as DWORD);
        }
        WM_KEYUP | WM_SYSKEYUP => {
            return HOTKEY_KeyUp(infoPtr, wParam as DWORD);
        }
        WM_KILLFOCUS => {
            return HOTKEY_KillFocus(infoPtr);
        }
        WM_LBUTTONDOWN => {
            return HOTKEY_LButtonDown(infoPtr);
        }
        WM_NCCREATE => {
            return HOTKEY_NCCreate(hwnd, lParam as LPCREATESTRUCTW as *const CREATESTRUCTW);
        }
        WM_PRINTCLIENT | WM_PAINT => {
            HOTKEY_Paint(infoPtr, wParam as HDC);
            return 0;
        }
        WM_SETFOCUS => {
            return HOTKEY_SetFocus(infoPtr);
        }
        WM_SETFONT => {
            return HOTKEY_SetFont(infoPtr, wParam as HFONT, (LOWORD!(lParam)) as BOOL);
        }
        _ => {
            if (uMsg >= WM_USER)
                && (uMsg < WM_APP)
                && !(COMCTL32_IsReflectedMessage(uMsg) != 0)
            {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/comctl32/hotkey.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"HOTKEY_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        533 as ::core::ffi::c_int,
                        b"unknown msg %04x wp=%08lx lp=%08lx\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        uMsg,
                        wParam,
                        lParam,
                    );
                }
            }
            return DefWindowProcW(hwnd, uMsg, wParam, lParam);
        }
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn HOTKEY_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();
    ZeroMemory!(
        core::ptr::addr_of_mut!(wndClass) as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>(),
    );
    wndClass.style = CS_GLOBALCLASS as UINT;
    wndClass.lpfnWndProc =
        Some(HOTKEY_WindowProc);
    wndClass.cbClsExtra = 0 as core::ffi::c_int;
    wndClass.cbWndExtra =
        core::mem::size_of::<*mut HOTKEY_INFO>() as core::ffi::c_int;
    wndClass.hCursor = core::ptr::null_mut();
    wndClass.hbrBackground = core::ptr::null_mut();
    wndClass.lpszClassName = (HOTKEY_CLASSW).as_ptr();
    RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn HOTKEY_Unregister() {
    UnregisterClassW((HOTKEY_CLASSW).as_ptr(), core::ptr::null_mut());
}
