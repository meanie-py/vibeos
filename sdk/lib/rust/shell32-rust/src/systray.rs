// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_BB39C012AB62: [WCHAR; 14] = [0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x005F, 0x0054, 0x0072, 0x0061, 0x0079, 0x0057, 0x006E, 0x0064, 0x0000];

use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;
use crate::win32::*;

#[no_mangle]
pub unsafe extern "system" fn Shell_NotifyIconA(dwMessage: u32, pnid: *mut NOTIFYICONDATAA) -> BOOL {
    let mut nidW: NOTIFYICONDATAW = core::mem::zeroed();
    ZeroMemory(
        &raw mut nidW as *mut NOTIFYICONDATAW as *mut _,
        core::mem::size_of::<NOTIFYICONDATAW>(),
    );
    nidW.cbSize = core::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nidW.hWnd = (*pnid).hWnd;
    nidW.uID = (*pnid).uID;
    nidW.uFlags = (*pnid).uFlags;
    nidW.uCallbackMessage = (*pnid).uCallbackMessage;
    nidW.hIcon = (*pnid).hIcon;

    let mut cbSize = (*pnid).cbSize;
    let mut dwValidFlags: u32 = NIF_MESSAGE | NIF_ICON | NIF_TIP;

    if (cbSize as usize) == core::mem::size_of::<NOTIFYICONDATAA>() {
        nidW.cbSize = core::mem::size_of::<NOTIFYICONDATAW>() as u32;
        dwValidFlags |= NIF_STATE | NIF_INFO | NIF_GUID;
    } else if (cbSize as usize) == NOTIFYICONDATAA_V3_SIZE as usize {
        nidW.cbSize = NOTIFYICONDATAW_V3_SIZE as u32;
        dwValidFlags |= NIF_STATE | NIF_INFO | NIF_GUID;
    } else if (cbSize as usize) == NOTIFYICONDATAA_V2_SIZE as usize {
        nidW.cbSize = NOTIFYICONDATAW_V2_SIZE as u32;
        dwValidFlags |= NIF_STATE | NIF_INFO;
    } else {
        if (cbSize as usize) != NOTIFYICONDATAA_V1_SIZE as usize {
            cbSize = NOTIFYICONDATAA_V1_SIZE as u32;
        }
        nidW.cbSize = NOTIFYICONDATAW_V1_SIZE as u32;
    }
    nidW.uFlags &= dwValidFlags;

    if (nidW.uFlags & NIF_TIP) != 0 {
        if (cbSize as usize) <= NOTIFYICONDATAA_V1_SIZE as usize {
            MultiByteToWideChar(
                CP_ACP,
                0,
                (*pnid).szTip.as_ptr() as *const _,
                NIDV1_TIP_SIZE_A,
                nidW.szTip.as_mut_ptr(),
                _countof!(nidW.szTip) as INT,
            );
            // Truncate the string
            nidW.szTip[NIDV1_TIP_SIZE_A as usize - 1 as usize] =
                0 as u16;
        } else {
            MultiByteToWideChar(
                CP_ACP,
                0,
                (*pnid).szTip.as_ptr() as *const _,
                -1,
                nidW.szTip.as_mut_ptr(),
                _countof!(nidW.szTip) as INT,
            );
        }
    }

    if (cbSize as usize) >= NOTIFYICONDATAA_V2_SIZE as usize {
        nidW.dwState = (*pnid).dwState;
        nidW.dwStateMask = (*pnid).dwStateMask;
        nidW.uTimeout = (*pnid).uTimeout;
        nidW.dwInfoFlags = (*pnid).dwInfoFlags;

        if (nidW.uFlags & NIF_INFO) != 0 {
            MultiByteToWideChar(
                CP_ACP,
                0,
                (*pnid).szInfo.as_ptr() as *const _,
                -1,
                nidW.szInfo.as_mut_ptr(),
                _countof!(nidW.szInfo) as INT,
            );
            MultiByteToWideChar(
                CP_ACP,
                0,
                (*pnid).szInfoTitle.as_ptr() as *const _,
                -1,
                nidW.szInfoTitle.as_mut_ptr(),
                _countof!(nidW.szInfoTitle) as INT,
            );
        }
    }

    if ((cbSize as usize) >= NOTIFYICONDATAA_V3_SIZE as usize)
        && ((nidW.uFlags & NIF_GUID) != 0)
    {
        nidW.guidItem = (*pnid).guidItem;
    }

    if (cbSize as usize) >= core::mem::size_of::<NOTIFYICONDATAA>() {
        nidW.hBalloonIcon = (*pnid).hBalloonIcon;
    }

    Shell_NotifyIconW(dwMessage, &raw mut nidW)
}

