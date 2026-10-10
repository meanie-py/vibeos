use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

pub type PDIALOG_DATA = *mut DIALOG_DATA;

unsafe extern "C" fn OnTrayInitDialog(hwnd: HWND, wParam: WPARAM, lParam: LPARAM) {
    let mut ptCursor: POINT = core::mem::zeroed();
    let mut rcWindow: RECT = core::mem::zeroed();
    let mut rcScreen: RECT = core::mem::zeroed();
    let mut x: LONG = 0;
    let mut y: LONG = 0;
    let mut cx: LONG = 0;
    let mut cy: LONG = 0;

    GetCursorPos(core::ptr::addr_of_mut!(ptCursor));

    GetWindowRect(hwnd, core::ptr::addr_of_mut!(rcWindow));

    GetWindowRect(GetDesktopWindow(), core::ptr::addr_of_mut!(rcScreen));

    cx = rcWindow.right - rcWindow.left;
    cy = rcWindow.bottom - rcWindow.top;

    if ptCursor.y + cy > rcScreen.bottom {
        y = ptCursor.y - cy;
    } else {
        y = ptCursor.y;
    }

    if ptCursor.x + cx > rcScreen.right {
        x = ptCursor.x - cx;
    } else {
        x = ptCursor.x;
    }

    SetWindowPos(hwnd, HWND_TOPMOST, x, y, 0, 0, SWP_NOSIZE);
}

