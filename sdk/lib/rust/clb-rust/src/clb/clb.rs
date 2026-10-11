use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_1CE990B58AC0: [u16; 22] = [
    0x0043u16, 0x004C, 0x0042, 0x0053, 0x005F, 0x0053, 0x0050, 0x0052, 0x0049, 0x004E,
    0x0047, 0x004C, 0x0059, 0x005F, 0x0043, 0x004F, 0x004C, 0x0055, 0x004D, 0x004E,
    0x0053, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_43FF9061CE33: [u16; 21] = [
    0x0043u16, 0x004C, 0x0042, 0x0053, 0x005F, 0x0044, 0x0049, 0x0053, 0x0041, 0x0042,
    0x004C, 0x0045, 0x004E, 0x004F, 0x0053, 0x0043, 0x0052, 0x004F, 0x004C, 0x004C,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_673B06C15387: [u16; 13] = [
    0x0043u16, 0x004C, 0x0042, 0x0053, 0x005F, 0x0056, 0x0053, 0x0043, 0x0052, 0x004F,
    0x004C, 0x004C, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_9F9BE88033FF: [u16; 10] = [0x0043u16, 0x004C, 0x0042, 0x0053, 0x005F, 0x0053, 0x004F, 0x0052, 0x0054, 0x0000];
pub static RUSTCONV_C2R_WSTR_A108F32AC68D: [u16; 21] = [
    0x0043u16, 0x004C, 0x0042, 0x0053, 0x005F, 0x0050, 0x004F, 0x0050, 0x004F, 0x0055,
    0x0054, 0x005F, 0x0048, 0x0045, 0x0041, 0x0044, 0x0049, 0x004E, 0x0047, 0x0053,
    0x0000,
];
pub static RUSTCONV_C2R_WSTR_A8D7C6A150CA: [u16; 12] = [
    0x0043u16, 0x004C, 0x0042, 0x0053, 0x005F, 0x0042, 0x004F, 0x0052, 0x0044, 0x0045,
    0x0052, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_CBF0A208D6AA: [u16; 12] = [
    0x0043u16, 0x004C, 0x0042, 0x0053, 0x005F, 0x004E, 0x004F, 0x0054, 0x0049, 0x0046,
    0x0059, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_EDF4AF44844D: [u16; 19] = [
    0x004Cu16, 0x0042, 0x0053, 0x005F, 0x004F, 0x0057, 0x004E, 0x0045, 0x0052, 0x0044,
    0x0052, 0x0041, 0x0057, 0x0046, 0x0049, 0x0058, 0x0045, 0x0044, 0x0000,
];

static mut hDllInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

static mut ClbClassName: [WCHAR; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [WCHAR; 14]>(*b"C\0o\0l\0u\0m\0n\0L\0i\0s\0t\0B\0o\0x\0\0\0")
};

static mut ClbColumns: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [WCHAR; 24]>(
        *b"C\0o\0l\0u\0m\0n\x001\0;\0C\0o\0l\0u\0m\0n\x002\0;\0C\0o\0l\0u\0m\0n\x003\0\0\0",
    )
};

static mut ClbsSupportedStyles: [CLBS_INFO; 8] = unsafe {
    [
        _CLBS_INFO {
            Style: CLBS_NOTIFY as DWORD,
            Unknown: 0 as DWORD,
            StyleName: RUSTCONV_C2R_WSTR_CBF0A208D6AA.as_ptr() as LPCWSTR,
        },
        _CLBS_INFO {
            Style: CLBS_SORT as DWORD,
            Unknown: 0 as DWORD,
            StyleName: RUSTCONV_C2R_WSTR_9F9BE88033FF.as_ptr() as LPCWSTR,
        },
        _CLBS_INFO {
            Style: CLBS_DISABLENOSCROLL as DWORD,
            Unknown: 0 as DWORD,
            StyleName: RUSTCONV_C2R_WSTR_43FF9061CE33.as_ptr() as LPCWSTR,
        },
        _CLBS_INFO {
            Style: CLBS_VSCROLL as DWORD,
            Unknown: 0 as DWORD,
            StyleName: RUSTCONV_C2R_WSTR_673B06C15387.as_ptr() as LPCWSTR,
        },
        _CLBS_INFO {
            Style: CLBS_BORDER as DWORD,
            Unknown: 0 as DWORD,
            StyleName: RUSTCONV_C2R_WSTR_A8D7C6A150CA.as_ptr() as LPCWSTR,
        },
        _CLBS_INFO {
            Style: CLBS_POPOUT_HEADINGS as DWORD,
            Unknown: 0 as DWORD,
            StyleName: RUSTCONV_C2R_WSTR_A108F32AC68D.as_ptr() as LPCWSTR,
        },
        _CLBS_INFO {
            Style: CLBS_SPRINGLY_COLUMNS as DWORD,
            Unknown: 0 as DWORD,
            StyleName: RUSTCONV_C2R_WSTR_1CE990B58AC0.as_ptr() as LPCWSTR,
        },
        _CLBS_INFO {
            Style: LBS_OWNERDRAWFIXED as DWORD,
            Unknown: 0 as DWORD,
            StyleName: RUSTCONV_C2R_WSTR_EDF4AF44844D.as_ptr() as LPCWSTR,
        },
    ]
};

#[repr(C)]
pub struct CLB_PRIVATEDATA {
    pub hwnd: HWND,
}
pub type PCLB_PRIVATEDATA = *mut CLB_PRIVATEDATA;

#[no_mangle]
pub unsafe extern "system" fn ClbAddData(
    Unknown1: DWORD,
    Unknown2: DWORD,
    Unknown3: DWORD,
) -> DWORD {
    DPRINT1!("ClbAddData(0x%x, 0x%x, 0x%x)\n", Unknown1, Unknown2, Unknown3);
    0
}

#[no_mangle]
pub unsafe extern "system" fn ClbSetColumnWidths(
    Unknown1: DWORD,
    Unknown2: DWORD,
    Unknown3: DWORD,
) -> DWORD {
    DPRINT1!("ClbSetColumnWidths(0x%x, 0x%x, 0x%x)\n", Unknown1, Unknown2, Unknown3);
    0
}

#[no_mangle]
pub unsafe extern "system" fn ClbWndProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut PrivData: PCLB_PRIVATEDATA = core::ptr::null_mut();
    let mut Ret: LRESULT = 0;

    DPRINT1!(
        b"ClbWndProc(0x%p, 0x%x, 0x%p, 0x%p)\n\0".as_ptr() as *const i8,
        hwnd,
        uMsg,
        wParam,
        lParam,
    );

    PrivData = GetWindowLongPtr(hwnd, 0) as PCLB_PRIVATEDATA;

    let mut at: u8 = 0;
    loop {
        match at {
            0 => {
                if PrivData.is_null() && uMsg != WM_CREATE {
                    at = 1;
                    continue;
                }
                if uMsg == WM_CREATE {
                    PrivData = HeapAlloc(
                        GetProcessHeap(),
                        0,
                        core::mem::size_of::<CLB_PRIVATEDATA>() as SIZE_T,
                    ) as PCLB_PRIVATEDATA;
                    if PrivData.is_null() {
                        Ret = (-(1)) as LRESULT;
                        break;
                    }
                    (*PrivData).hwnd = hwnd;
                    // Exceeding the C: clb.c never stores PrivData, so every later message
                    // read NULL (straight to DefWindowProc) and WM_DESTROY freed nothing --
                    // one leaked CLB_PRIVATEDATA per window. cbWndExtra already reserves it.
                    SetWindowLongPtr(hwnd, 0, PrivData as LONG_PTR);
                    break;
                } else if uMsg == WM_DESTROY {
                    HeapFree(
                        GetProcessHeap(),
                        0,
                        PrivData as LPVOID,
                    );
                    break;
                } else {
                    at = 1;
                    continue;
                }
            }
            1 => {
                // HandleDefMsg:
                Ret = DefWindowProc(hwnd, uMsg, wParam, lParam);
                break;
            }
            _ => break,
        }
    }

    Ret
}

unsafe extern "system" fn ClbpStyleDlgProc(
    mut hwndDlg: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    let mut Ret: INT_PTR = (FALSE) as _;

    DPRINT1!(
        b"ClbpStyleDlgProc(0x%p, 0x%x, 0x%p, 0x%p)\n\0".as_ptr() as *const i8,
        hwndDlg,
        uMsg,
        wParam,
        lParam,
    );

    match uMsg {
        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDOK | IDCANCEL => {
                    EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
                }
                _ => {}
            }
        }
        WM_CLOSE => {
            EndDialog(hwndDlg, (IDCANCEL) as INT_PTR);
        }
        WM_INITDIALOG => {
            Ret = (TRUE) as _;
        }
        _ => {}
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn ClbStyleW(hWndParent: HWND, dwInitParam: LPARAM) -> INT_PTR {
    return DialogBoxParam(hDllInstance, MAKEINTRESOURCE!(IDD_COLUMNLISTBOXSTYLES), hWndParent, Some(ClbpStyleDlgProc), dwInitParam);
}

#[no_mangle]
pub unsafe extern "system" fn CustomControlInfoW(CustomControlInfo: LPCUSTOM_CONTROL_INFO) -> BOOL {
    if !CustomControlInfo.is_null() {
        wcscpy(
            &raw mut (*CustomControlInfo).ClassName as *mut WCHAR,
            core::ptr::addr_of!(ClbClassName) as *const WCHAR,
        );

        (*CustomControlInfo).Zero1 = 0;

        wcscpy(
            &raw mut (*CustomControlInfo).ClassName2 as *mut WCHAR,
            core::ptr::addr_of!(ClbClassName) as *const WCHAR,
        );

        (*CustomControlInfo).Unknown1 = 0x60; // FIXME - ???
        (*CustomControlInfo).Unknown2 = 0x50; // FIXME - ???
        (*CustomControlInfo).Unknown3 = 0x50A10013; // FIXME - ???

        (*CustomControlInfo).Zero2 = 0;
        (*CustomControlInfo).Zero3 = 0;

        (*CustomControlInfo).StylesCount = _countof!(ClbsSupportedStyles);
        (*CustomControlInfo).SupportedStyles =
            core::ptr::addr_of!(ClbsSupportedStyles) as *const CLBS_INFO;

        wcscpy(
            &raw mut (*CustomControlInfo).Columns as *mut WCHAR,
            core::ptr::addr_of!(ClbColumns) as *const WCHAR,
        );

        (*CustomControlInfo).ClbStyleW = Some(ClbStyleW);

        (*CustomControlInfo).Zero4 = 0;
        (*CustomControlInfo).Zero5 = 0;
        (*CustomControlInfo).Zero6 = 0;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    dwReason: DWORD,
    lpvReserved: LPVOID,
) -> BOOL {
    let mut Ret: BOOL = (TRUE) as i32;

    if dwReason == DLL_PROCESS_ATTACH {
        let mut ClbWndClass: WNDCLASS = WNDCLASS {
            style: 0,
            lpfnWndProc: None,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: core::ptr::null_mut(),
            hIcon: core::ptr::null_mut(),
            hCursor: core::ptr::null_mut(),
            hbrBackground: core::ptr::null_mut(),
            lpszMenuName: core::ptr::null(),
            lpszClassName: core::ptr::null(),
        };

        hDllInstance = hinstDLL;

        InitCommonControls();

        /* register the control's window class */
        ClbWndClass.style = (CS_GLOBALCLASS | CS_OWNDC) as UINT;
        ClbWndClass.lpfnWndProc = Some(ClbWndProc);
        ClbWndClass.cbClsExtra = 0 as core::ffi::c_int;
        ClbWndClass.cbWndExtra =
            core::mem::size_of::<PCLB_PRIVATEDATA>() as core::ffi::c_int;
        ClbWndClass.hInstance = hinstDLL;
        ClbWndClass.hIcon = core::ptr::null_mut();
        ClbWndClass.hCursor = LoadCursor(
            core::ptr::null_mut(),
            (IDC_ARROW as usize) as LPWSTR,
        );
        ClbWndClass.hbrBackground = core::ptr::null_mut();
        ClbWndClass.lpszMenuName = core::ptr::null();
        ClbWndClass.lpszClassName = core::ptr::addr_of!(ClbClassName) as *const WCHAR as LPCWSTR;

        if RegisterClass(core::ptr::addr_of_mut!(ClbWndClass)) == 0 {
            Ret = (FALSE) as i32;
        }
    } else if dwReason == DLL_THREAD_ATTACH || dwReason == DLL_THREAD_DETACH {
        /* nothing to do */
    } else if dwReason == DLL_PROCESS_DETACH {
        UnregisterClass(
            core::ptr::addr_of!(ClbClassName) as *const WCHAR as LPCWSTR,
            hinstDLL,
        );
    }
    return Ret;
}
