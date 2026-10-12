// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_A9EDBA2FC474: [u16; 3] = [0x0025u16, 0x0064, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut hDllInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut Bauds: [DWORD; 16] = [
    CBR_110 as DWORD,
    CBR_300 as DWORD,
    CBR_600 as DWORD,
    CBR_1200 as DWORD,
    CBR_2400 as DWORD,
    CBR_4800 as DWORD,
    CBR_9600 as DWORD,
    CBR_14400 as DWORD,
    CBR_19200 as DWORD,
    CBR_38400 as DWORD,
    CBR_56000 as DWORD,
    CBR_57600 as DWORD,
    CBR_115200 as DWORD,
    CBR_128000 as DWORD,
    CBR_256000 as DWORD,
    0 as ::core::ffi::c_int as DWORD,
];

#[no_mangle]
pub static mut ByteSizes: [BYTE; 5] = [
    5 as ::core::ffi::c_int as BYTE,
    6 as ::core::ffi::c_int as BYTE,
    7 as ::core::ffi::c_int as BYTE,
    8 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
];

#[no_mangle]
pub static mut Parities: [PARITY_INFO; 6] = [
    _PARITY_INFO {
        Parity: EVENPARITY as BYTE,
        StrId: IDS_EVENPARITY as UINT,
    },
    _PARITY_INFO {
        Parity: MARKPARITY as BYTE,
        StrId: IDS_MARKPARITY as UINT,
    },
    _PARITY_INFO {
        Parity: NOPARITY as BYTE,
        StrId: IDS_NOPARITY as UINT,
    },
    _PARITY_INFO {
        Parity: ODDPARITY as BYTE,
        StrId: IDS_ODDPARITY as UINT,
    },
    _PARITY_INFO {
        Parity: SPACEPARITY as BYTE,
        StrId: IDS_SPACEPARITY as UINT,
    },
    _PARITY_INFO {
        Parity: 0 as BYTE,
        StrId: 0 as UINT,
    },
];

#[no_mangle]
pub static mut StopBits: [STOPBIT_INFO; 4] = [
    _STOPBIT_INFO {
        StopBit: ONESTOPBIT as BYTE,
        StrId: IDS_ONESTOPBIT as UINT,
    },
    _STOPBIT_INFO {
        StopBit: ONE5STOPBITS as BYTE,
        StrId: IDS_ONE5STOPBITS as UINT,
    },
    _STOPBIT_INFO {
        StopBit: TWOSTOPBITS as BYTE,
        StrId: IDS_TWOSTOPBITS as UINT,
    },
    _STOPBIT_INFO {
        StopBit: 0 as BYTE,
        StrId: 0 as UINT,
    },
];

// Resource string IDs (from serialui resource.h; values not provided in this pass)

