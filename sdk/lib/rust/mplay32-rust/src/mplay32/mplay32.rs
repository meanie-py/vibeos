// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_D7EA54CEF527: [u16; 7] = [0x0053u16, 0x0054, 0x0041, 0x0054, 0x0049, 0x0043, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut hTrackBar: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hToolBar: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hTimeDisplay: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hMainMenu: HMENU = ::core::ptr::null::<HMENU__>() as *mut HMENU__;

#[no_mangle]
pub static mut szAppTitle: [TCHAR; 256] = unsafe {
    ::core::mem::transmute::<
        [u8; 512],
        [TCHAR; 256],
    >(
        *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    )
};

#[no_mangle]
pub static mut szDefaultFilter: [TCHAR; 260] = unsafe {
    ::core::mem::transmute::<
        [u8; 520],
        [TCHAR; 260],
    >(
        *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    )
};

#[no_mangle]
pub static mut szCurrentFile: [TCHAR; 260] = unsafe {
    ::core::mem::transmute::<
        [u8; 520],
        [TCHAR; 260],
    >(
        *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    )
};

#[no_mangle]
pub static mut szFilter: LPTSTR = ::core::ptr::null::<WCHAR>() as *mut WCHAR;

#[no_mangle]
pub static mut wDeviceId: WORD = 0 as WORD;

#[no_mangle]
pub static mut bRepeat: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut bIsSingleWindow: BOOL = (FALSE) as i32;

#[no_mangle]
pub static mut MaxFilePos: UINT = 0 as UINT;

#[no_mangle]
pub static mut PrevWindowPos: RECT = tagRECT {
    left: 0,
    top: 0,
    right: 0,
    bottom: 0,
};

