use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn Ala_InitData(hwndDlg: HWND) -> BOOLEAN {
    let mut szAction: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szText: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szSound: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szMessage: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szTemp: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szBatteryLevel: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szProgram: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if ReadGlobalPwrPolicy(core::ptr::addr_of_mut!(gGPP)) == 0 {
        return FALSE as BOOLEAN;
    }

    if (gGPP.user.DischargePolicy[DISCHARGE_POLICY_LOW as usize].Enable as ::core::ffi::c_int) != 0 {
        CheckDlgButton(
            hwndDlg,
            IDC_ALARM1 as ::core::ffi::c_int,
            (if (gGPP.user.DischargePolicy[DISCHARGE_POLICY_LOW as usize].Enable as ::core::ffi::c_int) != 0 {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            }) as UINT,
        );

        if LoadString(
            hApplet,
            IDS_PERCENT as UINT,
            core::ptr::addr_of_mut!(szTemp) as LPWSTR,
            _countof!(szTemp) as ::core::ffi::c_int,
        ) != 0
        {
            _stprintf(
                (core::ptr::addr_of_mut!(szBatteryLevel) as *mut TCHAR).cast(),
                (core::ptr::addr_of_mut!(szTemp) as *mut TCHAR).cast(),
                gGPP.user.DischargePolicy[DISCHARGE_POLICY_LOW as usize].BatteryLevel,
            );
            SetDlgItemText(
                hwndDlg,
                IDC_ALARMVALUE1 as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(szBatteryLevel) as *mut TCHAR as LPCWSTR,
            );
        }

        SendDlgItemMessage(
            hwndDlg,
            IDC_ALARMBAR1 as ::core::ffi::c_int,
            TBM_SETRANGE as UINT,
            TRUE as WPARAM,
            MAKELONG!(0, 100) as LPARAM,
        );
        SendDlgItemMessage(
            hwndDlg,
            IDC_ALARMBAR1 as ::core::ffi::c_int,
            TBM_SETTICFREQ as UINT,
            TRUE as WPARAM,
            20 as LPARAM,
        );
        SendDlgItemMessage(
            hwndDlg,
            IDC_ALARMBAR1 as ::core::ffi::c_int,
            TBM_SETPOS as UINT,
            TRUE as WPARAM,
            gGPP.user.DischargePolicy[DISCHARGE_POLICY_LOW as usize].BatteryLevel as LPARAM,
        );

        if LoadString(
            hApplet,
            (gGPP.user.DischargePolicy[DISCHARGE_POLICY_LOW as usize].PowerPolicy.Action as ::core::ffi::c_int
                + IDS_PowerActionNone1 as ::core::ffi::c_int) as UINT,
            core::ptr::addr_of_mut!(szAction) as LPWSTR,
            _countof!(szAction) as ::core::ffi::c_int,
        ) != 0
        {
            SetDlgItemText(
                hwndDlg,
                IDC_ALARMAKTION1 as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(szAction) as *mut TCHAR as LPCWSTR,
            );
        }

        ZeroMemory!(
            core::ptr::addr_of_mut!(szMessage) as *mut core::ffi::c_void,
            core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>(),
        );
        LoadString(
            hApplet,
            IDS_NOACTION as UINT,
            core::ptr::addr_of_mut!(szMessage) as LPWSTR,
            _countof!(szMessage) as ::core::ffi::c_int,
        );

        if (LOWORD!(gGPP.user.DischargePolicy[DISCHARGE_POLICY_LOW as usize].PowerPolicy.EventCode) as ::core::ffi::c_int
            & POWER_LEVEL_USER_NOTIFY_TEXT as ::core::ffi::c_int) != 0
        {
            if (LOWORD!(gGPP.user.DischargePolicy[DISCHARGE_POLICY_LOW as usize].PowerPolicy.EventCode) as ::core::ffi::c_int
                & POWER_LEVEL_USER_NOTIFY_SOUND as ::core::ffi::c_int) != 0
            {
                if LoadString(
                    hApplet,
                    IDS_SOUND as UINT,
                    core::ptr::addr_of_mut!(szSound) as LPWSTR,
                    _countof!(szSound) as ::core::ffi::c_int,
                ) != 0
                    && LoadString(
                        hApplet,
                        IDS_TEXT as UINT,
                        core::ptr::addr_of_mut!(szText) as LPWSTR,
                        _countof!(szText) as ::core::ffi::c_int,
                    ) != 0
                {
                    static FMT_SOUND_TEXT: [u16; 7] = ['%' as u16, 's' as u16, ',' as u16, ' ' as u16, '%' as u16, 's' as u16, 0];
                    _stprintf(
                        (core::ptr::addr_of_mut!(szMessage) as *mut TCHAR).cast(),
                        (core::ptr::addr_of!(FMT_SOUND_TEXT) as *const TCHAR).cast(),
                        core::ptr::addr_of_mut!(szSound) as *mut TCHAR,
                        core::ptr::addr_of_mut!(szText) as *mut TCHAR,
                    );
                }
            } else {
                if LoadString(
                    hApplet,
                    IDS_TEXT as UINT,
                    core::ptr::addr_of_mut!(szText) as LPWSTR,
                    _countof!(szText) as ::core::ffi::c_int,
                ) != 0
                {
                    static FMT_TEXT: [u16; 3] = ['%' as u16, 's' as u16, 0];
                    _stprintf(
                        (core::ptr::addr_of_mut!(szMessage) as *mut TCHAR).cast(),
                        (core::ptr::addr_of!(FMT_TEXT) as *const TCHAR).cast(),
                        core::ptr::addr_of_mut!(szText) as *mut TCHAR,
                    );
                }
            }
        } else {
            if (LOWORD!(gGPP.user.DischargePolicy[DISCHARGE_POLICY_LOW as usize].PowerPolicy.EventCode) as ::core::ffi::c_int
                & POWER_LEVEL_USER_NOTIFY_SOUND as ::core::ffi::c_int) != 0
            {
                if LoadString(
                    hApplet,
                    IDS_SOUND as UINT,
                    core::ptr::addr_of_mut!(szSound) as LPWSTR,
                    _countof!(szSound) as ::core::ffi::c_int,
                ) != 0
                {
                    static FMT_SOUND: [u16; 3] = ['%' as u16, 's' as u16, 0];
                    _stprintf(
                        (core::ptr::addr_of_mut!(szMessage) as *mut TCHAR).cast(),
                        (core::ptr::addr_of!(FMT_SOUND) as *const TCHAR).cast(),
                        core::ptr::addr_of_mut!(szSound) as *mut TCHAR,
                    );
                }
            }
        }

        SetDlgItemText(
            hwndDlg,
            IDC_ALARMMSG1 as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(szMessage) as *mut TCHAR as LPCWSTR,
        );

        if LoadString(
            hApplet,
            IDS_PowerActionNone2 as UINT,
            core::ptr::addr_of_mut!(szProgram) as LPWSTR,
            _countof!(szProgram) as ::core::ffi::c_int,
        ) != 0
        {
            SetDlgItemText(
                hwndDlg,
                IDC_ALARMPROG1 as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(szProgram) as *mut TCHAR as LPCWSTR,
            );
        }
    }

    if (gGPP.user.DischargePolicy[DISCHARGE_POLICY_CRITICAL as usize].Enable as ::core::ffi::c_int) != 0 {
        CheckDlgButton(
            hwndDlg,
            IDC_ALARM2 as ::core::ffi::c_int,
            (if (gGPP.user.DischargePolicy[DISCHARGE_POLICY_CRITICAL as usize].Enable as ::core::ffi::c_int) != 0 {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            }) as UINT,
        );

        if LoadString(
            hApplet,
            IDS_PERCENT as UINT,
            core::ptr::addr_of_mut!(szTemp) as LPWSTR,
            _countof!(szTemp) as ::core::ffi::c_int,
        ) != 0
        {
            _stprintf(
                (core::ptr::addr_of_mut!(szBatteryLevel) as *mut TCHAR).cast(),
                (core::ptr::addr_of_mut!(szTemp) as *mut TCHAR).cast(),
                gGPP.user.DischargePolicy[DISCHARGE_POLICY_CRITICAL as usize].BatteryLevel,
            );
            SetDlgItemText(
                hwndDlg,
                IDC_ALARMVALUE2 as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(szBatteryLevel) as *mut TCHAR as LPCWSTR,
            );
        }

        SendDlgItemMessage(
            hwndDlg,
            IDC_ALARMBAR2 as ::core::ffi::c_int,
            TBM_SETRANGE as UINT,
            TRUE as WPARAM,
            MAKELONG!(0, 100) as LPARAM,
        );
        SendDlgItemMessage(
            hwndDlg,
            IDC_ALARMBAR2 as ::core::ffi::c_int,
            TBM_SETPOS as UINT,
            TRUE as WPARAM,
            gGPP.user.DischargePolicy[DISCHARGE_POLICY_CRITICAL as usize].BatteryLevel as LPARAM,
        );

        if LoadString(
            hApplet,
            (gGPP.user.DischargePolicy[DISCHARGE_POLICY_CRITICAL as usize].PowerPolicy.Action as ::core::ffi::c_int
                + IDS_PowerActionNone1 as ::core::ffi::c_int) as UINT,
            core::ptr::addr_of_mut!(szAction) as LPWSTR,
            _countof!(szAction) as ::core::ffi::c_int,
        ) != 0
        {
            SetDlgItemText(
                hwndDlg,
                IDC_ALARMAKTION2 as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(szAction) as *mut TCHAR as LPCWSTR,
            );
        }

        ZeroMemory!(
            core::ptr::addr_of_mut!(szMessage) as *mut core::ffi::c_void,
            core::mem::size_of::<[TCHAR; (MAX_PATH) as _]>(),
        );
        LoadString(
            hApplet,
            IDS_NOACTION as UINT,
            core::ptr::addr_of_mut!(szMessage) as LPWSTR,
            _countof!(szMessage) as ::core::ffi::c_int,
        );

        if (LOWORD!(gGPP.user.DischargePolicy[DISCHARGE_POLICY_CRITICAL as usize].PowerPolicy.EventCode) as ::core::ffi::c_int
            & POWER_LEVEL_USER_NOTIFY_TEXT as ::core::ffi::c_int) != 0
        {
            if (LOWORD!(gGPP.user.DischargePolicy[DISCHARGE_POLICY_CRITICAL as usize].PowerPolicy.EventCode) as ::core::ffi::c_int
                & POWER_LEVEL_USER_NOTIFY_SOUND as ::core::ffi::c_int) != 0
            {
                if LoadString(
                    hApplet,
                    IDS_TEXT as UINT,
                    core::ptr::addr_of_mut!(szText) as LPWSTR,
                    _countof!(szText) as ::core::ffi::c_int,
                ) != 0
                    && LoadString(
                        hApplet,
                        IDS_SOUND as UINT,
                        core::ptr::addr_of_mut!(szSound) as LPWSTR,
                        _countof!(szSound) as ::core::ffi::c_int,
                    ) != 0
                {
                    static FMT_SOUND_TEXT_B: [u16; 7] = ['%' as u16, 's' as u16, ',' as u16, ' ' as u16, '%' as u16, 's' as u16, 0];
                    _stprintf(
                        (core::ptr::addr_of_mut!(szMessage) as *mut TCHAR).cast(),
                        (core::ptr::addr_of!(FMT_SOUND_TEXT_B) as *const TCHAR).cast(),
                        core::ptr::addr_of_mut!(szSound) as *mut TCHAR,
                        core::ptr::addr_of_mut!(szText) as *mut TCHAR,
                    );
                }
            } else {
                if LoadString(
                    hApplet,
                    IDS_TEXT as UINT,
                    core::ptr::addr_of_mut!(szText) as LPWSTR,
                    _countof!(szText) as ::core::ffi::c_int,
                ) != 0
                {
                    static FMT_TEXT_B: [u16; 3] = ['%' as u16, 's' as u16, 0];
                    _stprintf(
                        (core::ptr::addr_of_mut!(szMessage) as *mut TCHAR).cast(),
                        (core::ptr::addr_of!(FMT_TEXT_B) as *const TCHAR).cast(),
                        core::ptr::addr_of_mut!(szText) as *mut TCHAR,
                    );
                }
            }
        } else {
            if (LOWORD!(gGPP.user.DischargePolicy[DISCHARGE_POLICY_CRITICAL as usize].PowerPolicy.EventCode) as ::core::ffi::c_int
                & POWER_LEVEL_USER_NOTIFY_SOUND as ::core::ffi::c_int) != 0
            {
                if LoadString(
                    hApplet,
                    IDS_SOUND as UINT,
                    core::ptr::addr_of_mut!(szSound) as LPWSTR,
                    _countof!(szSound) as ::core::ffi::c_int,
                ) != 0
                {
                    static FMT_SOUND_B: [u16; 3] = ['%' as u16, 's' as u16, 0];
                    _stprintf(
                        (core::ptr::addr_of_mut!(szMessage) as *mut TCHAR).cast(),
                        (core::ptr::addr_of!(FMT_SOUND_B) as *const TCHAR).cast(),
                        core::ptr::addr_of_mut!(szSound) as *mut TCHAR,
                    );
                }
            }
        }

        SetDlgItemText(
            hwndDlg,
            IDC_ALARMMSG2 as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(szMessage) as *mut TCHAR as LPCWSTR,
        );

        if LoadString(
            hApplet,
            IDS_PowerActionNone2 as UINT,
            core::ptr::addr_of_mut!(szProgram) as LPWSTR,
            _countof!(szProgram) as ::core::ffi::c_int,
        ) != 0
        {
            SetDlgItemText(
                hwndDlg,
                IDC_ALARMPROG2 as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(szProgram) as *mut TCHAR as LPCWSTR,
            );
        }
    }

    return TRUE as BOOLEAN;
}
#[no_mangle]
pub unsafe extern "system" fn AlarmsDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);

    match uMsg {
        WM_INITDIALOG => {
            if Ala_InitData(hwndDlg) == 0 {
                // TODO:
                // Handle initialization error
            }
            return (TRUE) as INT_PTR;
        }
        _ => {}
    }

    return (FALSE) as INT_PTR;
}
