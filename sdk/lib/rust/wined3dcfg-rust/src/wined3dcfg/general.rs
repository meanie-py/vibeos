use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const VALUE_GDI: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<
        [u8; 48],
        [WCHAR; 24],
    >(
        *b"G\0D\0I\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    )
};

pub const VALUE_ENABLED: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<
        [u8; 48],
        [WCHAR; 24],
    >(
        *b"E\0n\0a\0b\0l\0e\0d\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    )
};

pub const VALUE_DISABLED: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<
        [u8; 48],
        [WCHAR; 24],
    >(
        *b"D\0i\0s\0a\0b\0l\0e\0d\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    )
};

pub const VALUE_BACKBUFFER: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [WCHAR; 24]>(
        *b"B\0a\0c\0k\0b\0u\0f\0f\0e\0r\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    )
};

pub const VALUE_FBO: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<
        [u8; 48],
        [WCHAR; 24],
    >(
        *b"F\0B\0O\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    )
};

pub const VALUE_DEFAULT: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<
        [u8; 48],
        [WCHAR; 24],
    >(
        *b"D\0e\0f\0a\0u\0l\0t\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
    )
};

pub const KEY_WINE: [::core::ffi::c_ushort; 23] = unsafe {
    ::core::mem::transmute::<[u8; 46], [::core::ffi::c_ushort; 23]>(
        *b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0W\0i\0n\0e\0\\\0D\0i\0r\0e\0c\0t\x003\0D\0\0\0",
    )
};