static mut Buttons: [TBBUTTON; 8] = [
    _TBBUTTON {
        iBitmap: (TBICON_PLAY as i32),
        idCommand: (IDC_PLAY as i32),
        fsState: TBSTATE_ENABLED as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: (TBICON_STOP as i32),
        idCommand: (IDC_STOP as i32),
        fsState: TBSTATE_ENABLED as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: (TBICON_EJECT as i32),
        idCommand: (IDC_EJECT as i32),
        fsState: TBSTATE_ENABLED as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: 15 as ::core::ffi::c_int,
        idCommand: 0 as ::core::ffi::c_int,
        fsState: TBSTATE_ENABLED as BYTE,
        fsStyle: BTNS_SEP as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: (TBICON_BACKWARD as i32),
        idCommand: (IDC_BACKWARD as i32),
        fsState: TBSTATE_ENABLED as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: (TBICON_SEEKBACK as i32),
        idCommand: (IDC_SEEKBACK as i32),
        fsState: TBSTATE_ENABLED as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: (TBICON_SEEKFORW as i32),
        idCommand: (IDC_SEEKFORW as i32),
        fsState: TBSTATE_ENABLED as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
    _TBBUTTON {
        iBitmap: (TBICON_FORWARD as i32),
        idCommand: (IDC_FORWARD as i32),
        fsState: TBSTATE_ENABLED as BYTE,
        fsStyle: BTNS_BUTTON as BYTE,
        bReserved: unsafe { ::core::mem::zeroed() },
        dwData: 0 as DWORD_PTR,
        iString: 0 as INT_PTR,
    },
];

pub const IDT_PLAYTIMER: u32 = 1000;

pub const MAIN_WINDOW_HEIGHT: i32 = 125;

pub const MAIN_WINDOW_MIN_WIDTH: i32 = 250;

pub const MAX_MCISTR: i32 = 256;

// static DWORD GetDeviceMode(HWND hwnd);  -- prototype only; defined with its function translation
#[no_mangle]
pub unsafe extern "C" fn EnableMenuItems(mut hwnd: HWND) {
    let mut mciError: MCIERROR = 0;
    let mut mciGeneric: MCI_GENERIC_PARMS = core::mem::zeroed();
    let mut mciVideoRect: MCI_DGV_RECT_PARMS = core::mem::zeroed();
    let mut mciVideoWindow: MCI_DGV_WINDOW_PARMSW = core::mem::zeroed();

    EnableMenuItem(
        hMainMenu,
        IDM_CLOSE_FILE as UINT,
        (MF_BYCOMMAND | MF_ENABLED) as UINT,
    );

    mciError = mciSendCommand(
        wDeviceId as MCIDEVICEID,
        MCI_CONFIGURE as UINT,
        MCI_TEST as DWORD_PTR,
        &raw mut mciGeneric as DWORD_PTR,
    );
    if mciError == 0 as MCIERROR {
        EnableMenuItem(
            hMainMenu,
            IDM_DEVPROPS as UINT,
            (MF_BYCOMMAND | MF_ENABLED) as UINT,
        );
    }

    mciVideoWindow.hWnd = hwnd;

    mciError = mciSendCommand(
        wDeviceId as MCIDEVICEID,
        MCI_WINDOW as UINT,
        (MCI_DGV_WINDOW_HWND | MCI_TEST) as DWORD_PTR,
        &raw mut mciVideoWindow as DWORD_PTR,
    );
    if mciError == 0 {
        mciError = mciSendCommand(
            wDeviceId as MCIDEVICEID,
            MCI_WHERE as UINT,
            (MCI_DGV_WHERE_SOURCE | MCI_TEST) as DWORD_PTR,
            &raw mut mciVideoRect as DWORD_PTR,
        );
        if mciError == 0 {
            EnableMenuItem(
                hMainMenu,
                IDM_SWITCHVIEW as UINT,
                (MF_BYCOMMAND | MF_ENABLED) as UINT,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn DisableMenuItems() {
    EnableMenuItem(
        hMainMenu,
        IDM_CLOSE_FILE as UINT,
        (MF_BYCOMMAND | MF_GRAYED) as UINT,
    );
    EnableMenuItem(
        hMainMenu,
        IDM_DEVPROPS as UINT,
        (MF_BYCOMMAND | MF_GRAYED) as UINT,
    );
    EnableMenuItem(
        hMainMenu,
        IDM_SWITCHVIEW as UINT,
        (MF_BYCOMMAND | MF_GRAYED) as UINT,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ResizeClientArea(hwnd: HWND, nWidth: i32, nHeight: i32) {
    let mut rcClientRect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut rcWindowRect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut ptDifference: POINT = POINT { x: 0, y: 0 };
    GetClientRect(hwnd, &raw mut rcClientRect);
    GetWindowRect(hwnd, &raw mut rcWindowRect);
    ptDifference.x = (rcWindowRect.right - rcWindowRect.left) - rcClientRect.right;
    ptDifference.y = (rcWindowRect.bottom - rcWindowRect.top) - rcClientRect.bottom;
    MoveWindow(
        hwnd,
        rcWindowRect.left,
        rcWindowRect.top,
        nWidth + ptDifference.x,
        nHeight + ptDifference.y,
        (TRUE) as BOOL,
    );
}
#[no_mangle]
pub unsafe extern "C" fn UpdateWindowCaption(hwnd: HWND) {
    let mut szNewTitle: [TCHAR; (MAX_PATH + 3 + 256) as _] = [0; (MAX_PATH + 3 + 256) as _];
    let mut szStatus: [TCHAR; 128] = [0; 128];

    if wDeviceId as i32 == 0 {
        SetWindowText(hwnd, core::ptr::addr_of_mut!(szAppTitle) as *mut TCHAR);
        return;
    }

    match GetDeviceMode(hwnd) {
        MCI_MODE_PAUSE => {
            LoadString(
                hInstance,
                IDS_MODE_PAUSE as UINT,
                core::ptr::addr_of_mut!(szStatus) as LPTSTR,
                ARRAYSIZE!(szStatus) as i32,
            );
        }

        MCI_MODE_STOP => {
            LoadString(
                hInstance,
                IDS_MODE_STOP as UINT,
                core::ptr::addr_of_mut!(szStatus) as LPTSTR,
                ARRAYSIZE!(szStatus) as i32,
            );
        }

        MCI_MODE_PLAY => {
            LoadString(
                hInstance,
                IDS_MODE_PLAY as UINT,
                core::ptr::addr_of_mut!(szStatus) as LPTSTR,
                ARRAYSIZE!(szStatus) as i32,
            );
        }

        MCI_MODE_OPEN => {
            LoadString(
                hInstance,
                IDS_MODE_OPEN as UINT,
                core::ptr::addr_of_mut!(szStatus) as LPTSTR,
                ARRAYSIZE!(szStatus) as i32,
            );
        }

        MCI_MODE_RECORD => {
            LoadString(
                hInstance,
                IDS_MODE_RECORD as UINT,
                core::ptr::addr_of_mut!(szStatus) as LPTSTR,
                ARRAYSIZE!(szStatus) as i32,
            );
        }

        MCI_MODE_SEEK => {
            LoadString(
                hInstance,
                IDS_MODE_SEEK as UINT,
                core::ptr::addr_of_mut!(szStatus) as LPTSTR,
                ARRAYSIZE!(szStatus) as i32,
            );
        }

        MCI_MODE_NOT_READY => {
            LoadString(
                hInstance,
                IDS_MODE_NOT_READY as UINT,
                core::ptr::addr_of_mut!(szStatus) as LPTSTR,
                ARRAYSIZE!(szStatus) as i32,
            );
        }

        _ => {
            LoadString(
                hInstance,
                IDS_MODE_UNKNOWN as UINT,
                core::ptr::addr_of_mut!(szStatus) as LPTSTR,
                ARRAYSIZE!(szStatus) as i32,
            );
        }
    }

    StringCbPrintf!(
        core::ptr::addr_of_mut!(szNewTitle) as LPTSTR,
        core::mem::size_of::<[TCHAR; (MAX_PATH + 3 + 256) as _]>() as i32,
        (&[0x0025, 0x0073, 0x0020, 0x002d, 0x0020, 0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0000u16]).as_ptr() /* _T "%s - %s (%s)" */,
        core::ptr::addr_of_mut!(szAppTitle) as *mut TCHAR,
        core::ptr::addr_of_mut!(szCurrentFile) as *mut TCHAR,
        core::ptr::addr_of_mut!(szStatus) as *mut TCHAR,
    );
    SetWindowText(hwnd, core::ptr::addr_of_mut!(szNewTitle) as *mut TCHAR as LPCWSTR);
}
#[no_mangle]
pub unsafe extern "C" fn UpdateTimeDisplay(hwnd: HWND) {
    let mut mciStatus: MCI_STATUS_PARMS = core::mem::zeroed();
    let mut szTime: [TCHAR; (MAX_MCISTR) as usize] = [0 as TCHAR; (MAX_MCISTR) as usize];
    let mut dwTimeFormat: DWORD;

    if wDeviceId == 0 {
        let empty: [TCHAR; 1] = [0 as TCHAR];
        SetWindowText(hwnd, empty.as_ptr());
        return;
    }

    mciStatus.dwItem = MCI_STATUS_TIME_FORMAT as DWORD;
    mciStatus.dwReturn = 0 as DWORD_PTR;
    mciSendCommand(
        wDeviceId as MCIDEVICEID,
        MCI_STATUS as UINT,
        MCI_STATUS_ITEM as DWORD_PTR,
        &mut mciStatus as *mut MCI_STATUS_PARMS as DWORD_PTR,
    );
    dwTimeFormat = mciStatus.dwReturn as DWORD;

    mciStatus.dwItem = MCI_STATUS_POSITION as DWORD;
    mciStatus.dwReturn = 0 as DWORD_PTR;
    mciSendCommand(
        wDeviceId as MCIDEVICEID,
        MCI_STATUS as UINT,
        MCI_STATUS_ITEM as DWORD_PTR,
        &mut mciStatus as *mut MCI_STATUS_PARMS as DWORD_PTR,
    );

    match dwTimeFormat {
        MCI_FORMAT_MILLISECONDS => 'block: {
            let mut s: i32;
            let mut m: i32;
            let mut h: i32;

            s = ((mciStatus.dwReturn / 1000) % 60) as i32;
            m = (((mciStatus.dwReturn / (1000 * 60)) % 60)) as i32;
            h = (((mciStatus.dwReturn / (1000 * 60 * 60)) % 24)) as i32;
            let fmt: [TCHAR; 18] = [
                '%' as TCHAR, '0' as TCHAR, '2' as TCHAR, 'l' as TCHAR, 'u' as TCHAR, ':' as TCHAR,
                '%' as TCHAR, '0' as TCHAR, '2' as TCHAR, 'l' as TCHAR, 'u' as TCHAR, ':' as TCHAR,
                '%' as TCHAR, '0' as TCHAR, '2' as TCHAR, 'l' as TCHAR, 'u' as TCHAR, 0 as TCHAR,
            ];
            StringCbPrintf!(
                szTime.as_mut_ptr(),
                core::mem::size_of::<[TCHAR; (MAX_MCISTR) as usize]>(),
                fmt.as_ptr(),
                h,
                m,
                s,
            );
            break 'block;
        }

        _ => 'block: {
            let fmt: [TCHAR; 4] = ['%' as TCHAR, 'l' as TCHAR, 'u' as TCHAR, 0 as TCHAR];
            StringCbPrintf!(
                szTime.as_mut_ptr(),
                core::mem::size_of::<[TCHAR; (MAX_MCISTR) as usize]>(),
                fmt.as_ptr(),
                mciStatus.dwReturn,
            );
            break 'block;
        }
    }

    SetWindowText(hwnd, szTime.as_ptr());
}

unsafe extern "C" fn ShowLastWin32Error(hwnd: HWND) {
    let mut lpMessageBuffer: LPTSTR = core::ptr::null_mut();
    let dwError: DWORD = GetLastError();

    if dwError == ERROR_SUCCESS {
        return;
    }

    if FormatMessage(
        FORMAT_MESSAGE_ALLOCATE_BUFFER |
        FORMAT_MESSAGE_FROM_SYSTEM |
        FORMAT_MESSAGE_IGNORE_INSERTS,
        core::ptr::null_mut(),
        dwError,
        LANG_USER_DEFAULT,
        core::ptr::addr_of_mut!(lpMessageBuffer) as LPTSTR,
        0,
        core::ptr::null_mut(),
    ) == 0
    {
        return;
    }

    MessageBox(
        hwnd,
        lpMessageBuffer,
        core::ptr::addr_of_mut!(szAppTitle) as *mut TCHAR as LPTSTR,
        MB_OK | MB_ICONERROR,
    );
    LocalFree((lpMessageBuffer).cast());
}

unsafe extern "C" fn SetImageList(hwnd: HWND) {
    let mut hImageList: HIMAGELIST = ImageList_Create(16, 16, ILC_MASK | ILC_COLOR24, 1, 1);
    if hImageList.is_null() {
        ShowLastWin32Error(hwnd);
        return;
    }

    ImageList_AddMasked(hImageList,
                        (LoadImage(hInstance, MAKEINTRESOURCE!(IDB_PLAYICON), IMAGE_BITMAP, 16, 16, LR_DEFAULTCOLOR)).cast(),
                        RGB!(255, 255, 255));

    ImageList_AddMasked(hImageList,
                        (LoadImage(hInstance, MAKEINTRESOURCE!(IDB_STOPICON), IMAGE_BITMAP, 16, 16, LR_DEFAULTCOLOR)).cast(),
                        RGB!(255, 255, 255));

    ImageList_AddMasked(hImageList,
                        (LoadImage(hInstance, MAKEINTRESOURCE!(IDB_EJECTICON), IMAGE_BITMAP, 16, 16, LR_DEFAULTCOLOR)).cast(),
                        RGB!(255, 255, 255));

    ImageList_AddMasked(hImageList,
                        (LoadImage(hInstance, MAKEINTRESOURCE!(IDB_BACKWARDICON), IMAGE_BITMAP, 16, 16, LR_DEFAULTCOLOR)).cast(),
                        RGB!(255, 255, 255));

    ImageList_AddMasked(hImageList,
                        (LoadImage(hInstance, MAKEINTRESOURCE!(IDB_SEEKBACKICON), IMAGE_BITMAP, 16, 16, LR_DEFAULTCOLOR)).cast(),
                        RGB!(255, 255, 255));

    ImageList_AddMasked(hImageList,
                        (LoadImage(hInstance, MAKEINTRESOURCE!(IDB_SEEKFORWICON), IMAGE_BITMAP, 16, 16, LR_DEFAULTCOLOR)).cast(),
                        RGB!(255, 255, 255));

    ImageList_AddMasked(hImageList,
                        (LoadImage(hInstance, MAKEINTRESOURCE!(IDB_FORWARDICON), IMAGE_BITMAP, 16, 16, LR_DEFAULTCOLOR)).cast(),
                        RGB!(255, 255, 255));

    ImageList_AddMasked(hImageList,
                        (LoadImage(hInstance, MAKEINTRESOURCE!(IDB_PAUSEICON), IMAGE_BITMAP, 16, 16, LR_DEFAULTCOLOR)).cast(),
                        RGB!(255, 255, 255));

    ImageList_Destroy(SendMessage(hToolBar,
                                              TB_SETIMAGELIST,
                                              0,
                                              hImageList as LPARAM) as HIMAGELIST);
}

unsafe extern "C" fn ShowMCIError(hwnd: HWND, mciError: MCIERROR) {
    let mut szErrorMessage: [TCHAR; (MAX_MCISTR) as usize] = [0; (MAX_MCISTR) as usize];
    let mut szTempMessage: [TCHAR; (MAX_MCISTR + 44) as usize] = [0; (MAX_MCISTR + 44) as usize];
    let szFmt: [u16; 13] = ['M' as u16, 'M' as u16, 'S' as u16, 'Y' as u16, 'S' as u16, '%' as u16, 'l' as u16, 'u' as u16, ':' as u16, ' ' as u16, '%' as u16, 's' as u16, 0];

    if (mciGetErrorString(mciError, core::ptr::addr_of_mut!(szErrorMessage) as LPTSTR, ARRAYSIZE!(szErrorMessage) as UINT)) as u32 == FALSE
    {
        LoadString(hInstance, IDS_DEFAULTMCIERRMSG as UINT, core::ptr::addr_of_mut!(szErrorMessage) as LPTSTR, ARRAYSIZE!(szErrorMessage) as core::ffi::c_int);
    }

    StringCbPrintf!(core::ptr::addr_of_mut!(szTempMessage) as *mut TCHAR, core::mem::size_of::<[TCHAR; (MAX_MCISTR + 44) as usize]>(), szFmt.as_ptr() as *const TCHAR, mciError, core::ptr::addr_of_mut!(szErrorMessage) as *mut TCHAR);
    MessageBox(hwnd, core::ptr::addr_of_mut!(szTempMessage) as *mut TCHAR as LPCWSTR, core::ptr::addr_of_mut!(szAppTitle) as *mut TCHAR as LPCWSTR, (MB_OK | MB_ICONEXCLAMATION) as UINT);
}

unsafe extern "C" fn InitControls(hwnd: HWND) {
    let mut NumButtons: INT = (::core::mem::size_of::<[TBBUTTON; 8]>() / ::core::mem::size_of::<TBBUTTON>()) as INT;

    InitCommonControls();

    /* Create trackbar */
    hTrackBar = CreateWindowEx(
        0 as DWORD,
        TRACKBAR_CLASS as LPCWSTR,
        core::ptr::null::<WCHAR>(),
        (TBS_ENABLESELRANGE | WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_CLIPSIBLINGS) as DWORD,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        340 as ::core::ffi::c_int,
        20 as ::core::ffi::c_int,
        hwnd,
        core::ptr::null_mut::<HMENU__>(),
        hInstance,
        core::ptr::null_mut(),
    );
    if hTrackBar.is_null() {
        ShowLastWin32Error(hwnd);
        return;
    }

    /* Create toolbar */
    hToolBar = CreateWindowEx(
        0 as DWORD,
        TOOLBARCLASSNAME as LPCWSTR,
        core::ptr::null::<WCHAR>(),
        (WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_CLIPSIBLINGS |
         TBSTYLE_FLAT | CCS_BOTTOM | TBSTYLE_TOOLTIPS) as DWORD,
        0 as ::core::ffi::c_int,
        40 as ::core::ffi::c_int,
        340 as ::core::ffi::c_int,
        30 as ::core::ffi::c_int,
        hwnd,
        core::ptr::null_mut::<HMENU__>(),
        hInstance,
        core::ptr::null_mut(),
    );
    if hToolBar.is_null() {
        ShowLastWin32Error(hwnd);
        return;
    }

    hTimeDisplay = CreateWindowEx(
        0 as DWORD,
        RUSTCONV_WSTR_D7EA54CEF527.as_ptr() as LPCWSTR,
        core::ptr::null::<WCHAR>(),
        (WS_CHILD | WS_VISIBLE | SS_CENTER | SS_SUNKEN) as DWORD,
        195 as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
        135 as ::core::ffi::c_int,
        18 as ::core::ffi::c_int,
        hToolBar,
        core::ptr::null_mut::<HMENU__>(),
        hInstance,
        core::ptr::null_mut(),
    );
    if hTimeDisplay.is_null() {
        ShowLastWin32Error(hwnd);
        return;
    }

    SetImageList(hwnd);
    SendMessage(
        hToolBar,
        TB_ADDBUTTONS as UINT,
        NumButtons as WPARAM,
        core::ptr::addr_of!(Buttons) as *const TBBUTTON as LPARAM,
    );
}

unsafe extern "C" fn SwitchViewMode(hwnd: HWND) {
    let mut mciError: MCIERROR = 0;
    let mut mciVideoRect: MCI_DGV_RECT_PARMS = core::mem::zeroed::<MCI_DGV_RECT_PARMS>();
    let mut mciVideoWindow: MCI_DGV_WINDOW_PARMSW = core::mem::zeroed::<MCI_DGV_WINDOW_PARMSW>();
    let mut rcToolbarRect: RECT = core::mem::zeroed::<RECT>();
    let mut rcTempRect: RECT = core::mem::zeroed::<RECT>();

    mciVideoWindow.hWnd = hwnd;

    mciError = mciSendCommand(
        wDeviceId as MCIDEVICEID,
        MCI_WINDOW as UINT,
        (MCI_DGV_WINDOW_HWND | MCI_TEST) as DWORD_PTR,
        core::ptr::addr_of_mut!(mciVideoWindow) as DWORD_PTR,
    );
    if mciError != 0 as MCIERROR {
        return;
    }

    mciError = mciSendCommand(
        wDeviceId as MCIDEVICEID,
        MCI_WHERE as UINT,
        (MCI_DGV_WHERE_SOURCE | MCI_TEST) as DWORD_PTR,
        core::ptr::addr_of_mut!(mciVideoRect) as DWORD_PTR,
    );
    if mciError != 0 as MCIERROR {
        return;
    }

    if bIsSingleWindow == 0 {
        GetWindowRect(hwnd, core::ptr::addr_of_mut!(PrevWindowPos));

        SetParent(hTrackBar, hToolBar);

        mciError = mciSendCommand(
            wDeviceId as MCIDEVICEID,
            MCI_WHERE as UINT,
            MCI_DGV_WHERE_SOURCE as DWORD_PTR,
            core::ptr::addr_of_mut!(mciVideoRect) as DWORD_PTR,
        );
        if mciError != 0 as MCIERROR {
            ShowMCIError(hwnd, mciError);
            return;
        }

        GetWindowRect(hToolBar, core::ptr::addr_of_mut!(rcToolbarRect));
        ResizeClientArea(
            hwnd,
            mciVideoRect.rc.right as core::ffi::c_int,
            (mciVideoRect.rc.bottom + (rcToolbarRect.bottom - rcToolbarRect.top)) as core::ffi::c_int,
        );

        mciError = mciSendCommand(
            wDeviceId as MCIDEVICEID,
            MCI_WINDOW as UINT,
            MCI_DGV_WINDOW_HWND as DWORD_PTR,
            core::ptr::addr_of_mut!(mciVideoWindow) as DWORD_PTR,
        );
        if mciError != 0 as MCIERROR {
            ShowMCIError(hwnd, mciError);
            return;
        }

        GetWindowRect(hToolBar, core::ptr::addr_of_mut!(rcTempRect));
        MoveWindow(
            hTrackBar,
            180 as core::ffi::c_int,
            0 as core::ffi::c_int,
            (rcTempRect.right - rcTempRect.left - 322 as LONG) as core::ffi::c_int,
            25 as core::ffi::c_int,
            (TRUE) as BOOL,
        );
        MoveWindow(
            hTimeDisplay,
            (rcTempRect.right - rcTempRect.left - 140 as LONG) as core::ffi::c_int,
            4 as core::ffi::c_int,
            135 as core::ffi::c_int,
            18 as core::ffi::c_int,
            (TRUE) as BOOL,
        );

        CheckMenuItem(
            hMainMenu,
            IDM_SWITCHVIEW as UINT,
            (MF_BYCOMMAND | MF_CHECKED) as UINT,
        );
        bIsSingleWindow = (TRUE) as i32;
    } else {
        bIsSingleWindow = (FALSE) as i32;
        CheckMenuItem(
            hMainMenu,
            IDM_SWITCHVIEW as UINT,
            (MF_BYCOMMAND | MF_UNCHECKED) as UINT,
        );

        mciVideoWindow.hWnd = MCI_DGV_WINDOW_DEFAULT as usize as HWND;
        mciError = mciSendCommand(
            wDeviceId as MCIDEVICEID,
            MCI_WINDOW as UINT,
            MCI_DGV_WINDOW_HWND as DWORD_PTR,
            core::ptr::addr_of_mut!(mciVideoWindow) as DWORD_PTR,
        );
        if mciError != 0 as MCIERROR {
            ShowMCIError(hwnd, mciError);
            return;
        }

        SetParent(hTrackBar, hwnd);

        MoveWindow(
            hwnd,
            PrevWindowPos.left as core::ffi::c_int,
            PrevWindowPos.top as core::ffi::c_int,
            (PrevWindowPos.right - PrevWindowPos.left) as core::ffi::c_int,
            (PrevWindowPos.bottom - PrevWindowPos.top) as core::ffi::c_int,
            (TRUE) as BOOL,
        );
    }
}

unsafe extern "C" fn GetNumDevices() -> DWORD {
    let mut mciSysInfo: MCI_SYSINFO_PARMS = core::mem::zeroed();
    let mut dwNumDevices: DWORD = 0;

    mciSysInfo.dwCallback = 0;
    mciSysInfo.lpstrReturn = core::ptr::addr_of_mut!(dwNumDevices) as LPTSTR;
    mciSysInfo.dwRetSize = core::mem::size_of::<DWORD>() as DWORD;
    mciSysInfo.dwNumber = 0;
    mciSysInfo.wDeviceType = MCI_ALL_DEVICE_ID;

    mciSendCommand(
        MCI_ALL_DEVICE_ID as UINT,
        MCI_SYSINFO as UINT,
        MCI_SYSINFO_QUANTITY as DWORD_PTR,
        core::ptr::addr_of_mut!(mciSysInfo) as DWORD_PTR,
    );

    return *(mciSysInfo.lpstrReturn as *mut DWORD);
}

static GetDeviceName: unsafe extern "C" fn(DWORD, LPTSTR, DWORD) -> DWORD = {
    unsafe extern "C" fn GetDeviceName(dwDeviceIndex: DWORD, lpDeviceName: LPTSTR, dwDeviceNameSize: DWORD) -> DWORD {
        let mut mciSysInfo: MCI_SYSINFO_PARMS = core::mem::zeroed();

        mciSysInfo.dwCallback = 0;
        mciSysInfo.lpstrReturn = lpDeviceName;
        mciSysInfo.dwRetSize = dwDeviceNameSize;
        mciSysInfo.dwNumber = dwDeviceIndex;
        mciSysInfo.wDeviceType = MCI_DEVTYPE_WAVEFORM_AUDIO;

        return mciSendCommand(
            MCI_ALL_DEVICE_ID,
            MCI_SYSINFO,
            MCI_SYSINFO_NAME as DWORD_PTR,
            core::ptr::addr_of_mut!(mciSysInfo) as DWORD_PTR,
        ) as DWORD;
    }
    GetDeviceName
};

unsafe extern "C" fn GetDeviceFriendlyName(
    mut lpDeviceName: LPTSTR,
    mut lpFriendlyName: LPTSTR,
    mut dwFriendlyNameSize: DWORD,
) -> DWORD {
    let mut mciError: MCIERROR = 0;
    let mut mciOpen: MCI_OPEN_PARMS = core::mem::zeroed();
    let mut mciInfo: MCI_INFO_PARMS = core::mem::zeroed();
    let mut mciGeneric: MCI_GENERIC_PARMS = core::mem::zeroed();

    mciOpen.dwCallback = 0 as DWORD_PTR;
    mciOpen.wDeviceID = 0 as MCIDEVICEID;
    mciOpen.lpstrDeviceType = lpDeviceName as LPCWSTR;
    mciOpen.lpstrElementName = core::ptr::null::<WCHAR>();
    mciOpen.lpstrAlias = core::ptr::null::<WCHAR>();

    mciError = mciSendCommand(
        0 as MCIDEVICEID,
        MCI_OPEN as UINT,
        (MCI_OPEN_TYPE | MCI_WAIT) as DWORD_PTR,
        core::ptr::addr_of_mut!(mciOpen) as DWORD_PTR,
    );
    if mciError != 0 as MCIERROR {
        return mciError as DWORD;
    }

    mciInfo.dwCallback = 0 as DWORD_PTR;
    mciInfo.lpstrReturn = lpFriendlyName as LPWSTR;
    mciInfo.dwRetSize = dwFriendlyNameSize;

    mciError = mciSendCommand(
        mciOpen.wDeviceID,
        MCI_INFO as UINT,
        MCI_INFO_PRODUCT as DWORD_PTR,
        core::ptr::addr_of_mut!(mciInfo) as DWORD_PTR,
    );

    mciGeneric.dwCallback = 0 as DWORD_PTR;
    mciSendCommand(
        mciOpen.wDeviceID,
        MCI_CLOSE as UINT,
        MCI_WAIT as DWORD_PTR,
        core::ptr::addr_of_mut!(mciGeneric) as DWORD_PTR,
    );

    return mciError as DWORD;
}

unsafe extern "C" fn DeviceUsesFiles(mut lpDeviceName: LPTSTR) -> BOOL {
    let mut mciError: MCIERROR = 0;
    let mut mciOpen: MCI_OPEN_PARMS = core::mem::zeroed();
    let mut mciDevCaps: MCI_GETDEVCAPS_PARMS = core::mem::zeroed();
    let mut mciGeneric: MCI_GENERIC_PARMS = core::mem::zeroed();

    mciOpen.dwCallback = 0 as DWORD_PTR;
    mciOpen.wDeviceID = 0 as MCIDEVICEID;
    mciOpen.lpstrDeviceType = lpDeviceName as LPCWSTR;
    mciOpen.lpstrElementName = core::ptr::null::<WCHAR>();
    mciOpen.lpstrAlias = core::ptr::null::<WCHAR>();

    mciError = mciSendCommand(
        0 as MCIDEVICEID,
        MCI_OPEN as UINT,
        (MCI_OPEN_TYPE | MCI_WAIT) as DWORD_PTR,
        core::ptr::addr_of_mut!(mciOpen) as DWORD_PTR,
    );
    if mciError != 0 as MCIERROR {
        return (FALSE) as BOOL;
    }

    mciDevCaps.dwCallback = 0 as DWORD_PTR;
    mciDevCaps.dwReturn = 0 as DWORD;
    mciDevCaps.dwItem = MCI_GETDEVCAPS_USES_FILES as DWORD;

    mciError = mciSendCommand(
        mciOpen.wDeviceID,
        MCI_GETDEVCAPS as UINT,
        (MCI_WAIT | MCI_GETDEVCAPS_ITEM) as DWORD_PTR,
        core::ptr::addr_of_mut!(mciDevCaps) as DWORD_PTR,
    );
    if mciError != 0 as MCIERROR {
        return (FALSE) as BOOL;
    }

    mciGeneric.dwCallback = 0 as DWORD_PTR;
    mciSendCommand(
        mciOpen.wDeviceID,
        MCI_CLOSE as UINT,
        MCI_WAIT as DWORD_PTR,
        core::ptr::addr_of_mut!(mciGeneric) as DWORD_PTR,
    );

    return mciDevCaps.dwReturn as BOOL;
}

unsafe extern "C" fn CloseMciDevice() -> MCIERROR {
    let mut mciError: MCIERROR = 0;
    let mut mciGeneric: MCI_GENERIC_PARMS = core::mem::zeroed();
    if wDeviceId != 0 {
        mciError = mciSendCommand(
            wDeviceId as MCIDEVICEID,
            MCI_CLOSE as UINT,
            MCI_WAIT as DWORD_PTR,
            core::ptr::addr_of_mut!(mciGeneric) as DWORD_PTR,
        );
        if mciError != 0 as MCIERROR {
            return mciError;
        }
        wDeviceId = 0 as WORD;
    }

    UpdateTimeDisplay(hTimeDisplay);

    DisableMenuItems();

    return 0 as MCIERROR;
}

unsafe extern "C" fn OpenMciDevice(hwnd: HWND, lpType: LPTSTR, lpFileName: LPTSTR) -> MCIERROR {
    let mut mciError: MCIERROR = 0;
    let mut mciStatus: MCI_STATUS_PARMS = core::mem::zeroed();
    let mut mciOpen: MCI_OPEN_PARMS = core::mem::zeroed();
    let mut dwFlags: DWORD = MCI_WAIT as DWORD;
    let mut lpStr: LPTSTR = core::ptr::null_mut();

    if wDeviceId != 0 {
        CloseMciDevice();
    }

    mciOpen.lpstrDeviceType = lpType as LPCWSTR;
    mciOpen.lpstrElementName = lpFileName as LPCWSTR;
    mciOpen.dwCallback = 0 as DWORD_PTR;
    mciOpen.wDeviceID = 0 as MCIDEVICEID;
    mciOpen.lpstrAlias = core::ptr::null::<WCHAR>();

    if !lpType.is_null() {
        dwFlags |= MCI_OPEN_TYPE as DWORD;
    }

    if !lpFileName.is_null() {
        dwFlags |= MCI_OPEN_ELEMENT as DWORD;
    }

    mciError = mciSendCommand(
        0 as MCIDEVICEID,
        MCI_OPEN as UINT,
        dwFlags as DWORD_PTR,
        core::ptr::addr_of_mut!(mciOpen) as DWORD_PTR,
    );
    if mciError != 0 as MCIERROR {
        return mciError;
    }

    mciStatus.dwItem = MCI_STATUS_LENGTH as DWORD;

    mciError = mciSendCommand(
        mciOpen.wDeviceID,
        MCI_STATUS as UINT,
        (MCI_STATUS_ITEM | MCI_WAIT) as DWORD_PTR,
        core::ptr::addr_of_mut!(mciStatus) as DWORD_PTR,
    );
    if mciError != 0 as MCIERROR {
        return mciError;
    }

    SendMessage(
        hTrackBar,
        TBM_SETRANGEMIN as UINT,
        TRUE as WPARAM,
        1 as LPARAM,
    );
    SendMessage(
        hTrackBar,
        TBM_SETRANGEMAX as UINT,
        TRUE as WPARAM,
        mciStatus.dwReturn as LPARAM,
    );
    SendMessage(
        hTrackBar,
        TBM_SETPAGESIZE as UINT,
        0 as WPARAM,
        10 as LPARAM,
    );
    SendMessage(
        hTrackBar,
        TBM_SETLINESIZE as UINT,
        0 as WPARAM,
        1 as LPARAM,
    );
    SendMessage(
        hTrackBar,
        TBM_SETPOS as UINT,
        TRUE as WPARAM,
        1 as LPARAM,
    );

    if mciStatus.dwReturn < 10000 as DWORD_PTR {
        SendMessage(
            hTrackBar,
            TBM_SETTICFREQ as UINT,
            100 as WPARAM,
            0 as LPARAM,
        );
    } else if mciStatus.dwReturn < 100000 as DWORD_PTR {
        SendMessage(
            hTrackBar,
            TBM_SETTICFREQ as UINT,
            1000 as WPARAM,
            0 as LPARAM,
        );
    } else if mciStatus.dwReturn < 1000000 as DWORD_PTR {
        SendMessage(
            hTrackBar,
            TBM_SETTICFREQ as UINT,
            10000 as WPARAM,
            0 as LPARAM,
        );
    } else {
        SendMessage(
            hTrackBar,
            TBM_SETTICFREQ as UINT,
            100000 as WPARAM,
            0 as LPARAM,
        );
    }

    MaxFilePos = mciStatus.dwReturn as UINT;
    wDeviceId = mciOpen.wDeviceID as WORD;

    /* NOTE: Everything above this line may be done instead in OpenMediaFile() */

    if !lpFileName.is_null() {
        lpStr = _tcsrchr(lpFileName, ('\\' as u16)) as LPTSTR;
        if !lpStr.is_null() {
            lpStr = lpStr.add(1);
        } else {
            lpStr = lpFileName;
        }
    } else {
        lpStr = lpType;
    }

    StringCbCopy(
        core::ptr::addr_of_mut!(szCurrentFile) as STRSAFE_LPWSTR,
        core::mem::size_of::<[TCHAR; 260]>() as usize,
        lpStr as STRSAFE_LPCWSTR,
    );

    EnableMenuItems(hwnd);

    UpdateTimeDisplay(hTimeDisplay);
    UpdateWindowCaption(hwnd);

    return 0 as MCIERROR;
}

unsafe extern "C" fn GetDeviceMode(hwnd: HWND) -> DWORD {
    let mut mciError: MCIERROR = 0;
    let mut mciStatus: MCI_STATUS_PARMS = core::mem::zeroed();

    mciStatus.dwItem = MCI_STATUS_MODE as DWORD;
    mciError = mciSendCommand(
        wDeviceId as MCIDEVICEID,
        MCI_STATUS as UINT,
        (MCI_WAIT | MCI_STATUS_ITEM) as DWORD_PTR,
        core::ptr::addr_of_mut!(mciStatus) as DWORD_PTR,
    );
    if mciError != 0 as MCIERROR {
        ShowMCIError(hwnd, mciError);
        return MCI_MODE_NOT_READY as DWORD;
    }

    return mciStatus.dwReturn as DWORD;
}

unsafe extern "C" fn StopPlayback(mut hwnd: HWND) {
    let mut mciError: MCIERROR = 0;
    let mut mciGeneric: MCI_GENERIC_PARMS = core::mem::zeroed();
    let mut mciSeek: MCI_SEEK_PARMS = core::mem::zeroed();

    if wDeviceId == 0 as WORD {
        return;
    }

    KillTimer(hwnd, IDT_PLAYTIMER as UINT_PTR);
    SendMessage(
        hTrackBar,
        TBM_SETPOS as UINT,
        TRUE as WPARAM,
        1 as LPARAM,
    );

    mciGeneric.dwCallback = hwnd as DWORD_PTR;
    mciError = mciSendCommand(
        wDeviceId as MCIDEVICEID,
        MCI_STOP as UINT,
        MCI_WAIT as DWORD_PTR,
        core::ptr::addr_of_mut!(mciGeneric) as DWORD_PTR,
    );
    if mciError != 0 as MCIERROR {
        ShowMCIError(hwnd, mciError);
        return;
    }

    mciSendCommand(
        wDeviceId as MCIDEVICEID,
        MCI_SEEK as UINT,
        (MCI_WAIT | MCI_SEEK_TO_START) as DWORD_PTR,
        core::ptr::addr_of_mut!(mciSeek) as DWORD_PTR,
    );

    UpdateTimeDisplay(hTimeDisplay);
    UpdateWindowCaption(hwnd);

    SendMessage(
        hToolBar,
        TB_SETCMDID as UINT,
        0 as WPARAM,
        IDC_PLAY as LPARAM,
    );
    SendMessage(
        hToolBar,
        TB_CHANGEBITMAP as UINT,
        IDC_PLAY as WPARAM,
        (IDB_PLAYICON - IDB_PLAYICON) as LPARAM,
    );
}

unsafe extern "C" fn SeekPlayback(mut hwnd: HWND, mut dwNewPos: DWORD) {
    let mut mciError: MCIERROR = 0;
    let mut mciSeek: MCI_SEEK_PARMS = core::mem::zeroed();
    let mut mciPlay: MCI_PLAY_PARMS = core::mem::zeroed();
    if wDeviceId == 0 {
        return;
    }
    mciSeek.dwTo = dwNewPos as DWORD_PTR as DWORD;
    mciError = mciSendCommand(
        wDeviceId as MCIDEVICEID,
        MCI_SEEK as UINT,
        (MCI_WAIT | MCI_TO) as DWORD_PTR,
        core::ptr::addr_of_mut!(mciSeek) as DWORD_PTR,
    );
    if mciError != 0 {
        ShowMCIError(hwnd, mciError);
    }

    mciPlay.dwCallback = hwnd as DWORD_PTR;
    mciError = mciSendCommand(
        wDeviceId as MCIDEVICEID,
        MCI_PLAY as UINT,
        MCI_NOTIFY as DWORD_PTR,
        core::ptr::addr_of_mut!(mciPlay) as DWORD_PTR,
    );
    if mciError != 0 {
        ShowMCIError(hwnd, mciError);
    }
}

unsafe extern "C" fn SeekBackPlayback(hwnd: HWND) {
    let mut mciStatus: MCI_STATUS_PARMS = core::mem::zeroed();
    let mut dwNewPos: DWORD = 0;

    if wDeviceId as u32 == 0 as u32 {
        return;
    }

    mciStatus.dwItem = MCI_STATUS_POSITION as DWORD;
    mciSendCommand(
        wDeviceId as MCIDEVICEID,
        MCI_STATUS as UINT,
        MCI_STATUS_ITEM as DWORD_PTR,
        core::ptr::addr_of_mut!(mciStatus) as DWORD_PTR,
    );

    dwNewPos = (mciStatus.dwReturn.wrapping_sub((1 as DWORD) as _)) as u32;

    if (dwNewPos as UINT) <= 1 as UINT {
        StopPlayback(hwnd);
    } else {
        SeekPlayback(hwnd, dwNewPos);
    }
}

unsafe extern "C" fn SeekForwPlayback(hwnd: HWND) {
    let mut mciStatus: MCI_STATUS_PARMS = core::mem::zeroed();
    let mut dwNewPos: DWORD;

    if wDeviceId as i32 == 0 {
        return;
    }

    mciStatus.dwItem = MCI_STATUS_POSITION;
    mciSendCommand(
        wDeviceId as MCIDEVICEID,
        MCI_STATUS as UINT,
        MCI_STATUS_ITEM as DWORD_PTR,
        core::ptr::addr_of_mut!(mciStatus) as DWORD_PTR,
    );

    dwNewPos = (mciStatus.dwReturn.wrapping_add((1 as DWORD) as _)) as u32;

    if (dwNewPos as UINT) >= MaxFilePos {
        StopPlayback(hwnd);
    } else {
        SeekPlayback(hwnd, dwNewPos);
    }
}

#[no_mangle]
pub unsafe extern "system" fn PlayTimerProc(hwnd: HWND, uMsg: UINT, idEvent: UINT_PTR, dwTime: DWORD) {
    let mut mciStatus: MCI_STATUS_PARMS = core::mem::zeroed();
    let mut dwPos: DWORD;

    if wDeviceId == 0 {
        KillTimer(hwnd, (IDT_PLAYTIMER) as UINT_PTR);
    }

    mciStatus.dwItem = MCI_STATUS_POSITION;
    mciSendCommand(
        wDeviceId as MCIDEVICEID,
        MCI_STATUS,
        (MCI_STATUS_ITEM as DWORD_PTR),
        core::ptr::addr_of_mut!(mciStatus) as DWORD_PTR,
    );
    dwPos = (mciStatus.dwReturn) as u32;

    SendMessage(
        hTrackBar,
        TBM_SETPOS,
        TRUE as WPARAM,
        dwPos as LPARAM,
    );
    UpdateTimeDisplay(hTimeDisplay);
}

unsafe extern "C" fn StartPlayback(hwnd: HWND) {
    let mut mciError: MCIERROR = 0;
    let mut mciPlay: MCI_PLAY_PARMS = core::mem::zeroed();
    let mut mciSeek: MCI_SEEK_PARMS = core::mem::zeroed();

    SetTimer(hwnd, IDT_PLAYTIMER as UINT_PTR, 100 as UINT, Some(PlayTimerProc));

    mciSendCommand(wDeviceId as MCIDEVICEID, MCI_SEEK as UINT, (MCI_WAIT | MCI_SEEK_TO_START) as DWORD_PTR, core::ptr::addr_of_mut!(mciSeek) as DWORD_PTR);

    mciPlay.dwCallback = hwnd as DWORD_PTR;
    mciPlay.dwFrom = 0 as DWORD;
    mciPlay.dwTo = MaxFilePos as DWORD;

    mciError = mciSendCommand(wDeviceId as MCIDEVICEID, MCI_PLAY as UINT, (MCI_NOTIFY | MCI_FROM) as DWORD_PTR, core::ptr::addr_of_mut!(mciPlay) as DWORD_PTR);
    if mciError != 0 as MCIERROR {
        ShowMCIError(hwnd, mciError);
        return;
    }

    UpdateWindowCaption(hwnd);

    SendMessage(hToolBar,
                TB_SETCMDID as UINT,
                0 as WPARAM,
                IDC_PAUSE as LPARAM);
    SendMessage(hToolBar,
                TB_CHANGEBITMAP as UINT,
                IDC_PAUSE as WPARAM,
                (IDB_PAUSEICON - IDB_PLAYICON) as LPARAM);
}

unsafe extern "C" fn TogglePlaybackState(hwnd: HWND) {
    let mut mciError: MCIERROR = 0;
    let mut mciGeneric: MCI_GENERIC_PARMS = core::mem::zeroed();
    let mut idBmp: ULONG = IDB_PLAYICON as ULONG;
    let mut idCmd: ULONG = IDC_PLAY as ULONG;

    if wDeviceId == 0 as WORD {
        return;
    }

    match GetDeviceMode(hwnd) {
        MCI_MODE_OPEN | MCI_MODE_STOP => {
            StartPlayback(hwnd);
            return;
        }

        MCI_MODE_PLAY => {
            mciGeneric.dwCallback = hwnd as DWORD_PTR;
            mciError = mciSendCommand(
                wDeviceId as MCIDEVICEID,
                MCI_PAUSE as UINT,
                MCI_WAIT as DWORD_PTR,
                core::ptr::addr_of_mut!(mciGeneric) as DWORD_PTR,
            );
            idBmp = IDB_PLAYICON as ULONG;
            idCmd = IDC_PLAY as ULONG;
        }

        MCI_MODE_PAUSE => {
            mciGeneric.dwCallback = hwnd as DWORD_PTR;
            mciError = mciSendCommand(
                wDeviceId as MCIDEVICEID,
                MCI_RESUME as UINT,
                MCI_WAIT as DWORD_PTR,
                core::ptr::addr_of_mut!(mciGeneric) as DWORD_PTR,
            );
            idBmp = IDB_PAUSEICON as ULONG;
            idCmd = IDC_PAUSE as ULONG;
        }

        _ => {
            return;
        }
    }

    if mciError != 0 as MCIERROR {
        ShowMCIError(hwnd, mciError);
        return;
    }

    UpdateWindowCaption(hwnd);

    SendMessage(
        hToolBar,
        TB_SETCMDID as UINT,
        0 as WPARAM,
        idCmd as LPARAM,
    );
    SendMessage(
        hToolBar,
        TB_CHANGEBITMAP as UINT,
        idCmd as WPARAM,
        idBmp.wrapping_sub(IDB_PLAYICON as ULONG) as LPARAM,
    );
}

unsafe extern "C" fn ShowDeviceProperties(hwnd: HWND) {
    let mut mciError: MCIERROR = 0;
    let mut mciGeneric: MCI_GENERIC_PARMS = core::mem::zeroed::<MCI_GENERIC_PARMS>();
    mciError = mciSendCommand(
        wDeviceId as MCIDEVICEID,
        MCI_CONFIGURE as UINT,
        MCI_WAIT as DWORD_PTR,
        core::ptr::addr_of_mut!(mciGeneric) as DWORD_PTR,
    );
    if mciError != 0 as MCIERROR {
        ShowMCIError(hwnd, mciError);
    }
}

unsafe extern "C" fn CloseMediaFile(hwnd: HWND) {
    StopPlayback(hwnd);

    if bIsSingleWindow != 0 {
        SwitchViewMode(hwnd);
    }

    CloseMciDevice();
    UpdateWindowCaption(hwnd);
}

unsafe extern "C" fn OpenMediaFile(mut hwnd: HWND, mut lpFileName: LPTSTR, mut lpType: LPTSTR) {
    let mut mciError: MCIERROR = 0;
    if GetFileAttributes(lpFileName as *const u16) == INVALID_FILE_ATTRIBUTES {
        return;
    }
    if wDeviceId != 0 {
        CloseMediaFile(hwnd);
    }
    mciError = OpenMciDevice(hwnd, lpType, lpFileName);
    if mciError != 0 {
        ShowMCIError(hwnd, mciError);
        return;
    }
    StartPlayback(hwnd);
}

unsafe extern "C" fn InsertDeviceMenuItem(mut hMenu: HMENU, mut uItem: UINT, mut fByPosition: BOOL, mut uItemID: UINT, mut dwDeviceIndex: DWORD) -> DWORD {
    let mut lpmii: MENUITEMINFO = core::mem::zeroed();
    let mut mciError: MCIERROR = 0;
    let mut szDeviceName: [TCHAR; (MAX_MCISTR) as usize] = [0; (MAX_MCISTR) as usize];
    let mut szFriendlyName: [TCHAR; (MAX_MCISTR) as usize] = [0; (MAX_MCISTR) as usize];
    mciError = GetDeviceName(
        dwDeviceIndex,
        &raw mut szDeviceName as LPTSTR,
        core::mem::size_of_val(&szDeviceName) as DWORD,
    ) as MCIERROR;
    if mciError != 0 {
        return mciError as DWORD;
    }
    mciError = GetDeviceFriendlyName(
        &raw mut szDeviceName as LPTSTR,
        &raw mut szFriendlyName as LPTSTR,
        core::mem::size_of_val(&szFriendlyName) as DWORD,
    ) as MCIERROR;
    if mciError != 0 {
        return mciError as DWORD;
    }
    if DeviceUsesFiles(&raw mut szDeviceName as LPTSTR) != 0 {
        StringCbCat(
            &raw mut szFriendlyName as LPTSTR,
            core::mem::size_of_val(&szFriendlyName) as size_t,
            ([b'.' as u16, b'.' as u16, b'.' as u16, 0u16].as_ptr()),
        );
    }
    ZeroMemory!(
        &raw mut lpmii as *mut core::ffi::c_void,
        core::mem::size_of::<MENUITEMINFO>() as size_t,
    );
    lpmii.cbSize = core::mem::size_of::<MENUITEMINFO>() as UINT;
    lpmii.fMask = (MIIM_DATA | MIIM_TYPE | MIIM_ID) as UINT;
    lpmii.wID = uItemID;
    lpmii.fType = MF_STRING as UINT;
    lpmii.dwTypeData = (&raw mut szFriendlyName as *mut TCHAR as LPWSTR).cast();
    lpmii.dwItemData = (dwDeviceIndex as usize) as _;
    InsertMenuItem(
        hMenu,
        uItem,
        fByPosition,
        &raw mut lpmii as LPCMENUITEMINFOW,
    );
    return 0;
}

unsafe extern "C" fn BuildFileFilterAndDeviceMenu() {
    let mut szDeviceName: [TCHAR; (MAX_MCISTR) as usize] = [0 as TCHAR; (MAX_MCISTR) as usize];
    let mut szFriendlyName: [TCHAR; (MAX_MCISTR) as usize] = [0 as TCHAR; (MAX_MCISTR) as usize];
    let mut szDevice: *mut TCHAR = core::ptr::null_mut();
    static mut szDefaultExtension: [TCHAR; 4] = ['*' as TCHAR, '.' as TCHAR, '*' as TCHAR, 0];
    let mut szExtensionList: *mut TCHAR = core::ptr::null_mut();
    let mut szExtension: *mut TCHAR = core::ptr::null_mut();
    let mut c: *mut TCHAR = core::ptr::null_mut();
    let mut d: *mut TCHAR = core::ptr::null_mut();
    let mut dwNumValues: DWORD = 0;
    let mut dwNumDevices: DWORD = 0;
    let mut dwValueNameLen: DWORD = 0;
    let mut dwValueDataSize: DWORD = 0;
    let mut dwMaskLen: DWORD = 0;
    let mut dwFilterSize: DWORD = 0;
    let mut dwDeviceSize: DWORD = 0;
    let mut dwExtensionLen: DWORD = 0;
    let mut dwPosition: DWORD = 0;
    let mut i: DWORD = 0;
    let mut j: DWORD = 0;
    let mut uSizeRemain: size_t = 0;
    let mut uMaskRemain: size_t = 0;
    let mut hKey: HKEY = core::ptr::null_mut();

    /* Always load the default (all files) filter */
    LoadString(hInstance, IDS_ALL_TYPES_FILTER, core::ptr::addr_of_mut!(szDefaultFilter) as LPTSTR, (ARRAYSIZE!(szDefaultFilter)) as i32);

    'cleanup: {
        'failure: {
            if (RegOpenKeyEx(HKEY_LOCAL_MACHINE, (&[0x0053, 0x004f, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0057, 0x0069, 0x006e, 0x0064, 0x006f, 0x0077, 0x0073, 0x0020, 0x004e, 0x0054, 0x005c, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006e, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006f, 0x006e, 0x005c, 0x004d, 0x0043, 0x0049, 0x0020, 0x0045, 0x0078, 0x0074, 0x0065, 0x006e, 0x0073, 0x0069, 0x006f, 0x006e, 0x0073, 0x0000u16]).as_ptr() /* _T "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\MCI Extensi" */, 0, KEY_READ, core::ptr::addr_of_mut!(hKey))) as u32 != ERROR_SUCCESS {
                break 'failure;
            }

            if (RegQueryInfoKey(hKey, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(dwNumValues), core::ptr::addr_of_mut!(dwValueNameLen), core::ptr::addr_of_mut!(dwValueDataSize), core::ptr::null_mut(), core::ptr::null_mut())) as u32 != ERROR_SUCCESS {
                break 'failure;
            }

            dwMaskLen = (dwValueNameLen.wrapping_add(3).wrapping_mul(dwNumValues)).wrapping_add(1);

            szExtensionList = malloc((dwMaskLen as size_t).wrapping_mul(core::mem::size_of::<TCHAR>() as size_t)) as *mut TCHAR;
            if szExtensionList.is_null() {
                break 'failure;
            }

            dwNumDevices = GetNumDevices();

            /* Allocate space for every pair of Device and Extension Filter */
            dwFilterSize = ((MAX_MCISTR + (dwMaskLen * 2) as i32 + 5) * (dwNumDevices) as i32) as u32;

            /* Add space for the "All supported" entry */
            dwFilterSize = (((dwFilterSize + dwMaskLen * 2 + 7) as size_t).wrapping_mul(core::mem::size_of::<TCHAR>() as size_t)).wrapping_add(core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>() as size_t) as DWORD;

            szFilter = malloc(dwFilterSize as size_t) as LPTSTR;
            if szFilter.is_null() {
                break 'failure;
            }

            szExtension = malloc(((dwValueNameLen + 1) as size_t).wrapping_mul(core::mem::size_of::<TCHAR>() as size_t)) as *mut TCHAR;
            if szExtension.is_null() {
                break 'failure;
            }

            szDevice = malloc((dwValueDataSize as size_t).wrapping_add(core::mem::size_of::<TCHAR>() as size_t)) as *mut TCHAR;
            if szDevice.is_null() {
                break 'failure;
            }

            ZeroMemory!(szFilter, dwFilterSize);

            uSizeRemain = dwFilterSize as size_t;
            c = szFilter;

            j = 1;
            while j <= dwNumDevices {
                'jbody: {
                    if GetDeviceName(j, core::ptr::addr_of_mut!(szDeviceName) as LPTSTR, core::mem::size_of::<[TCHAR; (MAX_MCISTR) as usize]>() as DWORD) != 0 {
                        break 'jbody;
                    }

                    if GetDeviceFriendlyName(core::ptr::addr_of_mut!(szDeviceName) as LPTSTR, core::ptr::addr_of_mut!(szFriendlyName) as LPTSTR, core::mem::size_of::<[TCHAR; (MAX_MCISTR) as usize]>() as DWORD) != 0 {
                        break 'jbody;
                    }

                    /* Insert a menu item under the "Device" menu for every found MCI device */
                    InsertDeviceMenuItem(GetSubMenu(hMainMenu, 3), dwPosition, (TRUE) as BOOL, IDM_DEVICE_FIRST + dwPosition, j);
                    dwPosition = dwPosition.wrapping_add(1);

                    /* Copy the default extension list, that may be overwritten after... */
                    StringCbCopy(szExtensionList, (dwMaskLen as size_t).wrapping_mul(core::mem::size_of::<TCHAR>() as size_t), core::ptr::addr_of_mut!(szDefaultExtension) as *mut TCHAR);

                    /* Try to determine the real extension list */
                    uMaskRemain = (dwMaskLen as size_t).wrapping_mul(core::mem::size_of::<TCHAR>() as size_t);
                    d = szExtensionList;

                    i = 0;
                    while i < dwNumValues {
                        dwExtensionLen = dwValueNameLen.wrapping_add(1);
                        dwDeviceSize = dwValueDataSize.wrapping_add(core::mem::size_of::<TCHAR>() as DWORD);

                        ZeroMemory!(szDevice, dwDeviceSize);

                        if (RegEnumValue(hKey, i, szExtension, core::ptr::addr_of_mut!(dwExtensionLen), core::ptr::null_mut(), core::ptr::null_mut(), szDevice as LPBYTE, core::ptr::addr_of_mut!(dwDeviceSize))) as u32 == ERROR_SUCCESS {
                            CharLowerBuff(szDevice, ((dwDeviceSize as size_t) / core::mem::size_of::<TCHAR>() as size_t) as DWORD);
                            CharLowerBuff(core::ptr::addr_of_mut!(szDeviceName) as LPTSTR, ARRAYSIZE!(szDeviceName));
                            if _tcscmp(core::ptr::addr_of_mut!(szDeviceName) as LPTSTR, szDevice) == 0 {
                                CharLowerBuff(szExtension, dwExtensionLen);
                                StringCbPrintfEx!(d, uMaskRemain, core::ptr::addr_of_mut!(d), core::ptr::addr_of_mut!(uMaskRemain), 0, (&[0x0025, 0x0073, 0x0025, 0x0073, 0x0025, 0x0073, 0x0000u16]).as_ptr() /* _T "%s%s%s" */, (&[0x002a, 0x002e, 0x0000u16]).as_ptr() /* _T "*." */, szExtension, (&[0x003b, 0x0000u16]).as_ptr() /* _T ";" */);
                            }
                        }
                        i = i.wrapping_add(1);
                    }

                    /* Remove the last separator */
                    // Two divergences from the C here and below. It backed up unconditionally,
                    // so a device with no registered extension wrote szExtensionList[-1] (d had
                    // not moved; the default list is still in the buffer). And it credited the
                    // freed WCHAR to uSizeRemain -- the FILTER buffer's remaining space -- where
                    // d is in szExtensionList, whose counter is uMaskRemain: every device
                    // inflated the filter's room by one WCHAR for the bounded writes below.
                    if d != szExtensionList {
                        d = d.offset(-1);
                        uMaskRemain = uMaskRemain.wrapping_add(core::mem::size_of::<TCHAR>() as size_t);
                        *d = '\0' as TCHAR;
                    }

                    /* Add the description */
                    StringCbPrintfEx!(c, uSizeRemain, core::ptr::addr_of_mut!(c), core::ptr::addr_of_mut!(uSizeRemain), 0, (&[0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0000u16]).as_ptr() /* _T "%s (%s)" */, core::ptr::addr_of_mut!(szFriendlyName) as LPTSTR, szExtensionList);

                    /* Skip one char to separate the description from the filter mask */
                    c = c.offset(1);
                    uSizeRemain = uSizeRemain.wrapping_sub(core::mem::size_of::<TCHAR>() as size_t);

                    /* Append the filter mask */
                    StringCbCopyEx(c, uSizeRemain, szExtensionList, core::ptr::addr_of_mut!(c), core::ptr::addr_of_mut!(uSizeRemain), 0);

                    /* Skip another char to separate the elements of the filter mask */
                    c = c.offset(1);
                    uSizeRemain = uSizeRemain.wrapping_sub(core::mem::size_of::<TCHAR>() as size_t);
                }
                j = j.wrapping_add(1);
            }

            /* Build the full list of supported extensions */
            uMaskRemain = (dwMaskLen as size_t).wrapping_mul(core::mem::size_of::<TCHAR>() as size_t);
            d = szExtensionList;

            i = 0;
            while i < dwNumValues {
                dwExtensionLen = dwValueNameLen.wrapping_add(1);

                if (RegEnumValue(hKey, i, szExtension, core::ptr::addr_of_mut!(dwExtensionLen), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut())) as u32 == ERROR_SUCCESS {
                    CharLowerBuff(szExtension, dwExtensionLen);
                    StringCbPrintfEx!(d, uMaskRemain, core::ptr::addr_of_mut!(d), core::ptr::addr_of_mut!(uMaskRemain), 0, (&[0x0025, 0x0073, 0x0025, 0x0073, 0x0025, 0x0073, 0x0000u16]).as_ptr() /* _T "%s%s%s" */, (&[0x002a, 0x002e, 0x0000u16]).as_ptr() /* _T "*." */, szExtension, (&[0x003b, 0x0000u16]).as_ptr() /* _T ";" */);
                }
                i = i.wrapping_add(1);
            }

            /* Remove the last separator */
            // Same two fixes as the per-device list above.
            if d != szExtensionList {
                d = d.offset(-1);
                uMaskRemain = uMaskRemain.wrapping_add(core::mem::size_of::<TCHAR>() as size_t);
                *d = '\0' as TCHAR;
            }

            /* Add the default (all files) description */
            StringCbPrintfEx!(c, uSizeRemain, core::ptr::addr_of_mut!(c), core::ptr::addr_of_mut!(uSizeRemain), 0, (&[0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0000u16]).as_ptr() /* _T "%s (%s)" */, core::ptr::addr_of_mut!(szDefaultFilter) as LPTSTR, szExtensionList);

            /* Skip one char to separate the description from the filter mask */
            c = c.offset(1);
            uSizeRemain = uSizeRemain.wrapping_sub(core::mem::size_of::<TCHAR>() as size_t);

            /* Append the filter mask */
            StringCbCopyEx(c, uSizeRemain, szExtensionList, core::ptr::addr_of_mut!(c), core::ptr::addr_of_mut!(uSizeRemain), 0);

            break 'cleanup;
        }

        /* Failure: We failed at retrieving the supported files, so use the default filter */
        if !szFilter.is_null() {
            free(szFilter as *mut core::ffi::c_void);
        }
        szFilter = core::ptr::addr_of_mut!(szDefaultFilter) as *mut TCHAR as LPTSTR;

        uSizeRemain = core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>() as size_t;
        c = szFilter;

        /* Add the default (all files) description */
        StringCbPrintfEx!(c, uSizeRemain, core::ptr::addr_of_mut!(c), core::ptr::addr_of_mut!(uSizeRemain), 0, (&[0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x0073, 0x0029, 0x0000u16]).as_ptr() /* _T "%s (%s)" */, core::ptr::addr_of_mut!(szDefaultFilter) as LPTSTR, core::ptr::addr_of_mut!(szDefaultExtension) as *mut TCHAR);

        /* Skip one char to separate the description from the filter mask */
        c = c.offset(1);
        uSizeRemain = uSizeRemain.wrapping_sub(core::mem::size_of::<TCHAR>() as size_t);

        /* Append the filter mask */
        StringCbCopyEx(c, uSizeRemain, core::ptr::addr_of_mut!(szDefaultExtension) as *mut TCHAR, core::ptr::addr_of_mut!(c), core::ptr::addr_of_mut!(uSizeRemain), 0);
    }

    /* Cleanup: */
    if !szExtensionList.is_null() {
        free(szExtensionList as *mut core::ffi::c_void);
    }
    if !szExtension.is_null() {
        free(szExtension as *mut core::ffi::c_void);
    }
    if !szDevice.is_null() {
        free(szDevice as *mut core::ffi::c_void);
    }
    RegCloseKey(hKey);

    return;
}

unsafe extern "C" fn CleanupFileFilter() {
    if !szFilter.is_null() && szFilter != core::ptr::addr_of_mut!(szDefaultFilter) as *mut TCHAR {
        free(szFilter as *mut ::core::ffi::c_void);
    }
}

unsafe extern "C" fn OpenFileDialog(mut hwnd: HWND, mut dwFilterIndex: DWORD, mut lpType: LPTSTR) {
    let mut OpenFileName: OPENFILENAME = core::mem::zeroed();
    let mut szFile: [TCHAR; (MAX_PATH + 1) as _] = [0; (MAX_PATH + 1) as _] /* _T "" */;

    ZeroMemory!(
        core::ptr::addr_of_mut!(OpenFileName) as *mut core::ffi::c_void,
        core::mem::size_of::<OPENFILENAME>() as usize,
    );

    OpenFileName.lStructSize = core::mem::size_of::<OPENFILENAME>() as DWORD;
    OpenFileName.hwndOwner = hwnd;
    OpenFileName.hInstance = hInstance;
    OpenFileName.lpstrFilter = szFilter;
    OpenFileName.lpstrFile = core::ptr::addr_of_mut!(szFile) as LPTSTR;
    OpenFileName.nMaxFile = ARRAYSIZE!(szFile) as DWORD;
    OpenFileName.Flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_HIDEREADONLY | OFN_SHAREAWARE;
    OpenFileName.lpstrDefExt = (&[0x0000, 0x0000u16]).as_ptr() /* _T "\0" */;
    OpenFileName.nFilterIndex = dwFilterIndex;

    if GetOpenFileName(core::ptr::addr_of_mut!(OpenFileName)) == 0 {
        return;
    }

    OpenMediaFile(hwnd, OpenFileName.lpstrFile, lpType);
}

unsafe extern "C" fn HandleDeviceMenuItem(mut hwnd: HWND, mut uItem: UINT) {
    let mut lpmii: MENUITEMINFO = core::mem::zeroed();
    let mut szDeviceName: [TCHAR; (MAX_MCISTR) as usize] = [0; (MAX_MCISTR) as usize];
    let mut mciError: MCIERROR = 0;

    ZeroMemory!(
        &raw mut lpmii as *mut ::core::ffi::c_void,
        core::mem::size_of::<MENUITEMINFO>() as usize,
    );
    lpmii.cbSize = core::mem::size_of::<MENUITEMINFO>() as UINT;
    lpmii.fMask = MIIM_DATA as UINT;
    GetMenuItemInfo(hMainMenu, uItem, (FALSE as UINT) as i32, (&raw mut lpmii).cast());

    mciError = GetDeviceName(
        lpmii.dwItemData as DWORD,
        &raw mut szDeviceName as LPTSTR,
        core::mem::size_of::<[TCHAR; (MAX_MCISTR) as usize]>() as DWORD,
    ) as MCIERROR;
    if mciError != 0 {
        ShowMCIError(hwnd, mciError);
        return;
    }

    if DeviceUsesFiles(&raw mut szDeviceName as LPTSTR) != 0 {
        OpenFileDialog(
            hwnd,
            uItem
                .wrapping_sub(IDM_DEVICE_FIRST as UINT)
                .wrapping_add(1 as UINT) as DWORD,
            &raw mut szDeviceName as LPTSTR,
        );
        return;
    }

    mciError = OpenMciDevice(
        hwnd,
        &raw mut szDeviceName as LPTSTR,
        core::ptr::null_mut::<WCHAR>(),
    );
    if mciError != 0 {
        ShowMCIError(hwnd, mciError);
    }

    return;
}

#[no_mangle]
pub unsafe extern "system" fn MainWndProc(hwnd: HWND, Message: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    'sw: {
        match Message {
            WM_CREATE => {
                InitControls(hwnd);
                hMainMenu = GetMenu(hwnd);
                break 'sw;
            }

            WM_DROPFILES => {
                let mut drophandle: HDROP = core::ptr::null_mut();
                let mut droppedfile: [TCHAR; (MAX_PATH) as _] = core::mem::zeroed();

                drophandle = wParam as HDROP;
                DragQueryFile(drophandle, 0, droppedfile.as_mut_ptr(), ARRAYSIZE!(droppedfile));
                DragFinish(drophandle);
                OpenMediaFile(hwnd, droppedfile.as_mut_ptr(), core::ptr::null_mut());
                break 'sw;
            }

            MM_MCINOTIFY => {
                if wParam == (MCI_NOTIFY_SUCCESSFUL as WPARAM) {
                    StopPlayback(hwnd);
                    if bRepeat != 0 {
                        StartPlayback(hwnd);
                    }
                }
                break 'sw;
            }

            WM_NOTIFY => {
                'sn: {
                    let mut pnmhdr: LPNMHDR = lParam as LPNMHDR;

                    match (*pnmhdr).code {
                        TTN_GETDISPINFO => {
                            'sid: {
                                let mut lpttt: LPTOOLTIPTEXT = lParam as LPTOOLTIPTEXT;
                                let mut idButton: UINT = (*lpttt).hdr.idFrom as UINT;

                                match idButton {
                                    IDC_PLAY => {
                                        (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_PLAY);
                                        break 'sid;
                                    }
                                    IDC_STOP => {
                                        (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_STOP);
                                        break 'sid;
                                    }
                                    IDC_EJECT => {
                                        (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_EJECT);
                                        break 'sid;
                                    }
                                    IDC_BACKWARD => {
                                        (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_BACKWARD);
                                        break 'sid;
                                    }
                                    IDC_SEEKBACK => {
                                        (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_SEEKBACK);
                                        break 'sid;
                                    }
                                    IDC_SEEKFORW => {
                                        (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_SEEKFORW);
                                        break 'sid;
                                    }
                                    IDC_FORWARD => {
                                        (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_FORWARD);
                                        break 'sid;
                                    }
                                    IDC_PAUSE => {
                                        (*lpttt).lpszText = MAKEINTRESOURCE!(IDS_TOOLTIP_PAUSE);
                                        break 'sid;
                                    }
                                    _ => {}
                                }
                                break 'sn;
                            }
                        }
                        _ => {}
                    }
                }
                break 'sw;
            }

            WM_SIZING => {
                let mut pRect: LPRECT = lParam as LPRECT;

                if bIsSingleWindow == 0 {
                    if (*pRect).right - (*pRect).left < MAIN_WINDOW_MIN_WIDTH {
                        (*pRect).right = (*pRect).left + MAIN_WINDOW_MIN_WIDTH;
                    }

                    if (*pRect).bottom - (*pRect).top != MAIN_WINDOW_HEIGHT {
                        (*pRect).bottom = (*pRect).top + MAIN_WINDOW_HEIGHT;
                    }
                }
                return (TRUE) as LRESULT;
            }

            WM_SIZE => {
                let mut Rect: RECT = core::mem::zeroed();

                if !hToolBar.is_null() && !hTrackBar.is_null() {
                    SendMessage(hToolBar, TB_AUTOSIZE, 0, 0);
                    SendMessage(hToolBar, TB_GETITEMRECT, 1, core::ptr::addr_of_mut!(Rect) as LPARAM);
                    MoveWindow(hTimeDisplay, LOWORD!(lParam) as i32 - 140, 4, 135, 18, (TRUE) as BOOL);

                    if bIsSingleWindow == 0 {
                        let mut Size: UINT = (GetSystemMetrics((SM_CYMENU) as i32) as LONG + Rect.bottom) as UINT;
                        MoveWindow(hTrackBar, 0, 0, LOWORD!(lParam) as i32, (HIWORD!(lParam) as UINT).wrapping_sub(Size) as i32, (TRUE) as BOOL);
                    } else {
                        let mut ToolbarRect: RECT = core::mem::zeroed();
                        let mut mciPut: MCI_DGV_PUT_PARMS = core::mem::zeroed();

                        MoveWindow(hTrackBar, 180, 0, LOWORD!(lParam) as i32 - 322, 25, (TRUE) as BOOL);

                        GetClientRect(hwnd, core::ptr::addr_of_mut!(Rect));
                        GetClientRect(hToolBar, core::ptr::addr_of_mut!(ToolbarRect));

                        mciPut.rc.top = 0;
                        mciPut.rc.left = 0;
                        mciPut.rc.right = Rect.right;
                        mciPut.rc.bottom = Rect.bottom - (ToolbarRect.bottom - ToolbarRect.top) - 2;

                        mciSendCommand(wDeviceId as UINT, MCI_PUT, (MCI_DGV_PUT_DESTINATION | MCI_DGV_RECT | MCI_WAIT) as _, core::ptr::addr_of_mut!(mciPut) as DWORD_PTR);
                    }
                }
                return 0;
            }

            WM_HSCROLL => {
                if hTrackBar == lParam as HWND {
                    if wDeviceId != 0 {
                        let mut dwNewPos: DWORD = SendMessage(hTrackBar, TBM_GETPOS, 0, 0) as DWORD;
                        SeekPlayback(hwnd, dwNewPos);
                    } else {
                        SendMessage(hTrackBar, TBM_SETPOS, (TRUE) as _, 0);
                    }
                }
                break 'sw;
            }

            WM_NCLBUTTONDBLCLK => {
                if wParam == (HTCAPTION) as _ {
                    SwitchViewMode(hwnd);
                }
                break 'sw;
            }

            WM_COMMAND => {
                if LOWORD!(wParam) >= (IDM_DEVICE_FIRST as u16) {
                    HandleDeviceMenuItem(hwnd, (LOWORD!(wParam)) as UINT);
                    break 'sw;
                }

                'scmd: {
                    match (LOWORD!(wParam)) as u32 {
                        IDC_PLAY | IDC_PAUSE => {
                            if wDeviceId != 0 {
                                TogglePlaybackState(hwnd);
                            } else {
                                OpenFileDialog(hwnd, 1, core::ptr::null_mut());
                            }
                            break 'scmd;
                        }

                        IDC_STOP => {
                            StopPlayback(hwnd);
                            break 'scmd;
                        }

                        IDC_EJECT => {
                            break 'scmd;
                        }

                        IDC_BACKWARD => {
                            break 'scmd;
                        }

                        IDC_SEEKBACK => {
                            SeekBackPlayback(hwnd);
                            break 'scmd;
                        }

                        IDC_SEEKFORW => {
                            SeekForwPlayback(hwnd);
                            break 'scmd;
                        }

                        IDC_FORWARD => {
                            break 'scmd;
                        }

                        IDM_OPEN_FILE => {
                            OpenFileDialog(hwnd, 1, core::ptr::null_mut());
                            return 0;
                        }

                        IDM_CLOSE_FILE => {
                            CloseMediaFile(hwnd);
                            break 'scmd;
                        }

                        IDM_REPEAT => {
                            if bRepeat == 0 {
                                CheckMenuItem(hMainMenu, IDM_REPEAT, MF_BYCOMMAND | MF_CHECKED);
                                bRepeat = (TRUE) as i32;
                            } else {
                                CheckMenuItem(hMainMenu, IDM_REPEAT, MF_BYCOMMAND | MF_UNCHECKED);
                                bRepeat = (FALSE) as i32;
                            }
                            break 'scmd;
                        }

                        IDM_SWITCHVIEW => {
                            SwitchViewMode(hwnd);
                            break 'scmd;
                        }

                        IDM_DEVPROPS => {
                            ShowDeviceProperties(hwnd);
                            break 'scmd;
                        }

                        IDM_VOLUMECTL => {
                            ShellExecute(hwnd, core::ptr::null_mut(), (&[0x0053, 0x004e, 0x0044, 0x0056, 0x004f, 0x004c, 0x0033, 0x0032, 0x002e, 0x0045, 0x0058, 0x0045, 0x0000u16]).as_ptr() /* _T "SNDVOL32.EXE" */, core::ptr::null_mut(), core::ptr::null_mut(), (SW_SHOWNORMAL) as i32);
                            break 'scmd;
                        }

                        IDM_ABOUT => {
                            ShellAbout(hwnd, core::ptr::addr_of_mut!(szAppTitle) as LPTSTR, core::ptr::null_mut(),
                                LoadIcon(hInstance, MAKEINTRESOURCE!(IDI_MAIN)));
                            break 'scmd;
                        }

                        IDM_EXIT => {
                            PostMessage(hwnd, WM_CLOSE, 0, 0);
                            return 0;
                        }

                        _ => {}
                    }
                }
                break 'sw;
            }

            WM_DESTROY => {
                CloseMediaFile(hwnd);
                PostQuitMessage(0);
                return 0;
            }

            _ => {}
        }
    }

    return DefWindowProc(hwnd, Message, wParam, lParam);
}

