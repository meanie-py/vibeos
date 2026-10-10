use crate::sndvol32::mixer::SndMixerGetVolumeControlDetails;
use crate::sndvol32::mixer::SndMixerQueryControls;
use crate::sndvol32::mixer::SndMixerSetVolumeControlDetails;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _ADVANCED_DATA {
    pub Context: PADVANCED_CONTEXT,
    pub bEnabled: [BOOL; 4],
    pub dwControlId: [DWORD; 4],
    pub dwMaximum: [DWORD; 2],
    pub dwMinimum: [DWORD; 2],
}

pub type ADVANCED_DATA = _ADVANCED_DATA;
pub type PADVANCED_DATA = *mut _ADVANCED_DATA;

unsafe extern "C" fn OnInitDialog(hwndDlg: HWND, pData: PADVANCED_DATA) {
    let mut UnsignedDetails: MIXERCONTROLDETAILS_UNSIGNED = core::mem::zeroed();
    let mut BooleanDetails: MIXERCONTROLDETAILS_BOOLEAN = core::mem::zeroed();
    let mut szRawBuffer: [WCHAR; 256] = [0; 256];
    let mut szCookedBuffer: [WCHAR; 256] = [0; 256];
    let mut Control: LPMIXERCONTROL = core::ptr::null_mut();
    let mut ControlCount: UINT = 0 as UINT;
    let mut Index: UINT = 0;
    let mut i: DWORD = 0;
    let mut dwStep: DWORD = 0;
    let mut dwPosition: DWORD = 0;
    let mut dwOtherControls: DWORD = 0 as DWORD;
    let mut rect: RECT = core::mem::zeroed();
    let mut dy: LONG = 0;

    /* Set the dialog title */
    LoadStringW(hAppInstance, IDS_ADVANCED_CONTROLS as UINT, szRawBuffer.as_mut_ptr(), ARRAYSIZE!(szRawBuffer) as ::core::ffi::c_int);
    StringCchPrintfW!(szCookedBuffer.as_mut_ptr(), ARRAYSIZE!(szCookedBuffer) as size_t, szRawBuffer.as_mut_ptr() as *mut WCHAR as STRSAFE_LPCWSTR, core::ptr::addr_of_mut!((*(*pData).Context).LineName) as *mut WCHAR);
    SetWindowTextW(hwndDlg, szCookedBuffer.as_mut_ptr() as LPCWSTR);

    /* Disable the tone controls */
    i = IDC_ADV_BASS_LOW as DWORD;
    while i <= IDC_ADV_TREBLE_SLIDER as DWORD {
        EnableWindow(GetDlgItem(hwndDlg, i as ::core::ffi::c_int), (FALSE) as BOOL);
        i = i.wrapping_add(1);
    }

    /* Initialize the bass and treble trackbars */
    SendDlgItemMessageW(hwndDlg, (IDC_ADV_BASS_SLIDER as i32), TBM_SETRANGE as UINT, TRUE as WPARAM, MAKELONG!(VOLUME_MIN, VOLUME_MAX) as LPARAM);
    SendDlgItemMessageW(hwndDlg, (IDC_ADV_TREBLE_SLIDER as i32), TBM_SETRANGE as UINT, TRUE as WPARAM, MAKELONG!(VOLUME_MIN, VOLUME_MAX) as LPARAM);
    SendDlgItemMessageW(hwndDlg, (IDC_ADV_BASS_SLIDER as i32), TBM_SETPAGESIZE as UINT, 0 as WPARAM, VOLUME_PAGE_SIZE as LPARAM);
    SendDlgItemMessageW(hwndDlg, (IDC_ADV_TREBLE_SLIDER as i32), TBM_SETPAGESIZE as UINT, 0 as WPARAM, VOLUME_PAGE_SIZE as LPARAM);
    SendDlgItemMessageW(hwndDlg, (IDC_ADV_BASS_SLIDER as i32), TBM_SETPOS as UINT, TRUE as WPARAM, 0 as LPARAM);
    SendDlgItemMessageW(hwndDlg, (IDC_ADV_TREBLE_SLIDER as i32), TBM_SETPOS as UINT, TRUE as WPARAM, 0 as LPARAM);

    /* Calculate and set ticks */
    dwStep = (VOLUME_MAX / (VOLUME_TICKS + 1)) as DWORD;
    if VOLUME_MAX % (VOLUME_TICKS + 1) != 0 {
        dwStep = dwStep.wrapping_add(1);
    }
    i = dwStep;
    while i < VOLUME_MAX as DWORD {
        SendDlgItemMessageW(hwndDlg, (IDC_ADV_BASS_SLIDER as i32), TBM_SETTIC as UINT, 0 as WPARAM, i as LPARAM);
        SendDlgItemMessageW(hwndDlg, (IDC_ADV_TREBLE_SLIDER as i32), TBM_SETTIC as UINT, 0 as WPARAM, i as LPARAM);
        i = i.wrapping_add(dwStep);
    }

    /* Hide the other controls */
    i = IDC_ADV_OTHER_CONTROLS as DWORD;
    while i <= IDC_ADV_OTHER_CHECK2 as DWORD {
        ShowWindow(GetDlgItem(hwndDlg, i as ::core::ffi::c_int), (SW_HIDE) as i32);
        i = i.wrapping_add(1);
    }

    if SndMixerQueryControls((*(*pData).Context).Mixer, core::ptr::addr_of_mut!(ControlCount), (*(*pData).Context).Line, core::ptr::addr_of_mut!(Control)) != 0 {
        Index = 0 as UINT;
        while Index < ControlCount {
            if (*Control.offset(Index as isize)).dwControlType == MIXERCONTROL_CONTROLTYPE_BASS as DWORD {
                /* Bass control */

                if SndMixerGetVolumeControlDetails((*(*pData).Context).Mixer, (*Control.offset(Index as isize)).dwControlID, 1 as DWORD, core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD, core::ptr::addr_of_mut!(UnsignedDetails) as LPVOID) != -(1 as ::core::ffi::c_int) {
                    i = IDC_ADV_BASS_LOW as DWORD;
                    while i <= IDC_ADV_BASS_SLIDER as DWORD {
                        EnableWindow(GetDlgItem(hwndDlg, i as ::core::ffi::c_int), (TRUE) as BOOL);
                        i = i.wrapping_add(1);
                    }

                    dwStep = (*Control.offset(Index as isize)).Bounds.__bindgen_anon_2.dwMaximum.wrapping_sub((*Control.offset(Index as isize)).Bounds.__bindgen_anon_2.dwMinimum).wrapping_div((VOLUME_MAX - VOLUME_MIN) as DWORD);
                    dwPosition = UnsignedDetails.dwValue.wrapping_sub((*Control.offset(Index as isize)).Bounds.__bindgen_anon_2.dwMinimum).wrapping_div(dwStep);
                    SendDlgItemMessageW(hwndDlg, (IDC_ADV_BASS_SLIDER as i32), TBM_SETPOS as UINT, TRUE as WPARAM, dwPosition as LPARAM);

                    (*pData).bEnabled[0 as usize] = TRUE as BOOL;
                    (*pData).dwControlId[0 as usize] = (*Control.offset(Index as isize)).dwControlID;
                    (*pData).dwMaximum[0 as usize] = (*Control.offset(Index as isize)).Bounds.__bindgen_anon_2.dwMaximum;
                    (*pData).dwMinimum[0 as usize] = (*Control.offset(Index as isize)).Bounds.__bindgen_anon_2.dwMinimum;
                }
            } else if (*Control.offset(Index as isize)).dwControlType == MIXERCONTROL_CONTROLTYPE_TREBLE as DWORD {
                /* Treble control */

                if SndMixerGetVolumeControlDetails((*(*pData).Context).Mixer, (*Control.offset(Index as isize)).dwControlID, 1 as DWORD, core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD, core::ptr::addr_of_mut!(UnsignedDetails) as LPVOID) != -(1 as ::core::ffi::c_int) {
                    i = IDC_ADV_TREBLE_LOW as DWORD;
                    while i <= IDC_ADV_TREBLE_SLIDER as DWORD {
                        EnableWindow(GetDlgItem(hwndDlg, i as ::core::ffi::c_int), (TRUE) as BOOL);
                        i = i.wrapping_add(1);
                    }

                    dwStep = (*Control.offset(Index as isize)).Bounds.__bindgen_anon_2.dwMaximum.wrapping_sub((*Control.offset(Index as isize)).Bounds.__bindgen_anon_2.dwMinimum).wrapping_div((VOLUME_MAX - VOLUME_MIN) as DWORD);
                    dwPosition = UnsignedDetails.dwValue.wrapping_sub((*Control.offset(Index as isize)).Bounds.__bindgen_anon_2.dwMinimum).wrapping_div(dwStep);
                    SendDlgItemMessageW(hwndDlg, (IDC_ADV_TREBLE_SLIDER as i32), TBM_SETPOS as UINT, TRUE as WPARAM, dwPosition as LPARAM);

                    (*pData).bEnabled[1 as usize] = TRUE as BOOL;
                    (*pData).dwControlId[1 as usize] = (*Control.offset(Index as isize)).dwControlID;
                    (*pData).dwMaximum[1 as usize] = (*Control.offset(Index as isize)).Bounds.__bindgen_anon_2.dwMaximum;
                    (*pData).dwMinimum[1 as usize] = (*Control.offset(Index as isize)).Bounds.__bindgen_anon_2.dwMinimum;
                }
            } else if ((*Control.offset(Index as isize)).dwControlType & (MIXERCONTROL_CT_CLASS_MASK | MIXERCONTROL_CT_SUBCLASS_MASK | MIXERCONTROL_CT_UNITS_MASK) as DWORD) == MIXERCONTROL_CONTROLTYPE_BOOLEAN as DWORD
                && (*Control.offset(Index as isize)).dwControlType != MIXERCONTROL_CONTROLTYPE_MUTE as DWORD
            {
                /* All boolean controls but the Mute control (Maximum of 2) */

                if dwOtherControls < 2 as DWORD {
                    if SndMixerGetVolumeControlDetails((*(*pData).Context).Mixer, (*Control.offset(Index as isize)).dwControlID, 1 as DWORD, core::mem::size_of::<MIXERCONTROLDETAILS_BOOLEAN>() as DWORD, core::ptr::addr_of_mut!(BooleanDetails) as LPVOID) != -(1 as ::core::ffi::c_int) {
                        LoadStringW(hAppInstance, (IDS_OTHER_CONTROLS1 as DWORD).wrapping_add(dwOtherControls) as UINT, szRawBuffer.as_mut_ptr(), ARRAYSIZE!(szRawBuffer) as ::core::ffi::c_int);
                        StringCchPrintfW!(szCookedBuffer.as_mut_ptr(), ARRAYSIZE!(szCookedBuffer) as size_t, szRawBuffer.as_mut_ptr() as *mut WCHAR as STRSAFE_LPCWSTR, core::ptr::addr_of_mut!((*Control.offset(Index as isize)).szName) as *mut WCHAR);
                        SetWindowTextW(GetDlgItem(hwndDlg, (IDC_ADV_OTHER_CHECK1 as DWORD).wrapping_add(dwOtherControls) as ::core::ffi::c_int), szCookedBuffer.as_mut_ptr() as *mut WCHAR as LPCWSTR);

                        ShowWindow(GetDlgItem(hwndDlg, (IDC_ADV_OTHER_CHECK1 as DWORD).wrapping_add(dwOtherControls) as ::core::ffi::c_int), (SW_SHOWNORMAL) as i32);

                        SendDlgItemMessageW(hwndDlg, (IDC_ADV_OTHER_CHECK1 as DWORD).wrapping_add(dwOtherControls) as ::core::ffi::c_int, BM_SETCHECK as UINT, BooleanDetails.fValue as WPARAM, 0 as LPARAM);

                        (*pData).bEnabled[dwOtherControls.wrapping_add(2 as DWORD) as usize] = TRUE as BOOL;
                        (*pData).dwControlId[dwOtherControls.wrapping_add(2 as DWORD) as usize] = (*Control.offset(Index as isize)).dwControlID;

                        dwOtherControls = dwOtherControls.wrapping_add(1);
                    }
                }
            }
            Index = Index.wrapping_add(1);
        }

        /* Free controls */
        HeapFree(GetProcessHeap(), 0 as DWORD, Control as PVOID);
    }

    if dwOtherControls != 0 as DWORD {
        /* Show the 'Other controls' groupbox and text */
        ShowWindow(GetDlgItem(hwndDlg, (IDC_ADV_OTHER_CONTROLS as i32)), (SW_SHOWNORMAL) as i32);
        ShowWindow(GetDlgItem(hwndDlg, (IDC_ADV_OTHER_TEXT as i32)), (SW_SHOWNORMAL) as i32);

        /* Resize the dialog */
        GetWindowRect(hwndDlg, core::ptr::addr_of_mut!(rect));

        dy = MulDiv(
            (if dwOtherControls == 1 as DWORD { 73 } else { 73 + 15 }) as ::core::ffi::c_int,
            (*(*(*pData).Context).MixerWindow).baseUnit.cy as ::core::ffi::c_int,
            8 as ::core::ffi::c_int,
        ) as LONG;
        rect.bottom += dy;

        SetWindowPos(hwndDlg, HWND_TOP, rect.left as ::core::ffi::c_int, rect.top as ::core::ffi::c_int, (rect.right - rect.left) as ::core::ffi::c_int, (rect.bottom - rect.top) as ::core::ffi::c_int, (SWP_NOMOVE | SWP_NOZORDER) as UINT);

        /* Move the 'Close' button down */
        GetWindowRect(GetDlgItem(hwndDlg, (IDOK) as i32), core::ptr::addr_of_mut!(rect));
        MapWindowPoints(HWND_DESKTOP, hwndDlg, core::ptr::addr_of_mut!(rect) as LPPOINT, 2 as UINT);

        rect.top += dy;
        rect.bottom += dy;

        SetWindowPos(GetDlgItem(hwndDlg, (IDOK) as i32), HWND_TOP, rect.left as ::core::ffi::c_int, rect.top as ::core::ffi::c_int, (rect.right - rect.left) as ::core::ffi::c_int, (rect.bottom - rect.top) as ::core::ffi::c_int, (SWP_NOSIZE | SWP_NOZORDER) as UINT);

        if dwOtherControls == 2 as DWORD {
            /* Resize the 'Other Controls' groupbox */
            GetWindowRect(GetDlgItem(hwndDlg, (IDC_ADV_OTHER_CONTROLS as i32)), core::ptr::addr_of_mut!(rect));
            MapWindowPoints(HWND_DESKTOP, hwndDlg, core::ptr::addr_of_mut!(rect) as LPPOINT, 2 as UINT);

            dy = MulDiv(15 as ::core::ffi::c_int, (*(*(*pData).Context).MixerWindow).baseUnit.cy as ::core::ffi::c_int, 8 as ::core::ffi::c_int) as LONG;
            rect.bottom += dy;

            SetWindowPos(GetDlgItem(hwndDlg, (IDC_ADV_OTHER_CONTROLS as i32)), HWND_TOP, rect.left as ::core::ffi::c_int, rect.top as ::core::ffi::c_int, (rect.right - rect.left) as ::core::ffi::c_int, (rect.bottom - rect.top) as ::core::ffi::c_int, (SWP_NOMOVE | SWP_NOZORDER) as UINT);
        }
    }
}

