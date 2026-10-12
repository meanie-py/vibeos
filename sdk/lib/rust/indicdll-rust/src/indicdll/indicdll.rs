use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
// indicml.h: `#define INDICATOR_CLASS INDICATOR_CLASSW` (UNICODE), the wide literal L"Indicator".
pub const INDICATOR_CLASS: LPCWSTR = INDICATOR_CLASSW;

#[no_mangle]
pub static mut g_hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut g_hShared: HANDLE = NULL;

#[no_mangle]
pub static mut g_pShared: PSHARED_DATA =
    ::core::ptr::null::<tagSHARED_DATA>() as *mut tagSHARED_DATA;

#[no_mangle]
pub static mut g_hMutex: HANDLE = NULL;

// WINE_DEFAULT_DEBUG_CHANNEL(internat)
pub const __WINE_DEFAULT_DEBUG_CHANNEL: &[u8] = b"internat\0";

#[repr(C)]
pub struct tagSHARED_DATA {
    pub hWinHook: HHOOK,
    pub hShellHook: HHOOK,
    pub hKeyboardHook: HHOOK,
    pub hKbSwitchWnd: HWND,
    pub nHotID: UINT,
    pub dwHotMenuItemData: DWORD_PTR,
}
pub type SHARED_DATA = tagSHARED_DATA;
pub type PSHARED_DATA = *mut SHARED_DATA;

pub const MUTEX_TIMEOUT_MS: i32 = (4 * 1000);

unsafe extern "C" fn EnterProtectedSection() -> BOOL {
    let dwWaitResult: DWORD = WaitForSingleObject(g_hMutex, (MUTEX_TIMEOUT_MS) as DWORD);
    if dwWaitResult == WAIT_OBJECT_0 {
        return (TRUE) as BOOL;
    }

    if dwWaitResult == WAIT_TIMEOUT {
        WARN!("Timeout while waiting for mutex.\n");
    } else {
        WARN!("Failed to acquire mutex. Error code: %lu\n", GetLastError());
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn LeaveProtectedSection() {
    ReleaseMutex(g_hMutex);
}

unsafe extern "C" fn PostMessageToMainWnd(Msg: UINT, wParam: WPARAM, lParam: LPARAM) {
    if !(*g_pShared).hKbSwitchWnd.is_null() {
        PostMessage((*g_pShared).hKbSwitchWnd, Msg, wParam, lParam);
    }
}

unsafe extern "system" fn WinHookProc(mut code: INT, mut wParam: WPARAM, mut lParam: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(
            (*g_pShared).hWinHook,
            code,
            wParam,
            lParam,
        );
    }
    match (code) as u32 {
        HCBT_ACTIVATE | HCBT_SETFOCUS => {
            let mut hwndFocus: HWND = wParam as HWND;
            if !hwndFocus.is_null() && hwndFocus != (*g_pShared).hKbSwitchWnd {
                PostMessageToMainWnd(WM_WINDOW_ACTIVATE as UINT, hwndFocus as WPARAM, 0 as LPARAM);
            }
        }
        _ => {}
    }
    return CallNextHookEx(
        (*g_pShared).hWinHook,
        code,
        wParam,
        lParam,
    );
}

unsafe extern "system" fn ShellHookProc(code: INT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx((*g_pShared).hShellHook, code, wParam, lParam);
    }

    match (code) as u32 {
        HSHELL_WINDOWACTIVATED => {
            PostMessageToMainWnd(WM_WINDOW_ACTIVATE, wParam, 0 as LPARAM);
        }
        HSHELL_LANGUAGE => {
            PostMessageToMainWnd(WM_LANG_CHANGED, wParam, lParam);
        }
        _ => {}
    }

    return CallNextHookEx((*g_pShared).hShellHook, code, wParam, lParam);
}

