// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_6A661B0A8BF4: [u16; 16] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0054, 0x0072, 0x0061, 0x0079,
    0x005F, 0x004D, 0x0061, 0x0069, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_9F1BA324C93C: [u16; 28] = [
    0x0057u16, 0x0072, 0x0069, 0x0074, 0x0065, 0x0047, 0x006C, 0x006F, 0x0062, 0x0061,
    0x006C, 0x0050, 0x0077, 0x0072, 0x0050, 0x006F, 0x006C, 0x0069, 0x0063, 0x0079,
    0x0020, 0x0066, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut g_SystemBatteries: [POWER_ACTION; 3] = [PowerActionNone; 3];

static mut g_PowerButton: [POWER_ACTION; 5] = [PowerActionNone; 5];

static mut g_SleepButton: [POWER_ACTION; 5] = [PowerActionNone; 5];

unsafe extern "C" fn SetSystrayPowerIconState(bEnabled: BOOL) {
    let mut hwndTaskBar: HWND;

    hwndTaskBar = FindWindowW(RUSTCONV_WSTR_6A661B0A8BF4.as_ptr() as *const u16, core::ptr::null_mut());
    if hwndTaskBar.is_null() {
        return;
    }

    SendMessageW(
        hwndTaskBar,
        (WM_USER + 220) as UINT,
        1 as WPARAM,
        bEnabled as LPARAM,
    );
}

unsafe extern "C" fn GetSystrayPowerIconState() -> BOOL {
    let mut hwndTaskBar: HWND;

    hwndTaskBar = FindWindowW(RUSTCONV_WSTR_6A661B0A8BF4.as_ptr(), core::ptr::null_mut());
    if hwndTaskBar.is_null() {
        return (FALSE) as BOOL;
    }

    return SendMessageW(hwndTaskBar, WM_USER + 221, 1, 0) as BOOL;
}

unsafe extern "C" fn AddItem(
    hDlgCtrl: HWND,
    ResourceId: INT,
    lParam: LPARAM,
    lpAction: *mut POWER_ACTION,
) {
    let mut szBuffer: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut Index: LRESULT;

    if (LoadString(
        hApplet,
        ResourceId as UINT,
        szBuffer.as_mut_ptr() as LPWSTR,
        _countof!(szBuffer) as ::core::ffi::c_int,
    ) as usize) < (_countof!(szBuffer) as usize)
    {
        Index = SendMessage(
            hDlgCtrl,
            CB_INSERTSTRING as UINT,
            -(1 as ::core::ffi::c_int) as WPARAM,
            szBuffer.as_mut_ptr() as LPARAM,
        );
        if Index != CB_ERR as LRESULT {
            SendMessage(
                hDlgCtrl,
                CB_SETITEMDATA as UINT,
                Index as WPARAM,
                lParam,
            );
            *lpAction.offset(Index as isize) = lParam as POWER_ACTION;
        }
    }
}

unsafe extern "C" fn FindActionIndex(lpAction: *mut POWER_ACTION, dwActionSize: DWORD, poAction: POWER_ACTION) -> INT {
    let mut Index: INT;
    Index = 0;
    while Index < (dwActionSize as INT) {
        if *lpAction.offset(Index as isize) as ::core::ffi::c_uint == poAction as ::core::ffi::c_uint {
            return Index;
        }
        Index += 1;
    }
    return -1;
}

unsafe extern "C" fn IsBatteryUsed() -> BOOLEAN {
    let mut sbs: SYSTEM_BATTERY_STATE = core::mem::zeroed();
    if CallNtPowerInformation(SystemBatteryState, core::ptr::null_mut(), 0 as ULONG, core::ptr::addr_of_mut!(sbs) as PVOID, core::mem::size_of::<SYSTEM_BATTERY_STATE>() as ULONG) == STATUS_SUCCESS {
        return (sbs.BatteryPresent != 0 && sbs.AcOnLine == 0) as BOOLEAN;
    }
    return FALSE as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn GetPowerActionFromPolicy(
    Policy: *mut POWER_ACTION_POLICY,
    spc: PSYSTEM_POWER_CAPABILITIES,
    bIsLid: BOOL,
) -> POWER_ACTION {
    let mut poAction: POWER_ACTION = PowerActionNone;
    if (*Policy).Action == PowerActionNone {
        if (*Policy).Flags == (POWER_ACTION_UI_ALLOWED | POWER_ACTION_QUERY_ALLOWED) {
            if (*Policy).EventCode == POWER_FORCE_TRIGGER_RESET {
                poAction = PowerActionNone;
            } else if (*Policy).EventCode == POWER_USER_NOTIFY_BUTTON {
                poAction = PowerActionWarmEject;
            } else if (*Policy).EventCode == POWER_USER_NOTIFY_SHUTDOWN {
                poAction = PowerActionShutdown;
            }
        }
    } else {
        poAction = (*Policy).Action;
        if (poAction == PowerActionHibernate) && (IS_PWR_HIBERNATE_ALLOWED!(spc) == 0) {
            poAction = PowerActionSleep;
        }
        if (poAction == PowerActionSleep) && (IS_PWR_SUSPEND_ALLOWED!(spc) == 0) {
            if bIsLid != 0 {
                poAction = PowerActionNone;
            } else {
                poAction = PowerActionShutdown;
            }
        }
    }

    return poAction;
}
#[no_mangle]
pub unsafe extern "C" fn ShowCurrentPowerActionPolicy(
    hDlgCtrl: HWND,
    lpAction: *mut POWER_ACTION,
    dwActionSize: DWORD,
    Policy: *mut POWER_ACTION_POLICY,
    spc: PSYSTEM_POWER_CAPABILITIES,
    bIsLid: BOOL,
) {
    let mut poActionIndex: core::ffi::c_int;
    let mut poAction: POWER_ACTION;

    poAction = GetPowerActionFromPolicy(Policy, spc, bIsLid);
    poActionIndex = FindActionIndex(lpAction, dwActionSize, poAction);

    if poActionIndex < 0 {
        return;
    }

    SendMessage(hDlgCtrl, CB_SETCURSEL as UINT, poActionIndex as WPARAM, 0 as LPARAM);
}
#[no_mangle]
pub unsafe extern "C" fn SaveCurrentPowerActionPolicy(
    hDlgCtrl: HWND,
    Policy: *mut POWER_ACTION_POLICY,
) -> BOOLEAN {
    let mut Index: LRESULT = 0;
    let mut ItemData: LRESULT = 0;

    Index = SendMessage(hDlgCtrl, CB_GETCURSEL, 0, 0);
    if Index == (CB_ERR) as _ {
        return (FALSE) as BOOLEAN;
    }

    ItemData = SendMessage(hDlgCtrl, CB_GETITEMDATA, Index as WPARAM, 0);
    if ItemData == (CB_ERR) as _ {
        return (FALSE) as BOOLEAN;
    }

    match (ItemData) as u32 {
        PowerActionNone => {
            (*Policy).Action = PowerActionNone;
            (*Policy).EventCode = POWER_FORCE_TRIGGER_RESET;
        }
        PowerActionWarmEject => {
            (*Policy).Action = PowerActionNone;
            (*Policy).EventCode = POWER_USER_NOTIFY_BUTTON;
        }
        PowerActionShutdown => {
            (*Policy).Action = PowerActionNone;
            (*Policy).EventCode = POWER_USER_NOTIFY_SHUTDOWN;
        }
        PowerActionSleep | PowerActionHibernate => {
            (*Policy).Action = ItemData as POWER_ACTION;
            (*Policy).EventCode = 0;
        }
        _ => {
            return (FALSE) as BOOLEAN;
        }
    }

    (*Policy).Flags = (POWER_ACTION_UI_ALLOWED | POWER_ACTION_QUERY_ALLOWED) as DWORD;

    return (TRUE) as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "C" fn ShowCurrentPowerActionPolicies(hwndDlg: HWND, spc: PSYSTEM_POWER_CAPABILITIES) {
    if IsBatteryUsed() == 0 {
        ShowCurrentPowerActionPolicy(
            GetDlgItem(hwndDlg, (IDC_LIDCLOSE as i32)),
            core::ptr::addr_of_mut!(g_SystemBatteries) as *mut POWER_ACTION,
            (core::mem::size_of::<[POWER_ACTION; 3]>() / core::mem::size_of::<POWER_ACTION>()) as DWORD,
            core::ptr::addr_of_mut!(gGPP.user.LidCloseAc),
            spc,
            (TRUE) as BOOL,
        );

        ShowCurrentPowerActionPolicy(
            GetDlgItem(hwndDlg, (IDC_POWERBUTTON as i32)),
            core::ptr::addr_of_mut!(g_PowerButton) as *mut POWER_ACTION,
            (core::mem::size_of::<[POWER_ACTION; 5]>() / core::mem::size_of::<POWER_ACTION>()) as DWORD,
            core::ptr::addr_of_mut!(gGPP.user.PowerButtonAc),
            spc,
            (FALSE) as BOOL,
        );

        ShowCurrentPowerActionPolicy(
            GetDlgItem(hwndDlg, (IDC_SLEEPBUTTON as i32)),
            core::ptr::addr_of_mut!(g_SleepButton) as *mut POWER_ACTION,
            (core::mem::size_of::<[POWER_ACTION; 5]>() / core::mem::size_of::<POWER_ACTION>()) as DWORD,
            core::ptr::addr_of_mut!(gGPP.user.SleepButtonAc),
            spc,
            (FALSE) as BOOL,
        );
    } else {
        ShowCurrentPowerActionPolicy(
            GetDlgItem(hwndDlg, (IDC_LIDCLOSE as i32)),
            core::ptr::addr_of_mut!(g_SystemBatteries) as *mut POWER_ACTION,
            (core::mem::size_of::<[POWER_ACTION; 3]>() / core::mem::size_of::<POWER_ACTION>()) as DWORD,
            core::ptr::addr_of_mut!(gGPP.user.LidCloseDc),
            spc,
            (TRUE) as BOOL,
        );

        ShowCurrentPowerActionPolicy(
            GetDlgItem(hwndDlg, (IDC_POWERBUTTON as i32)),
            core::ptr::addr_of_mut!(g_PowerButton) as *mut POWER_ACTION,
            (core::mem::size_of::<[POWER_ACTION; 5]>() / core::mem::size_of::<POWER_ACTION>()) as DWORD,
            core::ptr::addr_of_mut!(gGPP.user.PowerButtonDc),
            spc,
            (FALSE) as BOOL,
        );

        ShowCurrentPowerActionPolicy(
            GetDlgItem(hwndDlg, (IDC_SLEEPBUTTON as i32)),
            core::ptr::addr_of_mut!(g_SleepButton) as *mut POWER_ACTION,
            (core::mem::size_of::<[POWER_ACTION; 5]>() / core::mem::size_of::<POWER_ACTION>()) as DWORD,
            core::ptr::addr_of_mut!(gGPP.user.SleepButtonDc),
            spc,
            (FALSE) as BOOL,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn Adv_InitDialog(hwndDlg: HWND) {
    let mut hList1: HWND = core::ptr::null_mut();
    let mut hList2: HWND = core::ptr::null_mut();
    let mut hList3: HWND = core::ptr::null_mut();
    let mut bSuspend: BOOLEAN;
    let mut bHibernate: BOOLEAN;
    let mut bShutdown: BOOLEAN;
    let mut bEnabled: BOOL;
    let mut spc: SYSTEM_POWER_CAPABILITIES = core::mem::zeroed();

    bEnabled = GetSystrayPowerIconState();

    if bEnabled != 0 {
        gGPP.user.GlobalFlags |= EnableSysTrayBatteryMeter as ULONG;
    } else {
        gGPP.user.GlobalFlags &= !(EnableSysTrayBatteryMeter as ULONG);
    }

    CheckDlgButton(
        hwndDlg,
        IDC_SYSTRAYBATTERYMETER as INT,
        (if bEnabled != 0 { BST_CHECKED } else { BST_UNCHECKED }) as UINT,
    );
    CheckDlgButton(
        hwndDlg,
        IDC_PASSWORDLOGON as INT,
        (if gGPP.user.GlobalFlags & (EnablePasswordLogon as ULONG) != 0 {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        }) as UINT,
    );
    CheckDlgButton(
        hwndDlg,
        IDC_VIDEODIMDISPLAY as INT,
        (if gGPP.user.GlobalFlags & (EnableVideoDimDisplay as ULONG) != 0 {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        }) as UINT,
    );

    GetPwrCapabilities(&raw mut spc);

    bSuspend = IS_PWR_SUSPEND_ALLOWED!(&raw mut spc) as BOOLEAN;
    bHibernate = IS_PWR_HIBERNATE_ALLOWED!(&raw mut spc) as BOOLEAN;
    bShutdown = IS_PWR_POWEROFF_ALLOWED!(&raw mut spc) as BOOLEAN;

    hList1 = GetDlgItem(hwndDlg, IDC_LIDCLOSE as INT);
    SendMessage(
        hList1,
        CB_RESETCONTENT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );

    ZeroMemory!(
        &raw mut g_SystemBatteries as *mut POWER_ACTION as *mut core::ffi::c_void,
        core::mem::size_of::<[POWER_ACTION; 3]>(),
    );
    if spc.SystemBatteriesPresent != 0 {
        AddItem(
            hList1,
            (IDS_PowerActionNone1) as INT,
            PowerActionNone as LPARAM,
            &raw mut g_SystemBatteries as *mut POWER_ACTION,
        );

        if bSuspend != 0 {
            AddItem(
                hList1,
                (IDS_PowerActionSleep) as INT,
                PowerActionSleep as LPARAM,
                &raw mut g_SystemBatteries as *mut POWER_ACTION,
            );
        }

        if bHibernate != 0 {
            AddItem(
                hList1,
                (IDS_PowerActionHibernate) as INT,
                PowerActionHibernate as LPARAM,
                &raw mut g_SystemBatteries as *mut POWER_ACTION,
            );
        }
    } else {
        ShowWindow(
            GetDlgItem(hwndDlg, IDC_VIDEODIMDISPLAY as INT),
            SW_HIDE as INT,
        );
        ShowWindow(
            GetDlgItem(hwndDlg, IDC_SLIDCLOSE as INT),
            SW_HIDE as INT,
        );
        ShowWindow(hList1, SW_HIDE as INT);
    }

    hList2 = GetDlgItem(hwndDlg, IDC_POWERBUTTON as INT);
    SendMessage(
        hList2,
        CB_RESETCONTENT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );

    ZeroMemory!(
        &raw mut g_PowerButton as *mut POWER_ACTION as *mut core::ffi::c_void,
        core::mem::size_of::<[POWER_ACTION; 5]>(),
    );
    if spc.PowerButtonPresent != 0 {
        AddItem(
            hList2,
            (IDS_PowerActionNone1) as INT,
            PowerActionNone as LPARAM,
            &raw mut g_PowerButton as *mut POWER_ACTION,
        );
        AddItem(
            hList2,
            (IDS_PowerActionWarmEject) as INT,
            PowerActionWarmEject as LPARAM,
            &raw mut g_PowerButton as *mut POWER_ACTION,
        );

        if bSuspend != 0 {
            AddItem(
                hList2,
                (IDS_PowerActionSleep) as INT,
                PowerActionSleep as LPARAM,
                &raw mut g_PowerButton as *mut POWER_ACTION,
            );
        }

        if bHibernate != 0 {
            AddItem(
                hList2,
                (IDS_PowerActionHibernate) as INT,
                PowerActionHibernate as LPARAM,
                &raw mut g_PowerButton as *mut POWER_ACTION,
            );
        }

        if bShutdown != 0 {
            AddItem(
                hList2,
                (IDS_PowerActionShutdown) as INT,
                PowerActionShutdown as LPARAM,
                &raw mut g_PowerButton as *mut POWER_ACTION,
            );
        }
    } else {
        ShowWindow(
            GetDlgItem(hwndDlg, IDC_SPOWERBUTTON as INT),
            SW_HIDE as INT,
        );
        ShowWindow(hList2, SW_HIDE as INT);
    }

    hList3 = GetDlgItem(hwndDlg, IDC_SLEEPBUTTON as INT);
    SendMessage(
        hList3,
        CB_RESETCONTENT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );

    ZeroMemory!(
        &raw mut g_SleepButton as *mut POWER_ACTION as *mut core::ffi::c_void,
        core::mem::size_of::<[POWER_ACTION; 5]>(),
    );
    if spc.SleepButtonPresent != 0 {
        AddItem(
            hList3,
            (IDS_PowerActionNone1) as INT,
            PowerActionNone as LPARAM,
            &raw mut g_SleepButton as *mut POWER_ACTION,
        );
        AddItem(
            hList3,
            (IDS_PowerActionWarmEject) as INT,
            PowerActionWarmEject as LPARAM,
            &raw mut g_SleepButton as *mut POWER_ACTION,
        );

        if bSuspend != 0 {
            AddItem(
                hList3,
                (IDS_PowerActionSleep) as INT,
                PowerActionSleep as LPARAM,
                &raw mut g_SleepButton as *mut POWER_ACTION,
            );
        }

        if bHibernate != 0 {
            AddItem(
                hList3,
                (IDS_PowerActionHibernate) as INT,
                PowerActionHibernate as LPARAM,
                &raw mut g_SleepButton as *mut POWER_ACTION,
            );
        }

        if bShutdown != 0 {
            AddItem(
                hList3,
                (IDS_PowerActionShutdown) as INT,
                PowerActionShutdown as LPARAM,
                &raw mut g_SleepButton as *mut POWER_ACTION,
            );
        }
    } else {
        ShowWindow(
            GetDlgItem(hwndDlg, IDC_SSLEEPBUTTON as INT),
            SW_HIDE as INT,
        );
        ShowWindow(hList3, SW_HIDE as INT);
    }

    if ReadGlobalPwrPolicy(&raw mut gGPP) != 0 {
        ShowCurrentPowerActionPolicies(hwndDlg, &raw mut spc);
    }
}

unsafe extern "C" fn Adv_SaveData(hwndDlg: HWND) {
    let mut bSystrayBatteryMeter: BOOL;
    let mut bPasswordLogon: BOOL;
    let mut bVideoDimDisplay: BOOL;

    bSystrayBatteryMeter =
        (IsDlgButtonChecked(hwndDlg, (IDC_SYSTRAYBATTERYMETER as i32)) == BST_CHECKED) as BOOL;

    bPasswordLogon =
        (IsDlgButtonChecked(hwndDlg, (IDC_PASSWORDLOGON as i32)) == BST_CHECKED) as BOOL;

    bVideoDimDisplay =
        (IsDlgButtonChecked(hwndDlg, (IDC_VIDEODIMDISPLAY as i32)) == BST_CHECKED) as BOOL;

    if bSystrayBatteryMeter != 0 {
        if (gGPP.user.GlobalFlags & (EnableSysTrayBatteryMeter as ULONG)) == 0 {
            gGPP.user.GlobalFlags = gGPP.user.GlobalFlags.wrapping_add(EnableSysTrayBatteryMeter as ULONG);
        }
    } else {
        if (gGPP.user.GlobalFlags & (EnableSysTrayBatteryMeter as ULONG)) != 0 {
            gGPP.user.GlobalFlags = gGPP.user.GlobalFlags.wrapping_sub(EnableSysTrayBatteryMeter as ULONG);
        }
    }

    if bPasswordLogon != 0 {
        if (gGPP.user.GlobalFlags & (EnablePasswordLogon as ULONG)) == 0 {
            gGPP.user.GlobalFlags = gGPP.user.GlobalFlags.wrapping_add(EnablePasswordLogon as ULONG);
        }
    } else {
        if (gGPP.user.GlobalFlags & (EnablePasswordLogon as ULONG)) != 0 {
            gGPP.user.GlobalFlags = gGPP.user.GlobalFlags.wrapping_sub(EnablePasswordLogon as ULONG);
        }
    }

    if bVideoDimDisplay != 0 {
        if (gGPP.user.GlobalFlags & (EnableVideoDimDisplay as ULONG)) == 0 {
            gGPP.user.GlobalFlags = gGPP.user.GlobalFlags.wrapping_add(EnableVideoDimDisplay as ULONG);
        }
    } else {
        if (gGPP.user.GlobalFlags & (EnableVideoDimDisplay as ULONG)) != 0 {
            gGPP.user.GlobalFlags = gGPP.user.GlobalFlags.wrapping_sub(EnableVideoDimDisplay as ULONG);
        }
    }

    if IsBatteryUsed() == 0 {
        SaveCurrentPowerActionPolicy(
            GetDlgItem(hwndDlg, (IDC_POWERBUTTON as i32)),
            &raw mut gGPP.user.PowerButtonAc,
        );
        SaveCurrentPowerActionPolicy(
            GetDlgItem(hwndDlg, (IDC_SLEEPBUTTON as i32)),
            &raw mut gGPP.user.SleepButtonAc,
        );
    } else {
    }

    if WriteGlobalPwrPolicy(&raw mut gGPP) == 0 {
        MessageBox(
            hwndDlg,
            RUSTCONV_WSTR_9F1BA324C93C.as_ptr(),
            core::ptr::null_mut(),
            MB_OK as UINT,
        );
    }

    SetSystrayPowerIconState(bSystrayBatteryMeter);

//    Adv_InitDialog(hwndDlg);
}
#[no_mangle]
pub unsafe extern "system" fn AdvancedDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            Adv_InitDialog(hwndDlg);
            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDC_SYSTRAYBATTERYMETER | IDC_PASSWORDLOGON | IDC_VIDEODIMDISPLAY => {
                    if (HIWORD!(wParam) as ::core::ffi::c_int) as u32 == BN_CLICKED {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_LIDCLOSE | IDC_POWERBUTTON | IDC_SLEEPBUTTON => {
                    if (HIWORD!(wParam) as ::core::ffi::c_int) as u32 == CBN_SELCHANGE {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                _ => {}
            }
        }

        WM_NOTIFY => {
            match (*(lParam as LPNMHDR)).code as ::core::ffi::c_int {
                PSN_APPLY => {
                    Adv_SaveData(hwndDlg);
                    return TRUE as INT_PTR;
                }

                PSN_SETACTIVE => {
                    Adv_InitDialog(hwndDlg);
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