unsafe extern "C" fn OnTrayInitMixer(pDialogData: PDIALOG_DATA, hwndDlg: HWND) {
    let mut mxln: MIXERLINE = core::mem::zeroed();
    let mut mxc: MIXERCONTROL = core::mem::zeroed();
    let mut mxlctrl: MIXERLINECONTROLS = core::mem::zeroed();
    let mut mxcd: MIXERCONTROLDETAILS = core::mem::zeroed();
    let mut mxcdBool: MIXERCONTROLDETAILS_BOOLEAN = core::mem::zeroed();
    let mut i: DWORD = 0;

    /* Open the mixer */
    if mixerOpen(
        &raw mut (*pDialogData).hMixer,
        0 as UINT,
        PtrToUlong!(hwndDlg as *const core::ffi::c_void) as DWORD_PTR,
        0 as DWORD_PTR,
        (MIXER_OBJECTF_MIXER | CALLBACK_WINDOW) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }

    /* Retrieve the mixer information */
    mxln.cbStruct = core::mem::size_of::<MIXERLINE>() as DWORD;
    mxln.dwComponentType = MIXERLINE_COMPONENTTYPE_DST_SPEAKERS as DWORD;

    if mixerGetLineInfo(
        (*pDialogData).hMixer as HMIXEROBJ,
        &raw mut mxln,
        (MIXER_OBJECTF_HMIXER | MIXER_GETLINEINFOF_COMPONENTTYPE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }

    (*pDialogData).volumeChannels = mxln.cChannels;

    /* Retrieve the line information */
    mxlctrl.cbStruct = core::mem::size_of::<MIXERLINECONTROLS>() as DWORD;
    mxlctrl.dwLineID = mxln.dwLineID;
    mxlctrl.__bindgen_anon_1.dwControlType = MIXERCONTROL_CONTROLTYPE_VOLUME as DWORD;
    mxlctrl.cControls = 1 as DWORD;
    mxlctrl.cbmxctrl = core::mem::size_of::<MIXERCONTROL>() as DWORD;
    mxlctrl.pamxctrl = &raw mut mxc;

    if mixerGetLineControls(
        (*pDialogData).hMixer as HMIXEROBJ,
        &raw mut mxlctrl,
        (MIXER_OBJECTF_HMIXER | MIXER_GETLINECONTROLSF_ONEBYTYPE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }

    (*pDialogData).volumeControlID = mxc.dwControlID;
    (*pDialogData).volumeMinimum = mxc.Bounds.__bindgen_anon_2.dwMinimum;
    (*pDialogData).volumeMaximum = mxc.Bounds.__bindgen_anon_2.dwMaximum;
    (*pDialogData).volumeStep = (*pDialogData)
        .volumeMaximum
        .wrapping_sub((*pDialogData).volumeMinimum)
        .wrapping_div((VOLUME_MAX - VOLUME_MIN) as DWORD);

    /* Allocate a buffer for all channel volume values */
    (*pDialogData).volumeInitValues = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (mxln.cChannels as SIZE_T)
            .wrapping_mul(core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as SIZE_T),
    ) as PMIXERCONTROLDETAILS_UNSIGNED;
    if (*pDialogData).volumeInitValues.is_null() {
        return;
    }

    (*pDialogData).volumeCurrentValues = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (mxln.cChannels as SIZE_T)
            .wrapping_mul(core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as SIZE_T),
    ) as PMIXERCONTROLDETAILS_UNSIGNED;
    if (*pDialogData).volumeCurrentValues.is_null() {
        return;
    }

    /* Retrieve the channel volume values */
    mxcd.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
    mxcd.dwControlID = mxc.dwControlID;
    mxcd.cChannels = mxln.cChannels;
    mxcd.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
    mxcd.cbDetails = core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD;
    mxcd.paDetails = (*pDialogData).volumeInitValues as PVOID;

    if mixerGetControlDetails(
        (*pDialogData).hMixer as HMIXEROBJ,
        &raw mut mxcd,
        (MIXER_OBJECTF_HMIXER | MIXER_GETCONTROLDETAILSF_VALUE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }

    (*pDialogData).maxVolume = 0 as DWORD;
    i = 0 as DWORD;
    while i < (*pDialogData).volumeChannels {
        (*(*pDialogData).volumeCurrentValues.offset(i as isize)).dwValue =
            (*(*pDialogData).volumeInitValues.offset(i as isize)).dwValue;

        if (*(*pDialogData).volumeInitValues.offset(i as isize)).dwValue > (*pDialogData).maxVolume {
            (*pDialogData).maxVolume =
                (*(*pDialogData).volumeInitValues.offset(i as isize)).dwValue;
        }
        i = i.wrapping_add(1);
    }

    /* Initialize the volume trackbar */
    SendDlgItemMessage(
        hwndDlg,
        IDC_LINE_SLIDER_VERT as i32,
        TBM_SETRANGE as UINT,
        TRUE as WPARAM,
        MAKELONG!(VOLUME_MIN, VOLUME_MAX) as LPARAM,
    );
    SendDlgItemMessage(
        hwndDlg,
        IDC_LINE_SLIDER_VERT as i32,
        TBM_SETPAGESIZE as UINT,
        0 as WPARAM,
        VOLUME_PAGE_SIZE as LPARAM,
    );
    SendDlgItemMessage(
        hwndDlg,
        IDC_LINE_SLIDER_VERT as i32,
        TBM_SETPOS as UINT,
        TRUE as WPARAM,
        (VOLUME_MAX as DWORD).wrapping_sub(
            (*pDialogData)
                .maxVolume
                .wrapping_sub((*pDialogData).volumeMinimum)
                .wrapping_div((*pDialogData).volumeStep),
        ) as LPARAM,
    );

    /* Retrieve the mute control information */
    mxlctrl.cbStruct = core::mem::size_of::<MIXERLINECONTROLS>() as DWORD;
    mxlctrl.dwLineID = mxln.dwLineID;
    mxlctrl.__bindgen_anon_1.dwControlType = MIXERCONTROL_CONTROLTYPE_MUTE as DWORD;
    mxlctrl.cControls = 1 as DWORD;
    mxlctrl.cbmxctrl = core::mem::size_of::<MIXERCONTROL>() as DWORD;
    mxlctrl.pamxctrl = &raw mut mxc;

    if mixerGetLineControls(
        (*pDialogData).hMixer as HMIXEROBJ,
        &raw mut mxlctrl,
        (MIXER_OBJECTF_HMIXER | MIXER_GETLINECONTROLSF_ONEBYTYPE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }

    (*pDialogData).muteControlID = mxc.dwControlID;

    /* Retrieve the mute value */
    mxcd.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
    mxcd.dwControlID = mxc.dwControlID;
    mxcd.cChannels = 1 as DWORD;
    mxcd.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
    mxcd.cbDetails = core::mem::size_of::<MIXERCONTROLDETAILS_BOOLEAN>() as DWORD;
    mxcd.paDetails = &raw mut mxcdBool as PVOID;

    if mixerGetControlDetails(
        (*pDialogData).hMixer as HMIXEROBJ,
        &raw mut mxcd,
        (MIXER_OBJECTF_HMIXER | MIXER_GETCONTROLDETAILSF_VALUE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }

    /* Initialize the mute checkbox */
    SendDlgItemMessage(
        hwndDlg,
        IDC_LINE_SWITCH as i32,
        BM_SETCHECK as UINT,
        (if mxcdBool.fValue != 0 {
            BST_CHECKED
        } else {
            BST_UNCHECKED
        }) as WPARAM,
        0 as LPARAM,
    );
}

unsafe extern "C" fn OnCommand(
    mut pDialogData: PDIALOG_DATA,
    mut hwndDlg: HWND,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) {
    let mut mxcd: MIXERCONTROLDETAILS = core::mem::zeroed();
    let mut mxcdMute: MIXERCONTROLDETAILS_BOOLEAN = core::mem::zeroed();
    if LOWORD!(wParam) as ::core::ffi::c_int == (IDC_LINE_SWITCH as i32)
        && (HIWORD!(wParam) as ::core::ffi::c_int) as u32 == BN_CLICKED
    {
        mxcd.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
        mxcd.dwControlID = (*pDialogData).muteControlID;
        mxcd.cChannels = 1 as DWORD;
        mxcd.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
        mxcd.cbDetails = core::mem::size_of::<MIXERCONTROLDETAILS_BOOLEAN>() as DWORD;
        mxcd.paDetails = core::ptr::addr_of_mut!(mxcdMute) as PVOID;

        mxcdMute.fValue = (SendMessage(
            lParam as HWND,
            BM_GETCHECK as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) == BST_CHECKED as LRESULT) as ::core::ffi::c_int as LONG;

        mixerSetControlDetails(
            (*pDialogData).hMixer as HMIXEROBJ,
            core::ptr::addr_of_mut!(mxcd),
            (MIXER_OBJECTF_HMIXER as ::core::ffi::c_uint
                | MIXER_SETCONTROLDETAILSF_VALUE as ::core::ffi::c_uint) as DWORD,
        );
    }
}

unsafe extern "C" fn OnVScroll(
    pDialogData: PDIALOG_DATA,
    hwndDlg: HWND,
    wParam: WPARAM,
    lParam: LPARAM,
) {
    let mut mxcd: MIXERCONTROLDETAILS = core::mem::zeroed();
    let mut dwPosition: DWORD = 0;
    let mut dwVolume: DWORD = 0;
    let mut i: DWORD = 0;

    match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
        TB_THUMBPOSITION => {}
        TB_ENDTRACK => {
            PlaySound(
                SND_ALIAS_SYSTEMDEFAULT as LPCTSTR,
                core::ptr::null_mut(),
                (SND_ASYNC | SND_ALIAS_ID) as DWORD,
            );
        }
        _ => {
            dwPosition = (VOLUME_MAX as DWORD).wrapping_sub(SendDlgItemMessage(
                hwndDlg,
                (IDC_LINE_SLIDER_VERT as UINT) as i32,
                TBM_GETPOS as UINT,
                0 as WPARAM,
                0 as LPARAM,
            ) as DWORD);

            if dwPosition == VOLUME_MIN as DWORD {
                dwVolume = (*pDialogData).volumeMinimum;
            } else if dwPosition == VOLUME_MAX as DWORD {
                dwVolume = (*pDialogData).volumeMaximum;
            } else {
                dwVolume = dwPosition
                    .wrapping_mul((*pDialogData).volumeStep)
                    .wrapping_add((*pDialogData).volumeMinimum);
            }

            i = 0 as DWORD;
            while i < (*pDialogData).volumeChannels {
                if (*(*pDialogData).volumeInitValues.offset(i as isize)).dwValue
                    == (*pDialogData).maxVolume
                {
                    (*(*pDialogData).volumeCurrentValues.offset(i as isize)).dwValue = dwVolume;
                } else {
                    (*(*pDialogData).volumeCurrentValues.offset(i as isize)).dwValue =
                        (*(*pDialogData).volumeInitValues.offset(i as isize))
                            .dwValue
                            .wrapping_mul(dwVolume)
                            .wrapping_div((*pDialogData).maxVolume);
                }
                i = i.wrapping_add(1);
            }

            mxcd.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
            mxcd.dwControlID = (*pDialogData).volumeControlID;
            mxcd.cChannels = (*pDialogData).volumeChannels;
            mxcd.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
            mxcd.cbDetails = core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD;
            mxcd.paDetails = (*pDialogData).volumeCurrentValues as PVOID;

            mixerSetControlDetails(
                (*pDialogData).hMixer as HMIXEROBJ,
                &raw mut mxcd,
                (MIXER_OBJECTF_HMIXER | MIXER_SETCONTROLDETAILSF_VALUE) as DWORD,
            );
        }
    }
}

unsafe extern "C" fn OnMixerControlChange(
    pDialogData: PDIALOG_DATA,
    hwndDlg: HWND,
    wParam: WPARAM,
    lParam: LPARAM,
) {
    let hMixer: HMIXER = wParam as HMIXER;
    let dwControlID: DWORD = lParam as DWORD;

    if hMixer != (*pDialogData).hMixer {
        return;
    }

    let mut mxcd: MIXERCONTROLDETAILS = core::mem::zeroed();
    mxcd.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
    mxcd.dwControlID = dwControlID;
    mxcd.__bindgen_anon_1.cMultipleItems = 0 as DWORD;

    if dwControlID == (*pDialogData).muteControlID {
        let mut mxcdBool: MIXERCONTROLDETAILS_BOOLEAN = core::mem::zeroed();

        mxcd.cChannels = 1 as DWORD;
        mxcd.cbDetails = core::mem::size_of::<MIXERCONTROLDETAILS_BOOLEAN>() as DWORD;
        mxcd.paDetails = core::ptr::addr_of_mut!(mxcdBool) as PVOID;

        if mixerGetControlDetails(
            hMixer as HMIXEROBJ,
            core::ptr::addr_of_mut!(mxcd),
            (MIXER_OBJECTF_HMIXER | MIXER_GETCONTROLDETAILSF_VALUE) as DWORD,
        ) == MMSYSERR_NOERROR as MMRESULT
        {
            SendDlgItemMessageW(
                hwndDlg,
                IDC_LINE_SWITCH as core::ffi::c_int,
                BM_SETCHECK as UINT,
                (if mxcdBool.fValue != 0 {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as WPARAM,
                0 as LPARAM,
            );
        }
    } else if dwControlID == (*pDialogData).volumeControlID {
        let mut mxcdVolume: PMIXERCONTROLDETAILS_UNSIGNED =
            core::ptr::null_mut::<tMIXERCONTROLDETAILS_UNSIGNED>();

        mxcdVolume = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            ((*pDialogData).volumeChannels as SIZE_T)
                .wrapping_mul(core::mem::size_of::<tMIXERCONTROLDETAILS_UNSIGNED>() as SIZE_T),
        ) as PMIXERCONTROLDETAILS_UNSIGNED;
        if mxcdVolume.is_null() {
            return;
        }

        mxcd.cChannels = (*pDialogData).volumeChannels;
        mxcd.cbDetails = core::mem::size_of::<tMIXERCONTROLDETAILS_UNSIGNED>() as DWORD;
        mxcd.paDetails = mxcdVolume as PVOID;

        if mixerGetControlDetails(
            hMixer as HMIXEROBJ,
            core::ptr::addr_of_mut!(mxcd),
            (MIXER_OBJECTF_HMIXER | MIXER_GETCONTROLDETAILSF_VALUE) as DWORD,
        ) != MMSYSERR_NOERROR as MMRESULT
        {
            return;
        }

        /* The Volume trackbar always shows the maximum volume value of all channels.
         * Changing volume balance in "full" sndvol32 view must not affect it. */
        let mut dwVolume: DWORD = 0 as DWORD;
        let mut i: core::ffi::c_int = 0 as core::ffi::c_int;
        while (i as DWORD) < (*pDialogData).volumeChannels {
            if !((*mxcdVolume.offset(i as isize)).dwValue <= dwVolume) {
                dwVolume = (*mxcdVolume.offset(i as isize)).dwValue;
            }
            i += 1;
        }

        let dwNewPosition: DWORD = (VOLUME_MAX as DWORD)
            .wrapping_sub(dwVolume.wrapping_div((*pDialogData).volumeStep));
        SendDlgItemMessageW(
            hwndDlg,
            IDC_LINE_SLIDER_VERT as core::ffi::c_int,
            TBM_SETPOS as UINT,
            TRUE as WPARAM,
            dwNewPosition as LPARAM,
        );

        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            mxcdVolume as PVOID,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn TrayDlgProc(mut hwndDlg: HWND, mut uMsg: UINT, mut wParam: WPARAM, mut lParam: LPARAM) -> INT_PTR {
    let mut pDialogData: PDIALOG_DATA = ::core::ptr::null_mut::<_DIALOG_DATA>();

    pDialogData = GetWindowLongPtr(hwndDlg, DWLP_USER as ::core::ffi::c_int) as PDIALOG_DATA;

    match uMsg {
        WM_INITDIALOG => {
            OnTrayInitDialog(hwndDlg, wParam, lParam);

            pDialogData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                core::mem::size_of::<DIALOG_DATA>() as SIZE_T,
            ) as PDIALOG_DATA;
            SetWindowLongPtr(
                hwndDlg,
                DWLP_USER as ::core::ffi::c_int,
                pDialogData as LONG_PTR,
            );

            if !pDialogData.is_null() {
                OnTrayInitMixer(pDialogData, hwndDlg);
            }
        }
        WM_COMMAND => {
            if !pDialogData.is_null() {
                OnCommand(pDialogData, hwndDlg, wParam, lParam);
            }
        }
        WM_VSCROLL => {
            if !pDialogData.is_null() {
                OnVScroll(pDialogData, hwndDlg, wParam, lParam);
            }
        }
        WM_DESTROY => {
            if !pDialogData.is_null() {
                if !(*pDialogData).volumeInitValues.is_null() {
                    HeapFree(
                        GetProcessHeap(),
                        0 as DWORD,
                        (*pDialogData).volumeInitValues as PVOID,
                    );
                }

                if !(*pDialogData).volumeCurrentValues.is_null() {
                    HeapFree(
                        GetProcessHeap(),
                        0 as DWORD,
                        (*pDialogData).volumeCurrentValues as PVOID,
                    );
                }

                if !(*pDialogData).hMixer.is_null() {
                    mixerClose((*pDialogData).hMixer);
                }

                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    pDialogData as PVOID,
                );
                pDialogData = ::core::ptr::null_mut::<_DIALOG_DATA>();
                SetWindowLongPtr(
                    hwndDlg,
                    DWLP_USER as ::core::ffi::c_int,
                    NULL as LONG_PTR,
                );
            }
        }
        WM_ACTIVATE => {
            if (LOWORD!(wParam) as ::core::ffi::c_int) as u32 == WA_INACTIVE {
                EndDialog(hwndDlg, IDOK as INT_PTR);
            }
        }
        MM_MIXM_CONTROL_CHANGE => {
            if !pDialogData.is_null() {
                OnMixerControlChange(pDialogData, hwndDlg, wParam, lParam);
            }
        }
        _ => {}
    }

    return 0 as INT_PTR;
}
