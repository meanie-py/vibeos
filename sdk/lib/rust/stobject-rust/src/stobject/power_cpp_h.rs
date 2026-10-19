use crate::prelude::*;
use crate::stobject::stobject_cpp_h::g_hInstance;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type PWRSCHEMECONTEXT = _PWRSCHEMECONTEXT;
pub type PPWRSCHEMECONTEXT = *mut _PWRSCHEMECONTEXT;

// IDI_BATTCAP0..4 / IDI_BATTCHA0..4 are resource IDs from the resource header; the caller must provide these.
pub static mut br_icons: [i32; 5] = [
    IDI_BATTCAP0 as i32,
    IDI_BATTCAP1 as i32,
    IDI_BATTCAP2 as i32,
    IDI_BATTCAP3 as i32,
    IDI_BATTCAP4 as i32,
];
pub static mut bc_icons: [i32; 5] = [
    IDI_BATTCHA0 as i32,
    IDI_BATTCHA1 as i32,
    IDI_BATTCHA2 as i32,
    IDI_BATTCHA3 as i32,
    IDI_BATTCHA4 as i32,
];

pub static mut g_strTooltip: CString = CString::new();

pub static mut g_hIconBattery: HICON = core::ptr::null_mut();



unsafe extern "C" fn Quantize(p: u8) -> u32 {
    if p <= 12 {
        return 0;
    } else if p > 12 && p <= 37 {
        return 1;
    } else if p > 37 && p <= 62 {
        return 2;
    } else if p > 62 && p <= 87 {
        return 3;
    } else {
        return 4;
    }
}

unsafe extern "C" fn DynamicLoadIcon(hinst: HINSTANCE) -> HICON {
    let mut PowerStatus: SYSTEM_POWER_STATUS = core::mem::zeroed();
    let hBatIcon: HICON;
    let uiHour: UINT;
    let uiMin: UINT;
    let mut index: UINT = 0u32.wrapping_sub(1 as UINT);

    if GetSystemPowerStatus(core::ptr::addr_of_mut!(PowerStatus)) == 0 ||
        PowerStatus.ACLineStatus == (AC_LINE_UNKNOWN as u8) ||
        PowerStatus.BatteryFlag == (BATTERY_FLAG_UNKNOWN as u8)
    {
        hBatIcon = LoadIcon(hinst, MAKEINTRESOURCE!(IDI_BATTCAP_ERR));
        (*core::ptr::addr_of_mut!(g_strTooltip)).LoadStringW(IDS_PWR_UNKNOWN_REMAINING);
        return hBatIcon;
    }

    if ((PowerStatus.BatteryFlag & (BATTERY_FLAG_NO_BATTERY as u8)) == 0) &&
        (PowerStatus.BatteryLifePercent == (BATTERY_PERCENTAGE_UNKNOWN as u8))
    {
        hBatIcon = LoadIcon(hinst, MAKEINTRESOURCE!(IDI_BATTCAP_ERR));
        (*core::ptr::addr_of_mut!(g_strTooltip)).LoadStringW(IDS_PWR_UNKNOWN_REMAINING);
    }
    else if ((PowerStatus.BatteryFlag & (BATTERY_FLAG_NO_BATTERY as u8)) == 0) &&
        ((PowerStatus.BatteryFlag & (BATTERY_FLAG_CHARGING as u8)) == (BATTERY_FLAG_CHARGING as u8))
    {
        index = Quantize(PowerStatus.BatteryLifePercent);
        hBatIcon = LoadIcon(hinst, MAKEINTRESOURCE!(bc_icons[index as usize]));
        atl_format!(
            (*core::ptr::addr_of_mut!(g_strTooltip)),
            CStringW::from_resource(IDS_PWR_CHARGING).GetString(),
            PowerStatus.BatteryLifePercent
        );
    }
    else if PowerStatus.ACLineStatus == (AC_LINE_ONLINE as u8) &&
             PowerStatus.BatteryLifePercent == 100
    {
        index = Quantize(PowerStatus.BatteryLifePercent);
        hBatIcon = LoadIcon(hinst, MAKEINTRESOURCE!(bc_icons[index as usize]));
        (*core::ptr::addr_of_mut!(g_strTooltip)).LoadStringW(IDS_PWR_FULLY_CHARGED);
    }
    else if ((PowerStatus.BatteryFlag & (BATTERY_FLAG_NO_BATTERY as u8)) == 0) &&
             ((PowerStatus.BatteryFlag & (BATTERY_FLAG_CHARGING as u8)) == 0)
    {
        index = Quantize(PowerStatus.BatteryLifePercent);
        hBatIcon = LoadIcon(hinst, MAKEINTRESOURCE!(br_icons[index as usize]));

        if PowerStatus.BatteryLifeTime != BATTERY_UNKNOWN_TIME
        {
            uiHour = PowerStatus.BatteryLifeTime / (HOUR_IN_SECS as u32);
            uiMin = (PowerStatus.BatteryLifeTime % (HOUR_IN_SECS as u32)) / (MIN_IN_SECS as u32);

            if uiHour != 0
            {
                atl_format!(
                    (*core::ptr::addr_of_mut!(g_strTooltip)),
                    CStringW::from_resource(IDS_PWR_HOURS_REMAINING).GetString(),
                    uiHour,
                    uiMin,
                    PowerStatus.BatteryLifePercent
                );
            }
            else
            {
                atl_format!(
                    (*core::ptr::addr_of_mut!(g_strTooltip)),
                    CStringW::from_resource(IDS_PWR_MINUTES_REMAINING).GetString(),
                    uiMin,
                    PowerStatus.BatteryLifePercent
                );
            }
        }
        else
        {
            atl_format!(
                (*core::ptr::addr_of_mut!(g_strTooltip)),
                CStringW::from_resource(IDS_PWR_PERCENT_REMAINING).GetString(),
                PowerStatus.BatteryLifePercent
            );
        }
    }
    else
    {
        hBatIcon = LoadIcon(hinst, MAKEINTRESOURCE!(IDI_POWER_AC));
        (*core::ptr::addr_of_mut!(g_strTooltip)).LoadStringW(IDS_PWR_AC);
    }

    return hBatIcon;
}
#[no_mangle]
pub unsafe extern "system" fn Power_Init(pSysTray: *mut CSysTray) -> HRESULT {
    g_hIconBattery = DynamicLoadIcon(g_hInstance);

    return (*pSysTray).NotifyIcon((NIM_ADD) as INT, ID_ICON_POWER, g_hIconBattery, g_strTooltip.GetString(), 0);
}

