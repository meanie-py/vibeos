use crate::mmsys::speakervolume::SpeakerVolume;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_2B9D6BDC2386: [u16; 24] = [
    0x0043u16, 0x0061, 0x006E, 0x006E, 0x006F, 0x0074, 0x0020, 0x0072, 0x0075, 0x006E,
    0x0020, 0x0073, 0x006E, 0x0064, 0x0076, 0x006F, 0x006C, 0x0033, 0x0032, 0x002E,
    0x0065, 0x0078, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_566AB50D80A9: [u16; 13] = [
    0x0073u16, 0x006E, 0x0064, 0x0076, 0x006F, 0x006C, 0x0033, 0x0032, 0x002E, 0x0065,
    0x0078, 0x0065, 0x0000,
];
pub static RUSTCONV_WSTR_6A661B0A8BF4: [u16; 16] = [
    0x0053u16, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0054, 0x0072, 0x0061, 0x0079,
    0x005F, 0x004D, 0x0061, 0x0069, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_82DB442E7A60: [u16; 5] = [0x006Fu16, 0x0070, 0x0065, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_B80563C54935: [u16; 23] = [
    0x006Du16, 0x0069, 0x0078, 0x0065, 0x0072, 0x0047, 0x0065, 0x0074, 0x0044, 0x0065,
    0x0076, 0x0043, 0x0061, 0x0070, 0x0073, 0x0020, 0x0066, 0x0061, 0x0069, 0x006C,
    0x0065, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_C4C3B6321C70: [u16; 18] = [
    0x0043u16, 0x0061, 0x006E, 0x006E, 0x006F, 0x0074, 0x0020, 0x006F, 0x0070, 0x0065,
    0x006E, 0x0020, 0x006D, 0x0069, 0x0078, 0x0065, 0x0072, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct GLOBAL_DATA {
    pub hMixer: HMIXER,
    pub hIconMuted: HICON,
    pub hIconUnMuted: HICON,
    pub hIconNoHW: HICON,
    pub hIconSpeakImg: HICON,

    pub muteVal: LONG,
    pub muteControlID: DWORD,

    pub volumeControlID: DWORD,
    pub volumeChannels: DWORD,
    pub volumeMinimum: DWORD,
    pub volumeMaximum: DWORD,
    pub volumeStep: DWORD,

    pub maxVolume: DWORD,
    pub volumeInitialValues: PMIXERCONTROLDETAILS_UNSIGNED,
    pub volumePreviousValues: PMIXERCONTROLDETAILS_UNSIGNED,
    pub volumeCurrentValues: PMIXERCONTROLDETAILS_UNSIGNED,
}

pub type PGLOBAL_DATA = *mut GLOBAL_DATA;
#[no_mangle]
pub unsafe extern "C" fn GetMuteControl(mut pGlobalData: PGLOBAL_DATA) {
    let mut mxln: MIXERLINEW = core::mem::zeroed();
    let mut mxc: MIXERCONTROLW = core::mem::zeroed();
    let mut mxlctrl: MIXERLINECONTROLSW = core::mem::zeroed();

    if (*pGlobalData).hMixer.is_null() {
        return;
    }

    mxln.cbStruct = core::mem::size_of::<MIXERLINEW>() as DWORD;
    mxln.dwComponentType = MIXERLINE_COMPONENTTYPE_DST_SPEAKERS as DWORD;

    if mixerGetLineInfoW(
        (*pGlobalData).hMixer as HMIXEROBJ,
        core::ptr::addr_of_mut!(mxln),
        (MIXER_OBJECTF_HMIXER | MIXER_GETLINEINFOF_COMPONENTTYPE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }

    mxlctrl.cbStruct = core::mem::size_of::<MIXERLINECONTROLSW>() as DWORD;
    mxlctrl.dwLineID = mxln.dwLineID;
    mxlctrl.__bindgen_anon_1.dwControlType = MIXERCONTROL_CONTROLTYPE_MUTE as DWORD;
    mxlctrl.cControls = 1 as DWORD;
    mxlctrl.cbmxctrl = core::mem::size_of::<MIXERCONTROLW>() as DWORD;
    mxlctrl.pamxctrl = core::ptr::addr_of_mut!(mxc) as LPMIXERCONTROLW;

    if mixerGetLineControlsW(
        (*pGlobalData).hMixer as HMIXEROBJ,
        core::ptr::addr_of_mut!(mxlctrl),
        (MIXER_OBJECTF_HMIXER | MIXER_GETLINECONTROLSF_ONEBYTYPE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }

    (*pGlobalData).muteControlID = mxc.dwControlID;
}
#[no_mangle]
pub unsafe extern "C" fn GetMuteState(pGlobalData: PGLOBAL_DATA) {
    let mut mxcdMute: MIXERCONTROLDETAILS_BOOLEAN = MIXERCONTROLDETAILS_BOOLEAN { fValue: 0 };
    let mut mxcd: MIXERCONTROLDETAILS = MIXERCONTROLDETAILS {
        cbStruct: 0,
        dwControlID: 0,
        cChannels: 0,
        __bindgen_anon_1: tMIXERCONTROLDETAILS__bindgen_ty_1 {
            hwndOwner: core::ptr::null_mut::<HWND__>(),
        },
        cbDetails: 0,
        paDetails: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    if (*pGlobalData).hMixer.is_null() {
        return;
    }
    mxcd.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
    mxcd.dwControlID = (*pGlobalData).muteControlID;
    mxcd.cChannels = 1 as DWORD;
    mxcd.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
    mxcd.cbDetails = core::mem::size_of::<MIXERCONTROLDETAILS_BOOLEAN>() as DWORD;
    mxcd.paDetails = core::ptr::addr_of_mut!(mxcdMute) as PVOID;
    if mixerGetControlDetailsW(
        (*pGlobalData).hMixer as HMIXEROBJ,
        core::ptr::addr_of_mut!(mxcd),
        (MIXER_OBJECTF_HMIXER | MIXER_GETCONTROLDETAILSF_VALUE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }
    (*pGlobalData).muteVal = mxcdMute.fValue;
}
#[no_mangle]
pub unsafe extern "C" fn SwitchMuteState(pGlobalData: PGLOBAL_DATA) {
    let mut mxcdMute: MIXERCONTROLDETAILS_BOOLEAN = core::mem::zeroed();
    let mut mxcd: MIXERCONTROLDETAILS = core::mem::zeroed();

    mxcd.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
    mxcd.dwControlID = (*pGlobalData).muteControlID;
    mxcd.cChannels = 1 as DWORD;
    mxcd.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
    mxcd.cbDetails = core::mem::size_of::<MIXERCONTROLDETAILS_BOOLEAN>() as DWORD;
    mxcd.paDetails = core::ptr::addr_of_mut!(mxcdMute) as PVOID;

    mxcdMute.fValue = ((*pGlobalData).muteVal == 0) as LONG;
    if mixerSetControlDetails(
        (*pGlobalData).hMixer as HMIXEROBJ,
        core::ptr::addr_of_mut!(mxcd),
        (MIXER_OBJECTF_HMIXER as DWORD) | (MIXER_SETCONTROLDETAILSF_VALUE as DWORD),
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }

    (*pGlobalData).muteVal = mxcdMute.fValue;
}
#[no_mangle]
pub unsafe extern "C" fn GetVolumeControl(pGlobalData: PGLOBAL_DATA) {
    let mut mxln: MIXERLINEW = core::mem::zeroed();
    let mut mxc: MIXERCONTROLW = core::mem::zeroed();
    let mut mxlc: MIXERLINECONTROLSW = core::mem::zeroed();

    if (*pGlobalData).hMixer.is_null() {
        return;
    }

    mxln.cbStruct = core::mem::size_of::<MIXERLINEW>() as DWORD;
    mxln.dwComponentType = MIXERLINE_COMPONENTTYPE_DST_SPEAKERS as DWORD;
    if mixerGetLineInfoW(
        (*pGlobalData).hMixer as HMIXEROBJ,
        &raw mut mxln,
        (MIXER_OBJECTF_HMIXER | MIXER_GETLINEINFOF_COMPONENTTYPE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }

    (*pGlobalData).volumeChannels = mxln.cChannels;

    mxlc.cbStruct = core::mem::size_of::<MIXERLINECONTROLSW>() as DWORD;
    mxlc.dwLineID = mxln.dwLineID;
    mxlc.__bindgen_anon_1.dwControlType = MIXERCONTROL_CONTROLTYPE_VOLUME as DWORD;
    mxlc.cControls = 1 as DWORD;
    mxlc.cbmxctrl = core::mem::size_of::<MIXERCONTROLW>() as DWORD;
    mxlc.pamxctrl = &raw mut mxc;
    if mixerGetLineControlsW(
        (*pGlobalData).hMixer as HMIXEROBJ,
        &raw mut mxlc,
        (MIXER_OBJECTF_HMIXER | MIXER_GETLINECONTROLSF_ONEBYTYPE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }

    (*pGlobalData).volumeMinimum = mxc.Bounds.c2rust_unnamed_1.dwMinimum;
    (*pGlobalData).volumeMaximum = mxc.Bounds.c2rust_unnamed_1.dwMaximum;
    (*pGlobalData).volumeControlID = mxc.dwControlID;
    (*pGlobalData).volumeStep = (*pGlobalData)
        .volumeMaximum
        .wrapping_sub((*pGlobalData).volumeMinimum)
        .wrapping_div((VOLUME_MAX - VOLUME_MIN) as DWORD);

    (*pGlobalData).volumeInitialValues = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (mxln.cChannels as SIZE_T)
            .wrapping_mul(core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as SIZE_T),
    ) as PMIXERCONTROLDETAILS_UNSIGNED;
    if (*pGlobalData).volumeInitialValues.is_null() {
        return;
    }

    (*pGlobalData).volumePreviousValues = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (mxln.cChannels as SIZE_T)
            .wrapping_mul(core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as SIZE_T),
    ) as PMIXERCONTROLDETAILS_UNSIGNED;
    if (*pGlobalData).volumePreviousValues.is_null() {
        return;
    }

    (*pGlobalData).volumeCurrentValues = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (mxln.cChannels as SIZE_T)
            .wrapping_mul(core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as SIZE_T),
    ) as PMIXERCONTROLDETAILS_UNSIGNED;
    if (*pGlobalData).volumeCurrentValues.is_null() {
        return;
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetVolumeValue(pGlobalData: PGLOBAL_DATA, bInit: BOOL) {
    let mut mxcd: MIXERCONTROLDETAILS = core::mem::zeroed();
    let mut i: DWORD = 0;
    if (*pGlobalData).hMixer.is_null() {
        return;
    }
    mxcd.cbStruct = ::core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
    mxcd.dwControlID = (*pGlobalData).volumeControlID;
    mxcd.cChannels = (*pGlobalData).volumeChannels;
    mxcd.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
    mxcd.cbDetails = ::core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD;
    mxcd.paDetails = (*pGlobalData).volumePreviousValues as PVOID;
    if mixerGetControlDetailsW(
        (*pGlobalData).hMixer as HMIXEROBJ,
        core::ptr::addr_of_mut!(mxcd),
        (MIXER_OBJECTF_HMIXER | MIXER_GETCONTROLDETAILSF_VALUE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }
    (*pGlobalData).maxVolume = 0 as DWORD;
    i = 0 as DWORD;
    while i < (*pGlobalData).volumeChannels {
        (*(*pGlobalData).volumeCurrentValues.offset(i as isize)).dwValue =
            (*(*pGlobalData).volumePreviousValues.offset(i as isize)).dwValue;
        if (*(*pGlobalData).volumePreviousValues.offset(i as isize)).dwValue
            > (*pGlobalData).maxVolume
        {
            (*pGlobalData).maxVolume =
                (*(*pGlobalData).volumePreviousValues.offset(i as isize)).dwValue;
        }
        if bInit != 0 {
            (*(*pGlobalData).volumeInitialValues.offset(i as isize)).dwValue =
                (*(*pGlobalData).volumeCurrentValues.offset(i as isize)).dwValue;
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn SetVolumeValue(pGlobalData: PGLOBAL_DATA, dwPosition: DWORD) {
    let mut mxcd: MIXERCONTROLDETAILS = core::mem::zeroed();
    let mut dwVolume: DWORD = 0;
    let mut i: DWORD = 0;

    if (*pGlobalData).hMixer.is_null() {
        return;
    }

    if dwPosition == VOLUME_MIN as DWORD {
        dwVolume = (*pGlobalData).volumeMinimum;
    } else if dwPosition == VOLUME_MAX as DWORD {
        dwVolume = (*pGlobalData).volumeMaximum;
    } else {
        dwVolume = dwPosition
            .wrapping_mul((*pGlobalData).volumeStep)
            .wrapping_add((*pGlobalData).volumeMinimum);
    }

    i = 0 as DWORD;
    while i < (*pGlobalData).volumeChannels {
        if (*(*pGlobalData).volumePreviousValues.offset(i as isize)).dwValue
            == (*pGlobalData).maxVolume
        {
            (*(*pGlobalData).volumeCurrentValues.offset(i as isize)).dwValue = dwVolume;
        } else {
            (*(*pGlobalData).volumeCurrentValues.offset(i as isize)).dwValue =
                (*(*pGlobalData).volumePreviousValues.offset(i as isize))
                    .dwValue
                    .wrapping_mul(dwVolume)
                    .wrapping_div((*pGlobalData).maxVolume);
        }
        i = i.wrapping_add(1);
    }

    mxcd.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
    mxcd.dwControlID = (*pGlobalData).volumeControlID;
    mxcd.cChannels = (*pGlobalData).volumeChannels;
    mxcd.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
    mxcd.cbDetails = core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD;
    mxcd.paDetails = (*pGlobalData).volumeCurrentValues as PVOID;

    if mixerSetControlDetails(
        (*pGlobalData).hMixer as HMIXEROBJ,
        core::ptr::addr_of_mut!(mxcd),
        (MIXER_OBJECTF_HMIXER | MIXER_SETCONTROLDETAILSF_VALUE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }
}

unsafe extern "C" fn RestoreVolumeValue(mut pGlobalData: PGLOBAL_DATA) {
    let mut mxcd: MIXERCONTROLDETAILS = core::mem::zeroed();
    if (*pGlobalData).hMixer.is_null() {
        return;
    }
    mxcd.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
    mxcd.dwControlID = (*pGlobalData).volumeControlID;
    mxcd.cChannels = (*pGlobalData).volumeChannels;
    mxcd.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
    mxcd.cbDetails = core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD;
    mxcd.paDetails = (*pGlobalData).volumeInitialValues as PVOID;
    mixerSetControlDetails(
        (*pGlobalData).hMixer as HMIXEROBJ,
        core::ptr::addr_of_mut!(mxcd),
        (MIXER_OBJECTF_HMIXER | MIXER_SETCONTROLDETAILSF_VALUE) as DWORD,
    );
}

unsafe extern "C" fn SetSystrayVolumeIconState(bEnabled: BOOL) {
    let mut hwndTaskBar: HWND;

    hwndTaskBar = FindWindowW(RUSTCONV_WSTR_6A661B0A8BF4.as_ptr(), core::ptr::null_mut());
    if hwndTaskBar.is_null() {
        return;
    }

    SendMessageW(
        hwndTaskBar,
        (WM_USER + 220) as UINT,
        4 as WPARAM,
        bEnabled as LPARAM,
    );
}

unsafe extern "C" fn GetSystrayVolumeIconState() -> BOOL {
    let mut hwndTaskBar: HWND;
    hwndTaskBar = FindWindowW(RUSTCONV_WSTR_6A661B0A8BF4.as_ptr(), core::ptr::null_mut());
    if hwndTaskBar.is_null() {
        return (FALSE) as BOOL;
    }

    return SendMessageW(hwndTaskBar, (WM_USER + 221) as UINT, 4 as WPARAM, 0 as LPARAM) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InitVolumeControls(hwndDlg: HWND, pGlobalData: PGLOBAL_DATA) {
    let mut NumMixers: UINT;
    let mut mxc: MIXERCAPSW = core::mem::zeroed();
    let mut szNoDevices: [WCHAR; 256] = [0; 256];

    LoadStringW(hApplet, IDS_NO_DEVICES, core::ptr::addr_of_mut!(szNoDevices) as *mut WCHAR, (_countof!(szNoDevices)) as INT);

    SendDlgItemMessageW(hwndDlg, (IDC_VOLUME_TRACKBAR as i32), TBM_SETRANGE, TRUE as WPARAM, MAKELONG!(VOLUME_MIN, VOLUME_MAX) as LPARAM);
    SendDlgItemMessageW(hwndDlg, (IDC_VOLUME_TRACKBAR as i32), TBM_SETTICFREQ, (VOLUME_TICFREQ) as WPARAM, 0);
    SendDlgItemMessageW(hwndDlg, (IDC_VOLUME_TRACKBAR as i32), TBM_SETPAGESIZE, 0, (VOLUME_PAGESIZE) as LPARAM);

    NumMixers = mixerGetNumDevs();
    if NumMixers == 0
    {
        EnableWindow(GetDlgItem(hwndDlg, (IDC_VOLUME_TRACKBAR as i32)), (FALSE) as BOOL);
        EnableWindow(GetDlgItem(hwndDlg, (IDC_VOLUME_LOW as i32)), (FALSE) as BOOL);
        EnableWindow(GetDlgItem(hwndDlg, (IDC_VOLUME_HIGH as i32)), (FALSE) as BOOL);
        EnableWindow(GetDlgItem(hwndDlg, (IDC_MUTE_CHECKBOX as i32)), (FALSE) as BOOL);
        EnableWindow(GetDlgItem(hwndDlg, (IDC_ICON_IN_TASKBAR as i32)), (FALSE) as BOOL);
        EnableWindow(GetDlgItem(hwndDlg, (IDC_ADVANCED_BTN as i32)), (FALSE) as BOOL);
        EnableWindow(GetDlgItem(hwndDlg, (IDC_SPEAKER_VOL_BTN as i32)), (FALSE) as BOOL);
        EnableWindow(GetDlgItem(hwndDlg, (IDC_ADVANCED2_BTN as i32)), (FALSE) as BOOL);
        SendDlgItemMessageW(hwndDlg, (IDC_MUTE_ICON as i32), STM_SETIMAGE, (IMAGE_ICON) as WPARAM, (*pGlobalData).hIconNoHW as LPARAM);
        SetDlgItemTextW(hwndDlg, (IDC_DEVICE_NAME as i32), core::ptr::addr_of_mut!(szNoDevices) as *mut WCHAR as LPCWSTR);
        return;
    }

    if mixerOpen(core::ptr::addr_of_mut!((*pGlobalData).hMixer), 0, PtrToUlong!(hwndDlg) as DWORD_PTR, 0, MIXER_OBJECTF_MIXER | CALLBACK_WINDOW) != MMSYSERR_NOERROR
    {
        MessageBoxW(hwndDlg, RUSTCONV_WSTR_C4C3B6321C70.as_ptr(), core::ptr::null_mut(), MB_OK);
        return;
    }

    ZeroMemory!(core::ptr::addr_of_mut!(mxc), core::mem::size_of::<MIXERCAPSW>() as usize);
    if mixerGetDevCapsW(PtrToUint!((*pGlobalData).hMixer) as UINT_PTR, core::ptr::addr_of_mut!(mxc), core::mem::size_of::<MIXERCAPSW>() as UINT) != MMSYSERR_NOERROR
    {
        MessageBoxW(hwndDlg, RUSTCONV_WSTR_B80563C54935.as_ptr(), core::ptr::null_mut(), MB_OK);
        return;
    }

    CheckDlgButton(hwndDlg,
                   (IDC_ICON_IN_TASKBAR as i32),
                   (if GetSystrayVolumeIconState() != 0 { BST_CHECKED } else { BST_UNCHECKED }) as UINT);

    GetMuteControl(pGlobalData);
    GetMuteState(pGlobalData);
    if (*pGlobalData).muteVal != 0
    {
        SendDlgItemMessageW(hwndDlg, (IDC_MUTE_CHECKBOX as i32), BM_SETCHECK, BST_CHECKED as WPARAM, 0);
        SendDlgItemMessageW(hwndDlg, (IDC_MUTE_ICON as i32), STM_SETIMAGE, (IMAGE_ICON) as WPARAM, (*pGlobalData).hIconMuted as LPARAM);
    }
    else
    {
        SendDlgItemMessageW(hwndDlg, (IDC_MUTE_CHECKBOX as i32), BM_SETCHECK, BST_UNCHECKED as WPARAM, 0);
        SendDlgItemMessageW(hwndDlg, (IDC_MUTE_ICON as i32), STM_SETIMAGE, (IMAGE_ICON) as WPARAM, (*pGlobalData).hIconUnMuted as LPARAM);
    }

    GetVolumeControl(pGlobalData);
    GetVolumeValue(pGlobalData, (TRUE) as BOOL);

    SendDlgItemMessageW(hwndDlg, (IDC_DEVICE_NAME as i32), WM_SETTEXT, 0, core::ptr::addr_of_mut!(mxc.szPname) as *mut WCHAR as LPARAM);
    SendDlgItemMessageW(hwndDlg, (IDC_VOLUME_TRACKBAR as i32), TBM_SETPOS, TRUE as WPARAM,
        ((*pGlobalData).maxVolume.wrapping_sub((*pGlobalData).volumeMinimum) as LPARAM) / ((*pGlobalData).volumeStep as LPARAM));
}
#[no_mangle]
pub unsafe extern "C" fn SaveData(hwndDlg: HWND) {
    let mut bShowIcon: BOOL;

    bShowIcon = (IsDlgButtonChecked(hwndDlg, (IDC_ICON_IN_TASKBAR as i32)) == BST_CHECKED) as BOOL;

    SetSystrayVolumeIconState(bShowIcon);
}
#[no_mangle]
pub unsafe extern "C" fn LaunchSoundControl(hwndDlg: HWND) {
    if (ShellExecuteW(core::ptr::null_mut(), RUSTCONV_WSTR_82DB442E7A60.as_ptr(), RUSTCONV_WSTR_566AB50D80A9.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), (SW_SHOWNORMAL) as INT) as INT_PTR) > 32 as INT_PTR {
        return;
    }
    MessageBoxW(hwndDlg, RUSTCONV_WSTR_2B9D6BDC2386.as_ptr(), core::ptr::null_mut(), MB_OK as UINT);
}
#[no_mangle]
pub unsafe extern "system" fn VolumeDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    static mut speakImgSize: [INT; 2] = [72, 72];
    let mut pGlobalData: PGLOBAL_DATA;

    pGlobalData = GetWindowLongPtrW(hwndDlg, DWLP_USER) as PGLOBAL_DATA;

    match uMsg {
        MM_MIXM_LINE_CHANGE => {
            GetMuteState(pGlobalData);
            if (*pGlobalData).muteVal != 0 {
                SendDlgItemMessageW(hwndDlg, (IDC_MUTE_CHECKBOX as i32), BM_SETCHECK, BST_CHECKED as WPARAM, 0);
                SendDlgItemMessageW(hwndDlg, (IDC_MUTE_ICON as i32), STM_SETIMAGE, (IMAGE_ICON) as WPARAM, (*pGlobalData).hIconMuted as LPARAM);
            } else {
                SendDlgItemMessageW(hwndDlg, (IDC_MUTE_CHECKBOX as i32), BM_SETCHECK, BST_UNCHECKED as WPARAM, 0);
                SendDlgItemMessageW(hwndDlg, (IDC_MUTE_ICON as i32), STM_SETIMAGE, (IMAGE_ICON) as WPARAM, (*pGlobalData).hIconUnMuted as LPARAM);
            }
        }
        MM_MIXM_CONTROL_CHANGE => {
            GetVolumeValue(pGlobalData, (FALSE) as BOOL);
            SendDlgItemMessageW(
                hwndDlg,
                (IDC_VOLUME_TRACKBAR as i32),
                TBM_SETPOS,
                TRUE as WPARAM,
                ((*pGlobalData).maxVolume.wrapping_sub((*pGlobalData).volumeMinimum) as LPARAM)
                    / ((*pGlobalData).volumeStep as LPARAM),
            );
        }
        WM_INITDIALOG => {
            pGlobalData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<GLOBAL_DATA>()) as SIZE_T) as PGLOBAL_DATA;
            SetWindowLongPtrW(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            (*pGlobalData).hIconUnMuted = (LoadImageW(hApplet, MAKEINTRESOURCEW!(IDI_CPLICON), IMAGE_ICON, 32, 32, LR_DEFAULTCOLOR)).cast();
            (*pGlobalData).hIconMuted = (LoadImageW(hApplet, MAKEINTRESOURCEW!(IDI_MUTED_ICON), IMAGE_ICON, 32, 32, LR_DEFAULTCOLOR)).cast();
            (*pGlobalData).hIconNoHW = (LoadImageW(hApplet, MAKEINTRESOURCEW!(IDI_NO_HW), IMAGE_ICON, 32, 32, LR_DEFAULTCOLOR)).cast();
            (*pGlobalData).hIconSpeakImg = (LoadImageW(
                hApplet,
                MAKEINTRESOURCEW!(IDI_SPEAKIMG),
                IMAGE_ICON,
                speakImgSize[0 as usize],
                speakImgSize[1 as usize],
                LR_DEFAULTCOLOR,
            )).cast();

            InitVolumeControls(hwndDlg, pGlobalData);
        }

        WM_DRAWITEM => {
            let mut lpDrawItem: LPDRAWITEMSTRUCT;
            lpDrawItem = lParam as LPDRAWITEMSTRUCT;
            if (*lpDrawItem).CtlID == IDC_SPEAKIMG {
                let mut left: LONG;

                /* Position image in centre of dialog */
                left = ((*lpDrawItem).rcItem.right - speakImgSize[0 as usize] as LONG) / (2 as LONG);

                DrawIconEx(
                    (*lpDrawItem).hDC,
                    left,
                    (*lpDrawItem).rcItem.top,
                    (*pGlobalData).hIconSpeakImg,
                    speakImgSize[0 as usize],
                    speakImgSize[1 as usize],
                    0,
                    core::ptr::null_mut(),
                    DI_NORMAL,
                );
            }
        }

        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                IDC_MUTE_CHECKBOX => {
                    if HIWORD!(wParam) as WPARAM == (BN_CLICKED) as _ {
                        SwitchMuteState(pGlobalData);
                        if (*pGlobalData).muteVal != 0 {
                            SendDlgItemMessageW(hwndDlg, (IDC_MUTE_ICON as i32), STM_SETIMAGE, (IMAGE_ICON) as WPARAM, (*pGlobalData).hIconMuted as LPARAM);
                        } else {
                            SendDlgItemMessageW(hwndDlg, (IDC_MUTE_ICON as i32), STM_SETIMAGE, (IMAGE_ICON) as WPARAM, (*pGlobalData).hIconUnMuted as LPARAM);
                        }

                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_ICON_IN_TASKBAR => {
                    if HIWORD!(wParam) as WPARAM == (BN_CLICKED) as _ {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_ADVANCED_BTN => {
                    LaunchSoundControl(hwndDlg);
                }

                IDC_SPEAKER_VOL_BTN => {
                    SpeakerVolume(hwndDlg);
                }
                _ => {}
            }
        }

        WM_HSCROLL => {
            let mut hVolumeTrackbar: HWND = GetDlgItem(hwndDlg, (IDC_VOLUME_TRACKBAR as i32));
            if hVolumeTrackbar == lParam as HWND {
                match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                    TB_THUMBPOSITION => {}

                    TB_ENDTRACK => {
                        PlaySoundW(SND_ALIAS_SYSTEMDEFAULT as LPCWSTR, core::ptr::null_mut(), SND_ALIAS_ID | SND_ASYNC);
                    }

                    _ => {
                        SetVolumeValue(
                            pGlobalData,
                            SendDlgItemMessageW(hwndDlg, (IDC_VOLUME_TRACKBAR as i32), TBM_GETPOS, 0, 0) as DWORD,
                        );
                    }
                }
            }
        }

        WM_DESTROY => {
            if !pGlobalData.is_null() {
                HeapFree(GetProcessHeap(), 0, ((*pGlobalData).volumeCurrentValues).cast());
                HeapFree(GetProcessHeap(), 0, ((*pGlobalData).volumePreviousValues).cast());
                HeapFree(GetProcessHeap(), 0, ((*pGlobalData).volumeInitialValues).cast());
                mixerClose((*pGlobalData).hMixer);

                if !(*pGlobalData).hIconSpeakImg.is_null() {
                    DestroyIcon((*pGlobalData).hIconSpeakImg);
                }

                if !(*pGlobalData).hIconNoHW.is_null() {
                    DestroyIcon((*pGlobalData).hIconNoHW);
                }

                if !(*pGlobalData).hIconMuted.is_null() {
                    DestroyIcon((*pGlobalData).hIconMuted);
                }

                if !(*pGlobalData).hIconUnMuted.is_null() {
                    DestroyIcon((*pGlobalData).hIconUnMuted);
                }

                HeapFree(GetProcessHeap(), 0, (pGlobalData).cast());
            }
        }

        WM_NOTIFY => {
            match ((*(lParam as LPNMHDR)).code) as i32 {
                PSN_APPLY => {
                    SaveData(hwndDlg);
                }

                PSN_RESET => {
                    RestoreVolumeValue(pGlobalData);
                }
                _ => {}
            }
            return TRUE as INT_PTR;
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}