#[export_name = "wWinMain"]
pub unsafe extern "system" fn _tWinMain(hInst: HINSTANCE, hPrevInst: HINSTANCE, lpCmdLine: LPTSTR, nCmdShow: INT) -> INT {
    let mut WndClass: WNDCLASSEX = core::mem::zeroed();
    let mut szClassName: [TCHAR; 11] = ['R' as TCHAR, 'O' as TCHAR, 'S' as TCHAR, 'M' as TCHAR, 'P' as TCHAR, 'L' as TCHAR, 'A' as TCHAR, 'Y' as TCHAR, '3' as TCHAR, '2' as TCHAR, 0 as TCHAR];
    let mut hwnd: HWND = core::ptr::null_mut();
    let mut msg: MSG = core::mem::zeroed();
    let mut dwError: DWORD = 0;
    let mut hAccel: HANDLE = core::ptr::null_mut();

    hInstance = hInst;

    match GetUserDefaultUILanguage() {
        v if v == MAKELANGID!(LANG_HEBREW, SUBLANG_DEFAULT) => {
            SetProcessDefaultLayout(LAYOUT_RTL);
        }
        _ => {}
    }

    LoadString(hInstance, IDS_APPTITLE, core::ptr::addr_of_mut!(szAppTitle) as LPTSTR, (ARRAYSIZE!(szAppTitle)) as i32);

    WndClass.cbSize = core::mem::size_of::<WNDCLASSEX>() as UINT;
    WndClass.lpszClassName = (core::ptr::addr_of_mut!(szClassName) as *mut TCHAR as LPCWSTR).cast();
    WndClass.lpfnWndProc = Some(MainWndProc);
    WndClass.hInstance = hInstance;
    WndClass.style = (CS_HREDRAW | CS_VREDRAW) as UINT;
    WndClass.hIcon = LoadIcon(hInstance, MAKEINTRESOURCE!(IDI_MAIN));
    WndClass.hCursor = LoadCursor(core::ptr::null_mut(), IDC_ARROW);
    WndClass.hbrBackground = (COLOR_BTNFACE + 1) as HBRUSH;
    WndClass.lpszMenuName = (MAKEINTRESOURCE!(IDR_MAINMENU) as LPCWSTR).cast();

    if RegisterClassEx(core::ptr::addr_of_mut!(WndClass) as *const WNDCLASSEXW) == 0 {
        ShowLastWin32Error(core::ptr::null_mut());
        return 0;
    }

    hwnd = CreateWindow!(
        core::ptr::addr_of_mut!(szClassName) as *mut TCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(szAppTitle) as *mut TCHAR as LPCWSTR,
        (WS_SYSMENU | WS_MINIMIZEBOX | WS_THICKFRAME | WS_OVERLAPPED | WS_CAPTION | WS_CLIPCHILDREN) as DWORD,
        CW_USEDEFAULT as INT,
        CW_USEDEFAULT as INT,
        350 as INT,
        MAIN_WINDOW_HEIGHT as INT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        hInstance,
        core::ptr::null_mut(),
    );
    if hwnd.is_null() {
        ShowLastWin32Error(core::ptr::null_mut());
        return 0;
    }

    hAccel = LoadAccelerators(hInstance, MAKEINTRESOURCE!(ID_ACCELERATORS)) as HANDLE;

    BuildFileFilterAndDeviceMenu();

    DragAcceptFiles(hwnd, (TRUE) as BOOL);

    DisableMenuItems();

    let mut szSndVolExe: [TCHAR; 13] = ['S' as TCHAR, 'N' as TCHAR, 'D' as TCHAR, 'V' as TCHAR, 'O' as TCHAR, 'L' as TCHAR, '3' as TCHAR, '2' as TCHAR, '.' as TCHAR, 'E' as TCHAR, 'X' as TCHAR, 'E' as TCHAR, 0 as TCHAR];
    dwError = SearchPath(core::ptr::null_mut(), core::ptr::addr_of_mut!(szSndVolExe) as LPTSTR, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut());
    if dwError == 0 {
        EnableMenuItem(hMainMenu, IDM_VOLUMECTL, (MF_BYCOMMAND | MF_GRAYED) as UINT);
    }

    /* Show it */
    ShowWindow(hwnd, (SW_SHOW) as i32);
    UpdateWindow(hwnd);

    if *lpCmdLine == '"' as TCHAR {
        OpenMediaFile(hwnd, *(*__p___wargv()).add(1 as usize), core::ptr::null_mut());
    } else {
        OpenMediaFile(hwnd, lpCmdLine, core::ptr::null_mut());
    }

    /* Message Loop */
    while GetMessage(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0) != 0 {
        if TranslateAccelerator(hwnd, (hAccel).cast(), core::ptr::addr_of_mut!(msg)) == 0 {
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessage(core::ptr::addr_of_mut!(msg));
        }
    }

    CleanupFileFilter();

    DestroyAcceleratorTable((hAccel).cast());

    return msg.wParam as INT;
}