#[no_mangle]
pub static mut gwd3dsShaderLvl: [WINED3D_SETTINGS; 4] = unsafe {
    [
        _WINED3D_SETTINGS {
            szValue: VALUE_DEFAULT,
            iType: (REG_NONE as i32),
            iValue: 0 as INT,
        },
        _WINED3D_SETTINGS {
            szValue: ::core::mem::transmute::<
                [u8; 48],
                [WCHAR; 24],
            >(
                *b"1\0.\x004\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            iType: (REG_DWORD as i32),
            iValue: 1 as INT,
        },
        _WINED3D_SETTINGS {
            szValue: ::core::mem::transmute::<
                [u8; 48],
                [WCHAR; 24],
            >(
                *b"2\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            iType: (REG_DWORD as i32),
            iValue: 2 as INT,
        },
        _WINED3D_SETTINGS {
            szValue: ::core::mem::transmute::<
                [u8; 48],
                [WCHAR; 24],
            >(
                *b"3\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            iType: (REG_DWORD as i32),
            iValue: 3 as INT,
        },
    ]
};

#[no_mangle]
pub static mut gwd3dsDisable: [WINED3D_SETTINGS; 2] = [
    _WINED3D_SETTINGS {
        szValue: VALUE_DEFAULT,
        iType: (REG_NONE as i32),
        iValue: 0 as INT,
    },
    _WINED3D_SETTINGS {
        szValue: VALUE_DISABLED,
        iType: (REG_SZ as i32),
        iValue: 0 as INT,
    },
];

#[no_mangle]
pub static mut gwd3dsEnable: [WINED3D_SETTINGS; 2] = [
    _WINED3D_SETTINGS {
        szValue: VALUE_DEFAULT,
        iType: (REG_NONE as i32),
        iValue: 0 as INT,
    },
    _WINED3D_SETTINGS {
        szValue: VALUE_ENABLED,
        iType: (REG_SZ as i32),
        iValue: 0 as INT,
    },
];

#[no_mangle]
pub static mut gwd3dsOffscreen: [WINED3D_SETTINGS; 3] = [
    _WINED3D_SETTINGS {
        szValue: VALUE_DEFAULT,
        iType: (REG_NONE as i32),
        iValue: 0 as INT,
    },
    _WINED3D_SETTINGS {
        szValue: VALUE_BACKBUFFER,
        iType: (REG_SZ as i32),
        iValue: 0 as INT,
    },
    _WINED3D_SETTINGS {
        szValue: VALUE_FBO,
        iType: (REG_SZ as i32),
        iValue: 0 as INT,
    },
];

#[no_mangle]
pub static mut gwd3dsVidMem: [WINED3D_SETTINGS; 8] = unsafe {
    [
        _WINED3D_SETTINGS {
            szValue: VALUE_DEFAULT,
            iType: (REG_NONE as i32),
            iValue: 0 as INT,
        },
        _WINED3D_SETTINGS {
            szValue: ::core::mem::transmute::<
                [u8; 48],
                [WCHAR; 24],
            >(
                *b"8\0 \0M\0B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            iType: (REG_SZ as i32),
            iValue: 8 as INT,
        },
        _WINED3D_SETTINGS {
            szValue: ::core::mem::transmute::<
                [u8; 48],
                [WCHAR; 24],
            >(
                *b"1\x006\0 \0M\0B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            iType: (REG_SZ as i32),
            iValue: 16 as INT,
        },
        _WINED3D_SETTINGS {
            szValue: ::core::mem::transmute::<
                [u8; 48],
                [WCHAR; 24],
            >(
                *b"3\x002\0 \0M\0B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            iType: (REG_SZ as i32),
            iValue: 32 as INT,
        },
        _WINED3D_SETTINGS {
            szValue: ::core::mem::transmute::<
                [u8; 48],
                [WCHAR; 24],
            >(
                *b"6\x004\0 \0M\0B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            iType: (REG_SZ as i32),
            iValue: 64 as INT,
        },
        _WINED3D_SETTINGS {
            szValue: ::core::mem::transmute::<
                [u8; 48],
                [WCHAR; 24],
            >(
                *b"1\x002\08\0 \0M\0B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            iType: (REG_SZ as i32),
            iValue: 128 as INT,
        },
        _WINED3D_SETTINGS {
            szValue: ::core::mem::transmute::<
                [u8; 48],
                [WCHAR; 24],
            >(
                *b"2\x005\x006\0 \0M\0B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            iType: (REG_SZ as i32),
            iValue: 256 as INT,
        },
        _WINED3D_SETTINGS {
            szValue: ::core::mem::transmute::<
                [u8; 48],
                [WCHAR; 24],
            >(
                *b"5\x001\x002\0 \0M\0B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            ),
            iType: (REG_SZ as i32),
            iValue: 512 as INT,
        },
    ]
};

#[no_mangle]
pub static mut gwd3dsDdRender: [WINED3D_SETTINGS; 2] = [
    _WINED3D_SETTINGS {
        szValue: VALUE_DEFAULT,
        iType: (REG_NONE as i32),
        iValue: 0 as INT,
    },
    _WINED3D_SETTINGS {
        szValue: VALUE_GDI,
        iType: (REG_SZ as i32),
        iValue: 0 as INT,
    },
];

// WINED3D_SETTINGS field names are not visible in this preamble; assumed `value`, `type_`, `data`.
#[no_mangle]
pub unsafe extern "C" fn InitControl(
    hWndDlg: HWND,
    hKey: HKEY,
    szKey: PWCHAR,
    pSettings: PWINED3D_SETTINGS,
    iControlId: INT,
    iCount: INT,
) {
    let mut szBuffer: [WCHAR; MAX_KEY_LENGTH as usize] = [0; MAX_KEY_LENGTH as usize];
    let mut dwSize: DWORD = ::core::mem::size_of::<[WCHAR; MAX_KEY_LENGTH as usize]>() as DWORD;
    let mut dwType: DWORD = 0 as DWORD;
    let mut iCurrent: INT;
    let mut iActive: INT = 0 as INT;

    RegQueryValueExW(
        hKey,
        szKey,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    );

    iCurrent = 0 as INT;
    while iCurrent < iCount {
        SendDlgItemMessageW(
            hWndDlg,
            iControlId as ::core::ffi::c_int,
            CB_ADDSTRING as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!((*pSettings.offset(iCurrent as isize)).szValue) as *mut WCHAR as LPARAM,
        );

        if dwSize != 0
            && ((dwType == REG_DWORD as DWORD
                && *(core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR) as ::core::ffi::c_int
                    == (*pSettings.offset(iCurrent as isize)).iValue)
                || (dwType == REG_SZ as DWORD
                    && wcscmp(
                        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
                        core::ptr::addr_of_mut!((*pSettings.offset(iCurrent as isize)).szValue) as *mut WCHAR,
                    ) == 0))
        {
            iActive = iCurrent;
        }
        iCurrent += 1;
    }

    SendDlgItemMessageW(
        hWndDlg,
        iControlId as ::core::ffi::c_int,
        CB_SETCURSEL as UINT,
        iActive as WPARAM,
        0 as LPARAM,
    );
}

unsafe extern "C" fn InitSettings(hWndDlg: HWND) {
    let mut hKey: HKEY = core::ptr::null_mut();
    if (RegCreateKeyExW(HKEY_CURRENT_USER, (KEY_WINE).as_ptr(), 0, core::ptr::null_mut(), 0, MAXIMUM_ALLOWED, core::ptr::null_mut(), core::ptr::addr_of_mut!(hKey), core::ptr::null_mut())) as u32 != ERROR_SUCCESS {
        return;
    }

    /* INIT_CONTROL */ InitControl(hWndDlg, hKey, KEY_GLSL.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsDisable) as _, IDC_GLSL as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsDisable)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* INIT_CONTROL */ InitControl(hWndDlg, hKey, KEY_OFFSCREEN.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsOffscreen) as _, IDC_OFFSCREEN as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsOffscreen)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* INIT_CONTROL */ InitControl(hWndDlg, hKey, KEY_VIDMEMSIZE.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsVidMem) as _, IDC_VIDMEMSIZE as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsVidMem)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* INIT_CONTROL */ InitControl(hWndDlg, hKey, KEY_MULTISAMPLING.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsDisable) as _, IDC_MULTISAMPLING as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsDisable)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* INIT_CONTROL */ InitControl(hWndDlg, hKey, KEY_STRICTDRAWORDERING.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsEnable) as _, IDC_STRICTDRAWORDERING as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsEnable)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* INIT_CONTROL */ InitControl(hWndDlg, hKey, KEY_ALWAYSOFFSCREEN.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsEnable) as _, IDC_ALWAYSOFFSCREEN as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsEnable)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* INIT_CONTROL */ InitControl(hWndDlg, hKey, KEY_DDRENDERER.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsDdRender) as _, IDC_DDRENDERER as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsDdRender)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* INIT_CONTROL */ InitControl(hWndDlg, hKey, KEY_PSLEVEL.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsShaderLvl) as _, IDC_PSLEVEL as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsShaderLvl)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* INIT_CONTROL */ InitControl(hWndDlg, hKey, KEY_VSLEVEL.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsShaderLvl) as _, IDC_VSLEVEL as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsShaderLvl)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* INIT_CONTROL */ InitControl(hWndDlg, hKey, KEY_GSLEVEL.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsShaderLvl) as _, IDC_GSLEVEL as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsShaderLvl)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);

    RegCloseKey(hKey);
}