#[no_mangle]
pub unsafe extern "system" fn Shell_NotifyIconW(dwMessage: u32, pnid: *mut NOTIFYICONDATAW) -> BOOL {
    let mut ret: BOOL = FALSE;
    let hShellTrayWnd: HWND;

    hShellTrayWnd = FindWindowW(CPP_COM_WSTR_BB39C012AB62.as_ptr(), core::ptr::null_mut());
    if hShellTrayWnd.is_null() {
        return FALSE;
    }

    let mut cbSize: u32 = (*pnid).cbSize;
    let mut dwValidFlags: u32 = NIF_MESSAGE | NIF_ICON | NIF_TIP;

    if cbSize == core::mem::size_of::<NOTIFYICONDATAW>() as u32 {
        dwValidFlags |= NIF_STATE | NIF_INFO | NIF_GUID;
    } else if cbSize == NOTIFYICONDATAW_V3_SIZE {
        dwValidFlags |= NIF_STATE | NIF_INFO | NIF_GUID;
    } else if cbSize == NOTIFYICONDATAW_V2_SIZE {
        dwValidFlags |= NIF_STATE | NIF_INFO;
    } else {
        if cbSize != NOTIFYICONDATAW_V1_SIZE {
            cbSize = NOTIFYICONDATAW_V1_SIZE;
        }
    }

    let mut tnid: TRAYNOTIFYDATAW = core::mem::zeroed();
    ZeroMemory(
        &raw mut tnid as *mut TRAYNOTIFYDATAW as *mut core::ffi::c_void,
        core::mem::size_of::<TRAYNOTIFYDATAW>() as usize,
    );

    tnid.dwSignature = NI_NOTIFY_SIG;
    tnid.dwMessage = dwMessage;

    CopyMemory(
        &raw mut tnid.nid as *mut NOTIFYICONDATAW as *mut core::ffi::c_void,
        pnid as *const core::ffi::c_void,
        cbSize as usize,
    );

    tnid.nid.cbSize = core::mem::size_of::<NOTIFYICONDATAW>() as u32;
    tnid.nid.uFlags &= dwValidFlags;

    if tnid.nid.uFlags & NIF_TIP != 0 {
        if cbSize <= NOTIFYICONDATAW_V1_SIZE {
            tnid.nid.szTip[(NIDV1_TIP_SIZE_W as usize - 1 as usize)] = 0;
        } else {
            tnid.nid.szTip[(_countof!(tnid.nid.szTip) as usize - 1 as usize)] = 0;
        }
    }

    if tnid.nid.uFlags & NIF_INFO != 0 {
        tnid.nid.szInfo[(_countof!(tnid.nid.szInfo) as usize - 1 as usize)] = 0;
        tnid.nid.szInfoTitle[(_countof!(tnid.nid.szInfoTitle) as usize - 1 as usize)] = 0;
    }

    let mut data: COPYDATASTRUCT = core::mem::zeroed();
    data.dwData = ((TABDMC_NOTIFY) as usize) as _;
    data.cbData = core::mem::size_of::<TRAYNOTIFYDATAW>() as u32;
    data.lpData = &raw const tnid as *const TRAYNOTIFYDATAW as *mut core::ffi::c_void;

    // C: `SendMessageW(hShellTrayWnd, WM_COPYDATA, (WPARAM)pnid->hWnd, (LPARAM)&data)`.
    // Cast to the semantic aliases, not to a concrete width — WPARAM/LPARAM are
    // pointer-width, so `usize`/`isize` would bake i386's 32 bits on amd64/arm64.
    let send_result: LRESULT = SendMessageW(
        hShellTrayWnd,
        WM_COPYDATA,
        (*pnid).hWnd as WPARAM,
        (&raw const data) as LPARAM,
    );

    if send_result != 0 {
        ret = TRUE;
    }

    ret
}