unsafe extern "C" fn CheckVirtualKey(vKey: UINT, vKey0: UINT, vKey1: UINT, vKey2: UINT) -> BOOL {
    return (vKey == vKey0 || vKey == vKey1 || vKey == vKey2) as BOOL;
}

unsafe extern "system" fn KeyboardProc(code: INT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx((*g_pShared).hKeyboardHook, code, wParam, lParam);
    }

    if code == (HC_ACTION as i32) {
        let mut vKey: UINT = wParam as UINT;
        let mut keyFlags: LONG = HIWORD!(lParam) as LONG;
        if (keyFlags & KF_UP as LONG) == 0 && (keyFlags & KF_REPEAT as LONG) == 0 {
            let mut bShiftPressed: BOOL = ((GetKeyState((VK_SHIFT as i32)) as INT) < 0) as BOOL;
            let mut bAltPressed: BOOL = (((keyFlags & KF_ALTDOWN as LONG) != 0)
                || ((GetKeyState((VK_MENU as i32)) as INT) < 0)) as BOOL;
            let mut bCtrlPressed: BOOL = ((GetKeyState((VK_CONTROL as i32)) as INT) < 0) as BOOL;
            // Detect Alt+Shift and Ctrl+Shift
            if (bAltPressed != 0
                && CheckVirtualKey(vKey, VK_SHIFT as UINT, VK_LSHIFT as UINT, VK_RSHIFT as UINT) != 0)
                || (bShiftPressed != 0
                    && CheckVirtualKey(vKey, VK_MENU as UINT, VK_LMENU as UINT, VK_RMENU as UINT) != 0)
                || (bCtrlPressed != 0
                    && CheckVirtualKey(vKey, VK_SHIFT as UINT, VK_LSHIFT as UINT, VK_RSHIFT as UINT) != 0)
                || (bShiftPressed != 0
                    && CheckVirtualKey(vKey, VK_CONTROL as UINT, VK_LCONTROL as UINT, VK_RCONTROL as UINT) != 0)
            {
                PostMessageToMainWnd(WM_LANG_CHANGED as UINT, 0 as WPARAM, 0 as LPARAM);
            }
        }
    }

    return CallNextHookEx((*g_pShared).hKeyboardHook, code, wParam, lParam);
}

