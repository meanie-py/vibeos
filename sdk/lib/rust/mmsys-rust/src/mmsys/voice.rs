use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn VoiceDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);
    if uMsg == WM_INITDIALOG {
        let mut NumWavOut: UINT;

        NumWavOut = waveOutGetNumDevs();
        if NumWavOut == 0 {
            EnableWindow(GetDlgItem(hwndDlg, (IDC_DEVICE_VOICE_LIST as i32)), (FALSE) as BOOL);
            EnableWindow(GetDlgItem(hwndDlg, (IDC_DEVICE_VOICE_REC_LIST as i32)), (FALSE) as BOOL);
            EnableWindow(GetDlgItem(hwndDlg, (IDC_VOLUME4_BTN as i32)), (FALSE) as BOOL);
            EnableWindow(GetDlgItem(hwndDlg, (IDC_ADV4_BTN as i32)), (FALSE) as BOOL);
            EnableWindow(GetDlgItem(hwndDlg, (IDC_VOLUME5_BTN as i32)), (FALSE) as BOOL);
            EnableWindow(GetDlgItem(hwndDlg, (IDC_ADV5_BTN as i32)), (FALSE) as BOOL);
            EnableWindow(GetDlgItem(hwndDlg, (IDC_TEST_HARDWARE as i32)), (FALSE) as BOOL);
        }
    }

    return FALSE as INT_PTR;
}