unsafe extern "C" fn OnHScroll(mut hwndDlg: HWND, mut pData: PADVANCED_DATA, mut dwCtrlID: DWORD) {
    let mut Details: MIXERCONTROLDETAILS_UNSIGNED = core::mem::zeroed();
    let mut dwControlID: DWORD = 0 as DWORD;
    let mut dwStep: DWORD = 0;
    let mut dwPosition: DWORD = 0;
    let mut dwMaximum: DWORD = 0;
    let mut dwMinimum: DWORD = 0;
    if dwCtrlID != IDC_ADV_BASS_SLIDER as DWORD && dwCtrlID != IDC_ADV_TREBLE_SLIDER as DWORD {
        return;
    }
    if dwCtrlID == IDC_ADV_BASS_SLIDER as DWORD {
        if ((*pData).bEnabled[0 as usize]) as u32 == FALSE {
            return;
        }
        dwControlID = (*pData).dwControlId[0 as usize];
        dwMaximum = (*pData).dwMaximum[0 as usize];
        dwMinimum = (*pData).dwMinimum[0 as usize];
    } else if dwCtrlID == IDC_ADV_TREBLE_SLIDER as DWORD {
        if ((*pData).bEnabled[1 as usize]) as u32 == FALSE {
            return;
        }
        dwControlID = (*pData).dwControlId[1 as usize];
        dwMaximum = (*pData).dwMaximum[1 as usize];
        dwMinimum = (*pData).dwMinimum[1 as usize];
    }
    dwPosition = SendDlgItemMessage(
        hwndDlg,
        dwCtrlID as ::core::ffi::c_int,
        TBM_GETPOS as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as DWORD;
    dwStep = dwMaximum
        .wrapping_sub(dwMinimum)
        .wrapping_div((VOLUME_MAX - VOLUME_MIN) as DWORD);
    Details.dwValue = dwPosition.wrapping_mul(dwStep).wrapping_add(dwMinimum);
    SndMixerSetVolumeControlDetails(
        (*(*pData).Context).Mixer,
        dwControlID,
        1 as DWORD,
        core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD,
        core::ptr::addr_of_mut!(Details) as LPVOID,
    );
}
#[no_mangle]
pub unsafe extern "system" fn AdvancedDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pData: PADVANCED_DATA = core::ptr::null_mut::<ADVANCED_DATA>();

    pData = GetWindowLongPtr(hwndDlg, DWLP_USER as ::core::ffi::c_int) as PADVANCED_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                core::mem::size_of::<ADVANCED_DATA>() as SIZE_T,
            ) as PADVANCED_DATA;
            if !pData.is_null() {
                SetWindowLongPtr(hwndDlg, DWLP_USER as ::core::ffi::c_int, pData as LONG_PTR);
                (*pData).Context = lParam as PADVANCED_CONTEXT;
                OnInitDialog(hwndDlg, pData);
            }
            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDOK => {
                    EndDialog(hwndDlg, IDOK as INT_PTR);
                }
                _ => {}
            }
        }

        WM_HSCROLL => {
            if !pData.is_null() {
                if LOWORD!(wParam) as ::core::ffi::c_int == (TB_THUMBTRACK as i32) {
                    OnHScroll(
                        hwndDlg,
                        pData,
                        GetDlgCtrlID(lParam as HWND) as DWORD,
                    );
                }
            }
        }

        WM_CLOSE => {
            EndDialog(hwndDlg, IDCANCEL as INT_PTR);
        }

        WM_DESTROY => {
            if !pData.is_null() {
                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    pData as PVOID,
                );
                SetWindowLongPtr(hwndDlg, DWLP_USER as ::core::ffi::c_int, core::ptr::null_mut::<core::ffi::c_void>() as LONG_PTR);
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