#[no_mangle]
pub unsafe extern "system" fn KbSwitchSetHooks(bDoHook: BOOL) -> BOOL {
    TRACE!("bDoHook: %d\n", bDoHook);
    if EnterProtectedSection() == 0 {
        return (FALSE) as BOOL;
    }
    if bDoHook != 0 {
        (*g_pShared).hWinHook = SetWindowsHookEx(
            (WH_CBT as i32),
            Some(WinHookProc),
            g_hInstance,
            0 as DWORD,
        );
        (*g_pShared).hShellHook = SetWindowsHookEx(
            (WH_SHELL as i32),
            Some(ShellHookProc),
            g_hInstance,
            0 as DWORD,
        );
        (*g_pShared).hKeyboardHook = SetWindowsHookEx(
            (WH_KEYBOARD as i32),
            Some(KeyboardProc),
            g_hInstance,
            0 as DWORD,
        );
        if !(*g_pShared).hWinHook.is_null()
            && !(*g_pShared).hShellHook.is_null()
            && !(*g_pShared).hKeyboardHook.is_null()
        {
            // Find kbswitch window if necessary
            if (*g_pShared).hKbSwitchWnd.is_null()
                || IsWindow((*g_pShared).hKbSwitchWnd) == 0
            {
                (*g_pShared).hKbSwitchWnd = FindWindow(
                    INDICATOR_CLASS,
                    core::ptr::null::<WCHAR>(),
                );
                TRACE!("hKbSwitchWnd: %p\n", (*g_pShared).hKbSwitchWnd);
            }

            LeaveProtectedSection();
            return (TRUE) as BOOL;
        }
    }

    /* Unhook */
    if !(*g_pShared).hKeyboardHook.is_null() {
        UnhookWindowsHookEx((*g_pShared).hKeyboardHook);
        (*g_pShared).hKeyboardHook = core::ptr::null_mut();
    }
    if !(*g_pShared).hShellHook.is_null() {
        UnhookWindowsHookEx((*g_pShared).hShellHook);
        (*g_pShared).hShellHook = core::ptr::null_mut();
    }
    if !(*g_pShared).hWinHook.is_null() {
        UnhookWindowsHookEx((*g_pShared).hWinHook);
        (*g_pShared).hWinHook = core::ptr::null_mut();
    }

    LeaveProtectedSection();
    return (bDoHook == 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetPenMenuData(pnID: PUINT, pdwItemData: PDWORD_PTR) {
    if EnterProtectedSection() != 0 {
        *pnID = (*g_pShared).nHotID;
        *pdwItemData = (*g_pShared).dwHotMenuItemData;
        LeaveProtectedSection();
    } else {
        WARN!("EnterProtectedSection failed\n");
        *pnID = 0;
        *pdwItemData = 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SetPenMenuData(nID: UINT, dwItemData: DWORD_PTR) {
    if EnterProtectedSection() != 0 {
        (*g_pShared).nHotID = nID;
        (*g_pShared).dwHotMenuItemData = dwItemData;
        LeaveProtectedSection();
    } else {
        WARN!("EnterProtectedSection failed\n");
    }
}
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    match dwReason {
        DLL_PROCESS_ATTACH => {
            g_hInstance = hinstDLL;

            let InternatSHData: [u16; 15] = [
                'I' as u16, 'n' as u16, 't' as u16, 'e' as u16, 'r' as u16,
                'n' as u16, 'a' as u16, 't' as u16, 'S' as u16, 'H' as u16,
                'D' as u16, 'a' as u16, 't' as u16, 'a' as u16, 0,
            ];
            g_hShared = CreateFileMapping(
                INVALID_HANDLE_VALUE,
                core::ptr::null_mut(),
                PAGE_READWRITE,
                0,
                core::mem::size_of::<SHARED_DATA>() as DWORD,
                InternatSHData.as_ptr(),
            );
            if g_hShared.is_null() {
                ERR!("!g_hShared\n");
                return (FALSE) as BOOL;
            }

            let bAlreadyExists: BOOL =
                (GetLastError() == ERROR_ALREADY_EXISTS) as BOOL;

            g_pShared = MapViewOfFile(
                g_hShared,
                FILE_MAP_WRITE,
                0,
                0,
                0,
            ) as PSHARED_DATA;
            if g_pShared.is_null() {
                ERR!("!g_pShared\n");
                return (FALSE) as BOOL;
            }

            if bAlreadyExists == 0 {
                ZeroMemory!(
                    g_pShared as LPVOID,
                    core::mem::size_of::<SHARED_DATA>(),
                );
            }

            let INDICDLL_PROTECTED: [u16; 19] = [
                'I' as u16, 'N' as u16, 'D' as u16, 'I' as u16, 'C' as u16,
                'D' as u16, 'L' as u16, 'L' as u16, '_' as u16, 'P' as u16,
                'R' as u16, 'O' as u16, 'T' as u16, 'E' as u16, 'C' as u16,
                'T' as u16, 'E' as u16, 'D' as u16, 0,
            ];
            g_hMutex = CreateMutex(
                core::ptr::null_mut(),
                (FALSE as i32),
                INDICDLL_PROTECTED.as_ptr(),
            );
            if g_hMutex.is_null() {
                ERR!("Failed to create mutex\n");
                return (FALSE) as BOOL;
            }
        }
        DLL_PROCESS_DETACH => {
            if !g_hMutex.is_null() {
                CloseHandle(g_hMutex);
            }
            UnmapViewOfFile(g_pShared as LPVOID);
            CloseHandle(g_hShared);
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