unsafe extern "C" fn SaveSetting(
    mut hWnd: HWND,
    mut hKey: HKEY,
    mut szKey: PWCHAR,
    mut pCfg: PWINED3D_SETTINGS,
    mut iControlId: INT,
    mut iCount: INT,
) {
    let mut iSel: INT = 0 as INT;
    iSel = SendDlgItemMessageW(
        hWnd,
        iControlId as ::core::ffi::c_int,
        CB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as INT;
    if iSel < 0 as INT || iSel > iCount {
        return;
    }
    if (*pCfg.offset(iSel as isize)).iType == REG_NONE as INT {
        RegDeleteValueW(hKey, szKey as LPCWSTR);
        return;
    }
    if (*pCfg.offset(iSel as isize)).iType == REG_DWORD as INT {
        RegSetValueExW(
            hKey,
            szKey as LPCWSTR,
            0 as DWORD,
            REG_DWORD as DWORD,
            core::ptr::addr_of_mut!((*pCfg.offset(iSel as isize)).iValue) as LPBYTE as *const BYTE,
            core::mem::size_of::<INT>() as DWORD,
        );
        return;
    } else if (*pCfg.offset(iSel as isize)).iType == REG_SZ as INT {
        RegSetValueExW(
            hKey,
            szKey as LPCWSTR,
            0 as DWORD,
            (*pCfg.offset(iSel as isize)).iType as DWORD,
            core::ptr::addr_of_mut!((*pCfg.offset(iSel as isize)).szValue) as *mut WCHAR as LPBYTE as *const BYTE,
            ((wcslen(core::ptr::addr_of_mut!((*pCfg.offset(iSel as isize)).szValue) as *mut WCHAR) as usize)
                .wrapping_add(1 as usize))
                .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD,
        );
    }
}

unsafe extern "C" fn WriteSettings(hWndDlg: HWND) {
    let mut hKey: HKEY = core::ptr::null_mut();

    if (RegOpenKeyExW(HKEY_CURRENT_USER, (KEY_WINE).as_ptr(), 0, KEY_WRITE, core::ptr::addr_of_mut!(hKey))) as u32 != ERROR_SUCCESS {
        return;
    }

    /* SAVE_CONTROL */ SaveSetting(hWndDlg, hKey, KEY_GLSL.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsDisable) as _, IDC_GLSL as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsDisable)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* SAVE_CONTROL */ SaveSetting(hWndDlg, hKey, KEY_OFFSCREEN.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsOffscreen) as _, IDC_OFFSCREEN as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsOffscreen)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* SAVE_CONTROL */ SaveSetting(hWndDlg, hKey, KEY_VIDMEMSIZE.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsVidMem) as _, IDC_VIDMEMSIZE as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsVidMem)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* SAVE_CONTROL */ SaveSetting(hWndDlg, hKey, KEY_MULTISAMPLING.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsDisable) as _, IDC_MULTISAMPLING as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsDisable)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* SAVE_CONTROL */ SaveSetting(hWndDlg, hKey, KEY_STRICTDRAWORDERING.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsEnable) as _, IDC_STRICTDRAWORDERING as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsEnable)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* SAVE_CONTROL */ SaveSetting(hWndDlg, hKey, KEY_ALWAYSOFFSCREEN.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsEnable) as _, IDC_ALWAYSOFFSCREEN as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsEnable)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* SAVE_CONTROL */ SaveSetting(hWndDlg, hKey, KEY_DDRENDERER.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsDdRender) as _, IDC_DDRENDERER as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsDdRender)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* SAVE_CONTROL */ SaveSetting(hWndDlg, hKey, KEY_PSLEVEL.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsShaderLvl) as _, IDC_PSLEVEL as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsShaderLvl)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* SAVE_CONTROL */ SaveSetting(hWndDlg, hKey, KEY_VSLEVEL.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsShaderLvl) as _, IDC_VSLEVEL as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsShaderLvl)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);
    /* SAVE_CONTROL */ SaveSetting(hWndDlg, hKey, KEY_GSLEVEL.as_ptr() as _, core::ptr::addr_of_mut!(gwd3dsShaderLvl) as _, IDC_GSLEVEL as _, (core::mem::size_of_val(&*core::ptr::addr_of!(gwd3dsShaderLvl)) / core::mem::size_of::<WINED3D_SETTINGS>()) as _);

    RegCloseKey(hKey);
}
#[no_mangle]
pub unsafe extern "system" fn GeneralPageProc(hWndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut lppsn: LPPSHNOTIFY;

    match uMsg {
        WM_INITDIALOG => {
            InitSettings(hWndDlg);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            if (LOWORD!(wParam) as u32) > IDC_MIN
                && (LOWORD!(wParam) as u32) < IDC_MAX
                && (HIWORD!(wParam) as u32) == CBN_SELCHANGE
            {
                PropSheet_Changed!(GetParent(hWndDlg), hWndDlg);
            }
        }
        WM_NOTIFY => {
            lppsn = lParam as LPPSHNOTIFY;
            if (*lppsn).hdr.code == PSN_APPLY {
                WriteSettings(hWndDlg);
                return TRUE as INT_PTR;
            }
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}