#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstance: HINSTANCE, dwReason: DWORD, reserved: LPVOID) -> BOOL {
    if dwReason == DLL_PROCESS_ATTACH {
        hDllInstance = hInstance;
    } else if dwReason == DLL_THREAD_ATTACH {
        DisableThreadLibraryCalls(hInstance);
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn drvCommConfigDialogW(
    lpszDevice: LPCWSTR,
    hWnd: HWND,
    lpCommConfig: LPCOMMCONFIG,
) -> DWORD {
    let mut DialogInfo: DIALOG_INFO = core::mem::zeroed();

    if lpszDevice.is_null() || lpCommConfig.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    DialogInfo.lpszDevice = lpszDevice;
    DialogInfo.lpCC = lpCommConfig;

    return DialogBoxParamW(
        hDllInstance,
        MAKEINTRESOURCEW!(IDD_COMMDLG),
        hWnd,
        Some(CommDlgProc),
        core::ptr::addr_of_mut!(DialogInfo) as LPARAM,
    ) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn drvCommConfigDialogA(
    lpszDevice: LPCSTR,
    hWnd: HWND,
    lpCommConfig: LPCOMMCONFIG,
) -> DWORD {
    let mut result: BOOL;
    let mut len: UINT;
    let mut wstr: *mut WCHAR;

    len = MultiByteToWideChar(CP_ACP, 0, lpszDevice, -1, core::ptr::null_mut(), 0) as UINT;
    wstr = HeapAlloc(GetProcessHeap(), 0, (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as *mut WCHAR;
    if !wstr.is_null() {
        MultiByteToWideChar(CP_ACP, 0, lpszDevice, -1, wstr, len as core::ffi::c_int);
        result = drvCommConfigDialogW(wstr as LPCWSTR, hWnd, lpCommConfig) as BOOL;
        HeapFree(GetProcessHeap(), 0, wstr as PVOID);
        return result as DWORD;
    } else {
        return ERROR_NOT_ENOUGH_MEMORY as DWORD;
    }
}

#[no_mangle]
pub unsafe extern "system" fn drvSetDefaultCommConfigW(
	lpszDevice: LPCWSTR,
	lpCommConfig: LPCOMMCONFIG,
	dwSize: DWORD,
) -> DWORD {
	UNIMPLEMENTED!();
	0
}

#[no_mangle]
pub unsafe extern "system" fn drvSetDefaultCommConfigA(mut lpszDevice: LPCSTR, mut lpCommConfig: LPCOMMCONFIG, mut dwSize: DWORD) -> DWORD {
    // The C body is a bare UNIMPLEMENTED with no return, an indeterminate value; 0 as in
    // drvSetDefaultCommConfigW.
    UNIMPLEMENTED!();
    0
}

#[no_mangle]
pub unsafe extern "system" fn drvGetDefaultCommConfigW(
    lpszDevice: LPCWSTR,
    lpCommConfig: LPCOMMCONFIG,
    lpdwSize: LPDWORD,
) -> DWORD {
    // A bare UNIMPLEMENTED with no return in the C: 0, as its siblings.
    UNIMPLEMENTED!();
    0
}

#[no_mangle]
pub unsafe extern "system" fn drvGetDefaultCommConfigA(
    mut lpszDevice: LPCSTR,
    mut lpCommConfig: LPCOMMCONFIG,
    mut lpdwSize: LPDWORD,
) -> DWORD {
    // A bare UNIMPLEMENTED with no return in the C: 0, as its siblings.
    UNIMPLEMENTED!();
    0
}

#[no_mangle]
pub unsafe extern "system" fn CommDlgProc(hDlg: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut lpDlgInfo: LPDIALOG_INFO = core::ptr::null_mut();
    let mut hBox: HWND = core::ptr::null_mut();
    if Msg == WM_INITDIALOG as UINT {
        let mut wstr: [WCHAR; 255] = [0 as WCHAR; 255];
        let mut rc: RECT = core::mem::zeroed();
        let mut rcDlg: RECT = core::mem::zeroed();
        let mut rcOwner: RECT = core::mem::zeroed();
        let mut hOwner: HWND = core::ptr::null_mut();
        let mut i: INT = 0;

        lpDlgInfo = lParam as LPDIALOG_INFO;
        SetWindowLongPtrW(hDlg, DWLP_USER as INT, lpDlgInfo as LONG_PTR);

        /* Set title */
        if LoadStringW(
            hDllInstance,
            IDS_TITLE as UINT,
            wstr.as_mut_ptr(),
            (core::mem::size_of_val(&wstr) / core::mem::size_of_val(&wstr[0 as usize])) as INT,
        ) != 0
        {
            SetWindowTextW(hDlg, wstr.as_mut_ptr() as LPCWSTR);
        }

        /* FIXME - this won't work correctly systems with multiple monitors! */
        hOwner = GetParent(hDlg);
        if hOwner.is_null() {
            hOwner = GetDesktopWindow();
        }

        /* Position dialog in the center of owner window */
        GetWindowRect(hOwner, core::ptr::addr_of_mut!(rcOwner));
        GetWindowRect(hDlg, core::ptr::addr_of_mut!(rcDlg));
        CopyRect(core::ptr::addr_of_mut!(rc), core::ptr::addr_of_mut!(rcOwner) as LPCRECT);
        OffsetRect(
            core::ptr::addr_of_mut!(rcDlg),
            -rcDlg.left as INT,
            -rcDlg.top as INT,
        );
        OffsetRect(
            core::ptr::addr_of_mut!(rc),
            -rc.left as INT,
            -rc.top as INT,
        );
        OffsetRect(
            core::ptr::addr_of_mut!(rc),
            -rcDlg.right as INT,
            -rcDlg.bottom as INT,
        );
        SetWindowPos(
            hDlg,
            HWND_TOP as HWND,
            (rcOwner.left + (rc.right / (2 as LONG))) as INT,
            (rcOwner.top + (rc.bottom / (2 as LONG))) as INT,
            0 as INT,
            0 as INT,
            SWP_NOSIZE as UINT,
        );

        /* Initialize baud rate combo */
        hBox = GetDlgItem(hDlg, IDC_BAUDRATE as INT);
        if hBox.is_null() {
            EndDialog(hDlg, ERROR_CANCELLED as INT_PTR);
        }

        i = 0 as INT;
        while Bauds[i as usize] != 0 {
            wsprintf(wstr.as_mut_ptr(), RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(), Bauds[i as usize]);
            SendMessageW(hBox, CB_INSERTSTRING as UINT, i as WPARAM, wstr.as_mut_ptr() as LPARAM);
            if Bauds[i as usize] == (*(*lpDlgInfo).lpCC).dcb.BaudRate {
                SendMessageW(hBox, CB_SETCURSEL as UINT, i as WPARAM, 0 as LPARAM);
            }
            i += 1;
        }

        if SendMessageW(hBox, CB_GETCURSEL as UINT, 0 as WPARAM, 0 as LPARAM)
            == CB_ERR as LRESULT
        {
            SendMessageW(hBox, CB_SETCURSEL as UINT, DEFAULT_BAUD_INDEX as WPARAM, 0 as LPARAM);
        }

        /* Initialize byte size combo */
        hBox = GetDlgItem(hDlg, IDC_BYTESIZE as INT);
        if hBox.is_null() {
            EndDialog(hDlg, ERROR_CANCELLED as INT_PTR);
        }

        i = 0 as INT;
        while ByteSizes[i as usize] != 0 {
            wsprintf(wstr.as_mut_ptr(), RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(), Bauds[i as usize]);
            SendMessageW(hBox, CB_INSERTSTRING as UINT, i as WPARAM, wstr.as_mut_ptr() as LPARAM);
            if ByteSizes[i as usize] as INT == (*(*lpDlgInfo).lpCC).dcb.ByteSize as INT {
                SendMessageW(hBox, CB_SETCURSEL as UINT, i as WPARAM, 0 as LPARAM);
            }
            i += 1;
        }

        if SendMessageW(hBox, CB_GETCURSEL as UINT, 0 as WPARAM, 0 as LPARAM)
            == CB_ERR as LRESULT
        {
            SendMessageW(hBox, CB_SETCURSEL as UINT, DEFAULT_BYTESIZE_INDEX as WPARAM, 0 as LPARAM);
        }

        /* Initialize parity combo */
        hBox = GetDlgItem(hDlg, IDC_PARITY as INT);
        if hBox.is_null() {
            EndDialog(hDlg, ERROR_CANCELLED as INT_PTR);
        }

        i = 0 as INT;
        while Parities[i as usize].StrId != 0 {
            if LoadStringW(
                hDllInstance,
                Parities[i as usize].StrId,
                wstr.as_mut_ptr(),
                (core::mem::size_of_val(&wstr) / core::mem::size_of_val(&wstr[0 as usize])) as INT,
            ) != 0
            {
                SendMessageW(hBox, CB_INSERTSTRING as UINT, i as WPARAM, wstr.as_mut_ptr() as LPARAM);
                if Parities[i as usize].Parity as INT == (*(*lpDlgInfo).lpCC).dcb.Parity as INT {
                    SendMessageW(hBox, CB_SETCURSEL as UINT, i as WPARAM, 0 as LPARAM);
                }
            }
            i += 1;
        }

        if SendMessageW(hBox, CB_GETCURSEL as UINT, 0 as WPARAM, 0 as LPARAM)
            == CB_ERR as LRESULT
        {
            SendMessageW(hBox, CB_SETCURSEL as UINT, DEFAULT_PARITY_INDEX as WPARAM, 0 as LPARAM);
        }

        /* Initialize stop bits combo */
        hBox = GetDlgItem(hDlg, IDC_STOPBITS as INT);
        if hBox.is_null() {
            EndDialog(hDlg, ERROR_CANCELLED as INT_PTR);
        }

        i = 0 as INT;
        while StopBits[i as usize].StrId != 0 {
            if LoadStringW(
                hDllInstance,
                StopBits[i as usize].StrId,
                wstr.as_mut_ptr(),
                (core::mem::size_of_val(&wstr) / core::mem::size_of_val(&wstr[0 as usize])) as INT,
            ) != 0
            {
                SendMessageW(hBox, CB_INSERTSTRING as UINT, i as WPARAM, wstr.as_mut_ptr() as LPARAM);
                if StopBits[i as usize].StopBit as INT == (*(*lpDlgInfo).lpCC).dcb.StopBits as INT {
                    SendMessageW(hBox, CB_SETCURSEL as UINT, i as WPARAM, 0 as LPARAM);
                }
            }
            i += 1;
        }

        if SendMessageW(hBox, CB_GETCURSEL as UINT, 0 as WPARAM, 0 as LPARAM)
            == CB_ERR as LRESULT
        {
            SendMessageW(hBox, CB_SETCURSEL as UINT, DEFAULT_STOPBITS_INDEX as WPARAM, 0 as LPARAM);
        }

        /* Initialize flow control combo */
        hBox = GetDlgItem(hDlg, IDC_FLOW as INT);
        if hBox.is_null() {
            EndDialog(hDlg, ERROR_CANCELLED as INT_PTR);
        }

        if LoadStringW(
            hDllInstance,
            IDS_FC_NO as UINT,
            wstr.as_mut_ptr(),
            (core::mem::size_of_val(&wstr) / core::mem::size_of_val(&wstr[0 as usize])) as INT,
        ) != 0
        {
            SendMessageW(hBox, CB_INSERTSTRING as UINT, 0 as WPARAM, wstr.as_mut_ptr() as LPARAM);
            SendMessageW(hBox, CB_SETCURSEL as UINT, 0 as WPARAM, 0 as LPARAM);
            (*lpDlgInfo).InitialFlowIndex = 0 as UINT;
        }

        if LoadStringW(
            hDllInstance,
            IDS_FC_CTSRTS as UINT,
            wstr.as_mut_ptr(),
            (core::mem::size_of_val(&wstr) / core::mem::size_of_val(&wstr[0 as usize])) as INT,
        ) != 0
        {
            SendMessageW(hBox, CB_INSERTSTRING as UINT, 1 as WPARAM, wstr.as_mut_ptr() as LPARAM);
            if (*(*lpDlgInfo).lpCC).dcb.fRtsControl() as INT == RTS_CONTROL_HANDSHAKE as INT
                || (*(*lpDlgInfo).lpCC).dcb.fOutxCtsFlow() as INT != FALSE as INT
            {
                SendMessageW(hBox, CB_SETCURSEL as UINT, 1 as WPARAM, 0 as LPARAM);
                (*lpDlgInfo).InitialFlowIndex = 1 as UINT;
            }
        }

        if LoadStringW(
            hDllInstance,
            IDS_FC_XONXOFF as UINT,
            wstr.as_mut_ptr(),
            (core::mem::size_of_val(&wstr) / core::mem::size_of_val(&wstr[0 as usize])) as INT,
        ) != 0
        {
            SendMessageW(hBox, CB_INSERTSTRING as UINT, 2 as WPARAM, wstr.as_mut_ptr() as LPARAM);
            if (*(*lpDlgInfo).lpCC).dcb.fOutX() as INT != 0 || (*(*lpDlgInfo).lpCC).dcb.fInX() as INT != 0 {
                SendMessageW(hBox, CB_SETCURSEL as UINT, 2 as WPARAM, 0 as LPARAM);
                (*lpDlgInfo).InitialFlowIndex = 2 as UINT;
            }
        }

        /* Set focus */
        SetFocus(GetDlgItem(hDlg, IDC_OKBTN as INT));

        return FALSE as INT_PTR;
    } /* WM_INITDIALOG */
    else if Msg == WM_COMMAND as UINT {
        /* WM_COMMAND */
        if wParam == IDC_CANCELBTN as WPARAM {
            EndDialog(hDlg, ERROR_CANCELLED as INT_PTR);
        } else if wParam == IDC_OKBTN as WPARAM {
            OkButton(hDlg);
            EndDialog(hDlg, ERROR_SUCCESS as INT_PTR);
        }
        return TRUE as INT_PTR;
    } /* WM_COMMAND */
    else if Msg == WM_CLOSE as UINT {
        /* WM_CLOSE */
        EndDialog(hDlg, ERROR_CANCELLED as INT_PTR);
        return TRUE as INT_PTR;
    } /* WM_CLOSE */
    else {
        return FALSE as INT_PTR;
    }
}
#[no_mangle]
pub unsafe extern "C" fn OkButton(mut hDlg: HWND) {
    let mut lpDlgInfo: LPDIALOG_INFO = core::ptr::null_mut();
    let mut Index: UINT = 0;

    lpDlgInfo = GetWindowLongPtrW(hDlg, DWLP_USER as core::ffi::c_int) as LPDIALOG_INFO;

    /* Baud rate */
    Index = SendMessageW(
        GetDlgItem(hDlg, (IDC_BAUDRATE as i32)),
        CB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as UINT;
    (*(*lpDlgInfo).lpCC).dcb.BaudRate = Bauds[Index as usize];

    /* Byte size */
    Index = SendMessageW(
        GetDlgItem(hDlg, (IDC_BYTESIZE as i32)),
        CB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as UINT;
    (*(*lpDlgInfo).lpCC).dcb.ByteSize = ByteSizes[Index as usize];

    /* Parity */
    Index = SendMessageW(
        GetDlgItem(hDlg, (IDC_PARITY as i32)),
        CB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as UINT;
    (*(*lpDlgInfo).lpCC).dcb.Parity = Parities[Index as usize].Parity;

    /* Stop bits */
    Index = SendMessageW(
        GetDlgItem(hDlg, (IDC_STOPBITS as i32)),
        CB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as UINT;
    (*(*lpDlgInfo).lpCC).dcb.StopBits = StopBits[Index as usize].StopBit;

    /* Flow Control */
    Index = SendMessageW(
        GetDlgItem(hDlg, (IDC_FLOW as i32)),
        CB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as UINT;
    if (*lpDlgInfo).InitialFlowIndex != Index {
        match Index {
            0 => {
                /* NO */
                (*(*lpDlgInfo).lpCC).dcb.set_fDtrControl(DTR_CONTROL_DISABLE as DWORD);
                (*(*lpDlgInfo).lpCC).dcb.set_fRtsControl(RTS_CONTROL_DISABLE as DWORD);
                (*(*lpDlgInfo).lpCC).dcb.set_fOutxCtsFlow(FALSE as DWORD);
                (*(*lpDlgInfo).lpCC).dcb.set_fOutxDsrFlow(FALSE as DWORD);
                (*(*lpDlgInfo).lpCC).dcb.set_fOutX(FALSE as DWORD);
                (*(*lpDlgInfo).lpCC).dcb.set_fInX(FALSE as DWORD);
            }
            1 => {
                /* CTS/RTS */
                (*(*lpDlgInfo).lpCC).dcb.set_fDtrControl(DTR_CONTROL_DISABLE as DWORD);
                (*(*lpDlgInfo).lpCC).dcb.set_fRtsControl(RTS_CONTROL_HANDSHAKE as DWORD);
                (*(*lpDlgInfo).lpCC).dcb.set_fOutxCtsFlow(TRUE as DWORD);
                (*(*lpDlgInfo).lpCC).dcb.set_fOutxDsrFlow(FALSE as DWORD);
                (*(*lpDlgInfo).lpCC).dcb.set_fOutX(FALSE as DWORD);
                (*(*lpDlgInfo).lpCC).dcb.set_fInX(FALSE as DWORD);
            }
            2 => {
                /* XON/XOFF */
                (*(*lpDlgInfo).lpCC).dcb.set_fDtrControl(DTR_CONTROL_DISABLE as DWORD);
                (*(*lpDlgInfo).lpCC).dcb.set_fRtsControl(RTS_CONTROL_DISABLE as DWORD);
                (*(*lpDlgInfo).lpCC).dcb.set_fOutxCtsFlow(FALSE as DWORD);
                (*(*lpDlgInfo).lpCC).dcb.set_fOutxDsrFlow(FALSE as DWORD);
                (*(*lpDlgInfo).lpCC).dcb.set_fOutX(TRUE as DWORD);
                (*(*lpDlgInfo).lpCC).dcb.set_fInX(TRUE as DWORD);
            }
            _ => {}
        }
    }
}
