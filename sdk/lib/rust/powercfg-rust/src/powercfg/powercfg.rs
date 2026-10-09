use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hApplet: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut gGPP: GLOBAL_POWER_POLICY = _GLOBAL_POWER_POLICY {
    user: _GLOBAL_USER_POWER_POLICY {
        Revision: 0,
        PowerButtonAc: _POWER_ACTION_POLICY {
            Action: PowerActionNone,
            Flags: 0,
            EventCode: 0,
        },
        PowerButtonDc: _POWER_ACTION_POLICY {
            Action: PowerActionNone,
            Flags: 0,
            EventCode: 0,
        },
        SleepButtonAc: _POWER_ACTION_POLICY {
            Action: PowerActionNone,
            Flags: 0,
            EventCode: 0,
        },
        SleepButtonDc: _POWER_ACTION_POLICY {
            Action: PowerActionNone,
            Flags: 0,
            EventCode: 0,
        },
        LidCloseAc: _POWER_ACTION_POLICY {
            Action: PowerActionNone,
            Flags: 0,
            EventCode: 0,
        },
        LidCloseDc: _POWER_ACTION_POLICY {
            Action: PowerActionNone,
            Flags: 0,
            EventCode: 0,
        },
        DischargePolicy: [_SYSTEM_POWER_LEVEL {
            Enable: 0,
            Spare: [0; 3],
            BatteryLevel: 0,
            PowerPolicy: _POWER_ACTION_POLICY {
                Action: PowerActionNone,
                Flags: 0,
                EventCode: 0,
            },
            MinSystemState: PowerSystemUnspecified,
        }; 4],
        GlobalFlags: 0,
    },
    mach: _GLOBAL_MACHINE_POWER_POLICY {
        Revision: 0,
        LidOpenWakeAc: PowerSystemUnspecified,
        LidOpenWakeDc: PowerSystemUnspecified,
        BroadcastCapacityResolution: 0,
    },
};

#[no_mangle]
pub static mut langSel: [TCHAR; 255] = [0; 255];

pub const NUM_APPLETS: i32 = 1;

unsafe extern "system" fn PropSheetAddPage(hpage: HPROPSHEETPAGE, lParam: LPARAM) -> BOOL {
    let ppsh: *mut PROPSHEETHEADERA = lParam as *mut PROPSHEETHEADERA;
    if !ppsh.is_null() && (*ppsh).nPages < MAX_POWER_PAGES {
        let idx = (*ppsh).nPages;
        (*ppsh).nPages += 1;
        *(*ppsh).__bindgen_anon_3.phpage.add(idx as usize) = hpage;
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn InitPropSheetPage(
    mut ppsh: *mut PROPSHEETHEADERW,
    mut idDlg: WORD,
    mut DlgProc: DLGPROC,
) -> BOOL {
    let mut hPage: HPROPSHEETPAGE;
    let mut psp: PROPSHEETPAGEW = core::mem::zeroed();

    if (*ppsh).nPages < MAX_POWER_PAGES as UINT {
        ZeroMemory!(
            core::ptr::addr_of_mut!(psp) as *mut ::core::ffi::c_void,
            core::mem::size_of::<PROPSHEETPAGEW>() as size_t,
        );
        psp.dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as DWORD;
        psp.dwFlags = PSP_DEFAULT as DWORD;
        psp.hInstance = hApplet;
        psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(idDlg) as LPCWSTR;
        psp.pfnDlgProc = DlgProc;

        hPage = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp) as LPCPROPSHEETPAGEW);
        if !hPage.is_null() {
            return PropSheetAddPage(hPage, ppsh as LPARAM);
        }
    }

    return FALSE as BOOL;
}

unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: UINT, lParam: LPARAM) -> i32 {
    // NOTE: This callback is needed to set large icon correctly.
    let mut hIcon: HICON = core::ptr::null_mut();

    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW!(IDC_CPLICON));
            SendMessageW(hwndDlg, WM_SETICON, (ICON_BIG) as WPARAM, hIcon as LPARAM);
        }
        _ => {}
    }
    return 0;
}
unsafe extern "system" fn Applet(mut hwnd: HWND, mut uMsg: UINT, mut wParam: LPARAM, mut lParam: LPARAM) -> LONG {
    let mut hpsp: [HPROPSHEETPAGE; (MAX_POWER_PAGES as usize)] = core::mem::zeroed();
    let mut psh: PROPSHEETHEADERW = core::mem::zeroed();
    let mut hpsxa: HPSXA = core::ptr::null_mut();
    let mut spc: SYSTEM_POWER_CAPABILITIES = core::mem::zeroed();
    let mut ret: LONG;

    let _ = uMsg;
    let _ = wParam;
    let _ = lParam;

    core::ptr::write_bytes(core::ptr::addr_of_mut!(psh), 0, 1);
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as DWORD;
    psh.dwFlags = (PSH_PROPTITLE | PSH_USEICONID | PSH_USECALLBACK) as DWORD;
    psh.hwndParent = hwnd;
    psh.hInstance = hApplet;
    psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDC_CPLICON) as LPCWSTR;
    psh.pszCaption = MAKEINTRESOURCEW!(IDS_CPLNAME) as LPCWSTR;
    psh.nPages = 0 as UINT;
    psh.__bindgen_anon_2.nStartPage = 0 as UINT;
    psh.__bindgen_anon_3.phpage = core::ptr::addr_of_mut!(hpsp) as *mut HPROPSHEETPAGE;
    psh.pfnCallback = Some(PropSheetProc);

    if GetPwrCapabilities(core::ptr::addr_of_mut!(spc)) == 0 {
        return GetLastError() as LONG;
    }

    if spc.SystemBatteriesPresent != 0 {
        InitPropSheetPage(
            core::ptr::addr_of_mut!(psh),
            IDD_POWERSCHEMESPAGE_ACDC as WORD,
            Some(PowerSchemesDlgProc),
        );
        InitPropSheetPage(
            core::ptr::addr_of_mut!(psh),
            IDD_PROPPAGEALARMS as WORD,
            Some(AlarmsDlgProc),
        );
        InitPropSheetPage(
            core::ptr::addr_of_mut!(psh),
            IDD_PROPPAGEPOWERMETER as WORD,
            Some(PowerMeterDlgProc),
        );
    } else {
        InitPropSheetPage(
            core::ptr::addr_of_mut!(psh),
            IDD_POWERSCHEMESPAGE_AC as WORD,
            Some(PowerSchemesDlgProc),
        );
    }
    InitPropSheetPage(
        core::ptr::addr_of_mut!(psh),
        IDD_PROPPAGEADVANCED as WORD,
        Some(AdvancedDlgProc),
    );
    if spc.SystemS4 != 0 {
        /* ACPI S4 state is supported, display the "Hibernate" page
         * where we can enable or disable the hibernation file */
        InitPropSheetPage(
            core::ptr::addr_of_mut!(psh),
            IDD_PROPPAGEHIBERNATE as WORD,
            Some(HibernateDlgProc),
        );
    }

    // TODO: Add UPS page (check spc.UpsPresent)

    /* Load additional pages provided by shell extensions */
    let REGSTR_PATH_POWER: [u16; 64] = [
        'S' as u16, 'o' as u16, 'f' as u16, 't' as u16, 'w' as u16, 'a' as u16, 'r' as u16, 'e' as u16,
        '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16,
        'f' as u16, 't' as u16, '\\' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16,
        'w' as u16, 's' as u16, '\\' as u16, 'C' as u16, 'u' as u16, 'r' as u16, 'r' as u16, 'e' as u16,
        'n' as u16, 't' as u16, 'V' as u16, 'e' as u16, 'r' as u16, 's' as u16, 'i' as u16, 'o' as u16,
        'n' as u16, '\\' as u16, 'C' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'r' as u16, 'o' as u16,
        'l' as u16, 's' as u16, ' ' as u16, 'F' as u16, 'o' as u16, 'l' as u16, 'd' as u16, 'e' as u16,
        'r' as u16, '\\' as u16, 'P' as u16, 'o' as u16, 'w' as u16, 'e' as u16, 'r' as u16, 0,
    ];
    hpsxa = SHCreatePropSheetExtArray(
        HKEY_LOCAL_MACHINE,
        REGSTR_PATH_POWER.as_ptr(),
        (MAX_POWER_PAGES as UINT).wrapping_sub(psh.nPages),
    );
    if !hpsxa.is_null() {
        SHAddFromPropSheetExtArray(
            hpsxa,
            Some(PropSheetAddPage),
            core::ptr::addr_of_mut!(psh) as LPARAM,
        );
    }

    ret = (PropertySheet(core::ptr::addr_of_mut!(psh) as LPCPROPSHEETHEADERW)
        != -(1) as INT_PTR) as core::ffi::c_int as LONG;

    if !hpsxa.is_null() {
        SHDestroyPropSheetExtArray(hpsxa);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(
    hwndCPl: HWND,
    uMsg: UINT,
    lParam1: LPARAM,
    lParam2: LPARAM,
) -> LONG {
    let i: UINT = lParam1 as UINT;

    match uMsg {
        CPL_INIT => {
            return (TRUE) as LONG;
        }

        CPL_GETCOUNT => {
            return NUM_APPLETS;
        }

        CPL_INQUIRE => {
            if i < (NUM_APPLETS as u32) {
                let CPlInfo: *mut CPLINFO = lParam2 as *mut CPLINFO;
                (*CPlInfo).lData = 0;
                (*CPlInfo).idIcon = IDC_CPLICON as i32;
                (*CPlInfo).idName = (IDS_CPLNAME as i32);
                (*CPlInfo).idInfo = (IDS_CPLDESCRIPTION as i32);
            } else {
                return (TRUE) as LONG;
            }
        }

        CPL_DBLCLK => {
            if i < (NUM_APPLETS as u32) {
                Applet(hwndCPl, uMsg, lParam1, lParam2);
            } else {
                return (TRUE) as LONG;
            }
        }

        _ => {}
    }

    return (FALSE) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, dwReason: DWORD, lpvReserved: LPVOID) -> BOOLEAN {
    UNREFERENCED_PARAMETER!(lpvReserved);

    match dwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
            hApplet = hinstDLL;
        }
        _ => {}
    }

    return TRUE as BOOLEAN;
}