#[no_mangle]
pub unsafe extern "system" fn Power_Update(pSysTray: *mut CSysTray) -> HRESULT {
    g_hIconBattery = DynamicLoadIcon(g_hInstance);

    return (*pSysTray).NotifyIcon((NIM_MODIFY) as INT, ID_ICON_POWER, g_hIconBattery, (*core::ptr::addr_of_mut!(g_strTooltip)).GetString(), 0);
}
#[no_mangle]
pub unsafe extern "system" fn Power_Shutdown(pSysTray: *mut CSysTray) -> HRESULT {
    return (*pSysTray).NotifyIcon(
        (NIM_DELETE) as INT,
        ID_ICON_POWER,
        core::ptr::null_mut(),
        core::ptr::null(),
        0,
    );
}

unsafe extern "C" fn _RunPower() {
    CSysTray::RunDll(b"powercfg.cpl\0".as_ptr() as *const i8, b"\0".as_ptr() as *const i8);
}

unsafe extern "C" fn _ShowContextMenu__power(pSysTray: *mut CSysTray) {
    let mut strOpen: CStringW = CStringW::from_ansi(IDS_PWR_PROPERTIES as LPCSTR);
    let hPopup: HMENU = CreatePopupMenu();
    AppendMenuW(hPopup, MF_STRING as UINT, IDS_PWR_PROPERTIES as UINT_PTR, strOpen.GetString());
    SetMenuDefaultItem(hPopup, IDS_PWR_PROPERTIES as UINT, FALSE as UINT);

    SetForegroundWindow((*pSysTray).GetHWnd());
    let flags: DWORD = (TPM_RETURNCMD as DWORD)
        | (TPM_NONOTIFY as DWORD)
        | (TPM_RIGHTALIGN as DWORD)
        | (TPM_BOTTOMALIGN as DWORD);
    let mut pt: POINT = core::mem::zeroed();
    GetCursorPos(&mut pt);

    let id: DWORD = TrackPopupMenuEx(hPopup, flags as UINT,
        pt.x, pt.y,
        (*pSysTray).GetHWnd(), core::ptr::null_mut()) as DWORD;

    match id {
        IDS_PWR_PROPERTIES => {
            _RunPower();
        }
        _ => {}
    }
    DestroyMenu(hPopup);
}

unsafe extern "system" fn PowerSchemesEnumProc(uiIndex: UINT, dwName: DWORD, sName: LPWSTR, dwDesc: DWORD, sDesc: LPWSTR, pp: PPOWER_POLICY, lParam: LPARAM) -> BOOLEAN {
    let PowerSchemeContext: PPWRSCHEMECONTEXT = lParam as PPWRSCHEMECONTEXT;

    if AppendMenuW((*PowerSchemeContext).hPopup, MF_STRING, (uiIndex + 1) as UINT_PTR, sName) != 0
    {
        if (*PowerSchemeContext).uiFirst == 0
        {
            (*PowerSchemeContext).uiFirst = uiIndex + 1;
        }

        (*PowerSchemeContext).uiLast = uiIndex + 1;
    }

    return (TRUE) as BOOLEAN;
}

unsafe extern "C" fn ShowPowerSchemesPopupMenu(pSysTray: *mut CSysTray) {
    let mut PowerSchemeContext: PWRSCHEMECONTEXT = PWRSCHEMECONTEXT {
        hPopup: core::ptr::null_mut(),
        uiFirst: 0,
        uiLast: 0,
    };
    let mut uiActiveScheme: UINT = core::mem::zeroed();
    let mut id: DWORD;
    let mut pt: POINT = core::mem::zeroed();
    PowerSchemeContext.hPopup = CreatePopupMenu();
    EnumPwrSchemes(Some(PowerSchemesEnumProc), core::ptr::addr_of_mut!(PowerSchemeContext) as LPARAM);

    if GetActivePwrScheme(core::ptr::addr_of_mut!(uiActiveScheme)) != 0 {
        CheckMenuRadioItem(
            PowerSchemeContext.hPopup,
            PowerSchemeContext.uiFirst,
            PowerSchemeContext.uiLast,
            uiActiveScheme + 1,
            MF_BYCOMMAND,
        );
    }

    SetForegroundWindow((*pSysTray).GetHWnd());
    GetCursorPos(core::ptr::addr_of_mut!(pt));

    id = (TrackPopupMenuEx(
        PowerSchemeContext.hPopup,
        TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTALIGN | TPM_BOTTOMALIGN,
        pt.x,
        pt.y,
        (*pSysTray).GetHWnd(),
        core::ptr::null_mut(),
    )) as u32;

    DestroyMenu(PowerSchemeContext.hPopup);

    if id != 0 {
        SetActivePwrScheme(id.wrapping_sub(1), core::ptr::null_mut(), core::ptr::null_mut());
    }
}
#[no_mangle]
pub unsafe extern "system" fn Power_Message(pSysTray: *mut CSysTray, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, lResult: &mut LRESULT) -> HRESULT {
    'sw: {
        if uMsg == WM_USER + 220 {
            TRACE!("Power_Message: WM_USER+220\n");
            if wParam == (POWER_SERVICE_FLAG as _) {
                if lParam != 0 {
                    (*pSysTray).EnableService(POWER_SERVICE_FLAG, (TRUE) as BOOL);
                    return Power_Init(pSysTray);
                } else {
                    (*pSysTray).EnableService(POWER_SERVICE_FLAG, (FALSE) as BOOL);
                    return Power_Shutdown(pSysTray);
                }
            }
            return S_FALSE;
        } else if uMsg == WM_USER + 221 {
            TRACE!("Power_Message: WM_USER+221\n");
            if wParam == (POWER_SERVICE_FLAG as _) {
                *lResult = (*pSysTray).IsServiceEnabled(POWER_SERVICE_FLAG) as LRESULT;
                return S_OK;
            }
            return S_FALSE;
        } else if uMsg == WM_TIMER {
            if wParam == (POWER_TIMER_ID as _) {
                KillTimer((*pSysTray).GetHWnd(), (POWER_TIMER_ID as _));
                ShowPowerSchemesPopupMenu(pSysTray);
            }
            break 'sw;
        } else if uMsg == ID_ICON_POWER {
            Power_Update(pSysTray);

            'inner: {
                if lParam == (WM_LBUTTONDOWN as LPARAM) {
                    SetTimer((*pSysTray).GetHWnd(), (POWER_TIMER_ID as _), GetDoubleClickTime(), None);
                } else if lParam == (WM_LBUTTONUP as LPARAM) {
                } else if lParam == (WM_LBUTTONDBLCLK as LPARAM) {
                    KillTimer((*pSysTray).GetHWnd(), (POWER_TIMER_ID as _));
                    _RunPower();
                } else if lParam == (WM_RBUTTONDOWN as LPARAM) {
                } else if lParam == (WM_RBUTTONUP as LPARAM) {
                    _ShowContextMenu__power(pSysTray);
                } else if lParam == (WM_RBUTTONDBLCLK as LPARAM) {
                } else if lParam == (WM_MOUSEMOVE as LPARAM) {
                }
            }
            return S_OK;
        } else {
            TRACE!("Power_Message received for unknown ID %d, ignoring.\n");
            return S_FALSE;
        }
    }

    return S_FALSE;
}

impl _PWRSCHEMECONTEXT {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
