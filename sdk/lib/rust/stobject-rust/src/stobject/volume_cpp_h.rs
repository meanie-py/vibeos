use crate::prelude::*;
use crate::stobject::stobject_cpp_h::g_hInstance;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_566AB50D80A9: [WCHAR; 13] = [0x0073, 0x006E, 0x0064, 0x0076, 0x006F, 0x006C, 0x0033, 0x0032, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static CPP_COM_WSTR_BC817FFFA4AA: [WCHAR; 3] = [0x002F, 0x0074, 0x0000];
pub static CPP_COM_WSTR_FB213D92F063: [WCHAR; 19] = [0x0077, 0x0069, 0x006E, 0x006D, 0x006D, 0x005F, 0x0064, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x0063, 0x0068, 0x0061, 0x006E, 0x0067, 0x0065, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut g_hIconVolume: HICON = core::ptr::null_mut();
pub static mut g_hIconMute: HICON = core::ptr::null_mut();
pub static mut g_hMixer: HMIXER = core::ptr::null_mut();
pub static mut g_mixerLineID: DWORD = 0;
pub static mut g_muteControlID: DWORD = 0;
pub static mut g_mmDeviceChange: UINT = 0;
static mut g_IsMute: BOOL = 0;

unsafe extern "system" fn Volume_FindMixerControl(pSysTray: *mut CSysTray) -> HRESULT {
    let mut result: MMRESULT = 0;
    let mut mixerId: UINT = 0;
    let mut waveOutId: DWORD = 0;
    let mut param2: DWORD = 0;

    if !g_hMixer.is_null() {
        mixerClose(g_hMixer);
    }
    g_hMixer = core::ptr::null_mut();

    result = waveOutMessage(
        UlongToHandle!(WAVE_MAPPER) as HWAVEOUT,
        DRVM_MAPPER_PREFERRED_GET,
        core::ptr::addr_of_mut!(waveOutId) as DWORD_PTR,
        core::ptr::addr_of_mut!(param2) as DWORD_PTR,
    );
    if result != 0 {
        return E_FAIL;
    }

    if waveOutId == (1 as DWORD).wrapping_neg() {
        if waveOutGetNumDevs() == 0 {
            return S_FALSE;
        }

        mixerId = 0;
    } else {
        result = mixerGetID(
            UlongToHandle!(waveOutId) as HMIXEROBJ,
            core::ptr::addr_of_mut!(mixerId),
            MIXER_OBJECTF_WAVEOUT,
        );
        if result != 0 {
            return E_FAIL;
        }
    }

    let mut mixerLine: MIXERLINE = core::mem::zeroed();
    let mut mixerControl: MIXERCONTROL = core::mem::zeroed();
    let mut mixerLineControls: MIXERLINECONTROLS = core::mem::zeroed();

    g_mixerLineID = (1 as DWORD).wrapping_neg();
    g_muteControlID = (1 as DWORD).wrapping_neg();

    result = mixerOpen(
        core::ptr::addr_of_mut!(g_hMixer),
        mixerId,
        (*pSysTray).__base2.m_hWnd as DWORD_PTR,
        0,
        MIXER_OBJECTF_HMIXER | CALLBACK_WINDOW,
    );
    if result != 0 {
        return E_FAIL;
    }

    mixerLine.cbStruct = core::mem::size_of::<MIXERLINE>() as DWORD;
    mixerLine.dwComponentType = MIXERLINE_COMPONENTTYPE_DST_SPEAKERS;

    result = mixerGetLineInfoW(
        g_hMixer as HMIXEROBJ,
        core::ptr::addr_of_mut!(mixerLine),
        MIXER_OBJECTF_HMIXER | MIXER_GETLINEINFOF_COMPONENTTYPE,
    );
    if result != 0 {
        return E_FAIL;
    }

    g_mixerLineID = mixerLine.dwLineID;

    mixerLineControls.cbStruct = core::mem::size_of::<MIXERLINECONTROLS>() as DWORD;
    mixerLineControls.dwLineID = mixerLine.dwLineID;
    mixerLineControls.cControls = 1;
    mixerLineControls.__bindgen_anon_1.dwControlType = MIXERCONTROL_CONTROLTYPE_MUTE;
    mixerLineControls.pamxctrl = core::ptr::addr_of_mut!(mixerControl);
    mixerLineControls.cbmxctrl = core::mem::size_of::<MIXERCONTROL>() as DWORD;

    result = mixerGetLineControlsW(
        g_hMixer as HMIXEROBJ,
        core::ptr::addr_of_mut!(mixerLineControls),
        MIXER_OBJECTF_HMIXER | MIXER_GETLINECONTROLSF_ONEBYTYPE,
    );
    if result != MMSYSERR_NOERROR {
        return E_FAIL;
    }

    g_muteControlID = mixerControl.dwControlID;

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn Volume_IsMute() -> HRESULT {
    let mut mixerControlDetails: MIXERCONTROLDETAILS = core::mem::zeroed();

    if !g_hMixer.is_null() && g_muteControlID != 0u32.wrapping_sub(1 as DWORD) {
        let mut detailsResult: BOOL = 0;
        mixerControlDetails.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
        mixerControlDetails.__bindgen_anon_1.hwndOwner = core::ptr::null_mut();
        mixerControlDetails.dwControlID = g_muteControlID;
        mixerControlDetails.cChannels = 1;
        mixerControlDetails.paDetails = core::ptr::addr_of_mut!(detailsResult) as LPVOID;
        mixerControlDetails.cbDetails = core::mem::size_of::<BOOL>() as DWORD;
        if mixerGetControlDetailsW(
            g_hMixer as HMIXEROBJ,
            core::ptr::addr_of_mut!(mixerControlDetails),
            MIXER_OBJECTF_HMIXER | MIXER_GETCONTROLDETAILSF_VALUE,
        ) != 0
        {
            return E_FAIL;
        }

        g_IsMute = (detailsResult != 0) as BOOL;
    }

    return S_OK;
}
#[no_mangle]
pub unsafe extern "system" fn Volume_Init(pSysTray: *mut CSysTray) -> HRESULT {
    let mut hr: HRESULT;
    let mut strTooltip: [u16; 128] = [0u16; 128];

    if (g_hMixer as usize) == 0 {
        hr = Volume_FindMixerControl(pSysTray);
        if ((hr) < 0) {
            return hr;
        }

        g_mmDeviceChange = RegisterWindowMessageW(CPP_COM_WSTR_FB213D92F063.as_ptr());
    }

    g_hIconVolume = LoadIcon(g_hInstance, MAKEINTRESOURCE!(IDI_VOLUME));
    g_hIconMute = LoadIcon(g_hInstance, MAKEINTRESOURCE!(IDI_VOLMUTE));

    Volume_IsMute();

    let icon: HICON;
    if ((g_IsMute) != 0) {
        icon = g_hIconMute;
    } else {
        icon = g_hIconVolume;
    }

    LoadStringW(g_hInstance, IDS_VOL_VOLUME, strTooltip.as_mut_ptr(), _countof!(strTooltip) as i32);
    return (*pSysTray).NotifyIcon((NIM_ADD) as INT, ID_ICON_VOLUME, icon, strTooltip.as_ptr(), 0);
}
#[no_mangle]
pub unsafe extern "system" fn Volume_Update(pSysTray: *mut CSysTray) -> HRESULT {
    let PrevState: BOOL;

    PrevState = g_IsMute;
    Volume_IsMute();

    if PrevState != g_IsMute {
        let mut strTooltip: [u16; 128] = [0; 128];
        let icon: HICON;
        if g_IsMute != 0 {
            icon = g_hIconMute;
            LoadStringW(g_hInstance, IDS_VOL_MUTED, strTooltip.as_mut_ptr(), (_countof!(strTooltip)) as i32);
        } else {
            icon = g_hIconVolume;
            LoadStringW(g_hInstance, IDS_VOL_VOLUME, strTooltip.as_mut_ptr(), (_countof!(strTooltip)) as i32);
        }

        return (*pSysTray).NotifyIcon((NIM_MODIFY) as INT, ID_ICON_VOLUME, icon, strTooltip.as_ptr(), 0);
    } else {
        return S_OK;
    }
}

#[no_mangle]
pub unsafe extern "system" fn Volume_Shutdown(pSysTray: *mut CSysTray) -> HRESULT {
    return (*pSysTray).NotifyIcon((NIM_DELETE) as INT, ID_ICON_VOLUME, core::ptr::null_mut(), core::ptr::null(), 0);
}
#[no_mangle]
pub unsafe extern "C" fn Volume_OnDeviceChange(pSysTray: *mut CSysTray, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    return Volume_FindMixerControl(pSysTray);
}

unsafe extern "C" fn _RunVolume(bTray: BOOL) {
    ShellExecuteW(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        CPP_COM_WSTR_566AB50D80A9.as_ptr(),
        if bTray != 0 { CPP_COM_WSTR_BC817FFFA4AA.as_ptr() } else { core::ptr::null_mut() },
        core::ptr::null_mut(),
        (SW_SHOWNORMAL) as INT,
    );
}

unsafe extern "C" fn _RunMMCpl() {
    CSysTray::RunDll(b"mmsys.cpl\0".as_ptr() as *const i8, b"\0".as_ptr() as *const i8);
}

unsafe extern "C" fn _ShowContextMenu__volume(pSysTray: *mut CSysTray) {
    let mut strAdjust = [0u16; 128];
    let mut strOpen = [0u16; 128];
    LoadStringW(g_hInstance, IDS_VOL_OPEN, strOpen.as_mut_ptr(), _countof!(strOpen) as i32);
    LoadStringW(g_hInstance, IDS_VOL_ADJUST, strAdjust.as_mut_ptr(), _countof!(strAdjust) as i32);

    let hPopup: HMENU = CreatePopupMenu();
    AppendMenuW(hPopup, MF_STRING, (IDS_VOL_OPEN) as UINT_PTR, strOpen.as_ptr());
    AppendMenuW(hPopup, MF_STRING, (IDS_VOL_ADJUST) as UINT_PTR, strAdjust.as_ptr());
    SetMenuDefaultItem(hPopup, IDS_VOL_OPEN, FALSE);

    let flags: DWORD = TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTALIGN | TPM_BOTTOMALIGN;
    let mut pt: POINT = core::mem::zeroed();
    SetForegroundWindow((*pSysTray).GetHWnd());
    GetCursorPos(&mut pt);

    let id: DWORD = TrackPopupMenuEx(hPopup, flags,
        pt.x, pt.y,
        (*pSysTray).GetHWnd(), core::ptr::null_mut()) as DWORD;

    DestroyMenu(hPopup);

    if id == IDS_VOL_OPEN {
        _RunVolume((FALSE) as BOOL);
    } else if id == IDS_VOL_ADJUST {
        _RunMMCpl();
    }
}
#[no_mangle]
pub unsafe extern "system" fn Volume_Message(pSysTray: *mut CSysTray, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, lResult: &mut LRESULT) -> HRESULT {
    if uMsg == g_mmDeviceChange {
        return Volume_OnDeviceChange(pSysTray, wParam, lParam);
    }

    match uMsg {
        _ if uMsg == WM_USER + 220 => {
            TRACE!("Volume_Message: WM_USER+220\n");
            if wParam == (VOLUME_SERVICE_FLAG as _) {
                if lParam != 0 {
                    (*pSysTray).EnableService(VOLUME_SERVICE_FLAG, (TRUE) as BOOL);
                    return Volume_Init(pSysTray);
                } else {
                    (*pSysTray).EnableService(VOLUME_SERVICE_FLAG, (FALSE) as BOOL);
                    return Volume_Shutdown(pSysTray);
                }
            }
            return S_FALSE;
        }

        _ if uMsg == WM_USER + 221 => {
            TRACE!("Volume_Message: WM_USER+221\n");
            if wParam == (VOLUME_SERVICE_FLAG as _) {
                *lResult = (*pSysTray).IsServiceEnabled(VOLUME_SERVICE_FLAG) as LRESULT;
                return S_OK;
            }
            return S_FALSE;
        }

        _ if uMsg == WM_TIMER => {
            if wParam == (VOLUME_TIMER_ID as _) {
                KillTimer((*pSysTray).GetHWnd(), (VOLUME_TIMER_ID as _));
                _RunVolume((TRUE) as BOOL);
            }
        }

        _ if uMsg == ID_ICON_VOLUME => {
            TRACE!("Volume_Message uMsg=%d, w=%x, l=%x\n", uMsg, wParam, lParam);

            match lParam {
                _ if (lParam) as u32 == WM_LBUTTONDOWN => {
                    SetTimer((*pSysTray).GetHWnd(), (VOLUME_TIMER_ID as _), GetDoubleClickTime(), None);
                }

                _ if (lParam) as u32 == WM_LBUTTONUP => {
                }

                _ if (lParam) as u32 == WM_LBUTTONDBLCLK => {
                    KillTimer((*pSysTray).GetHWnd(), (VOLUME_TIMER_ID as _));
                    _RunVolume((FALSE) as BOOL);
                }

                _ if (lParam) as u32 == WM_RBUTTONDOWN => {
                }

                _ if (lParam) as u32 == WM_RBUTTONUP => {
                    _ShowContextMenu__volume(pSysTray);
                }

                _ if (lParam) as u32 == WM_RBUTTONDBLCLK => {
                }

                _ if (lParam) as u32 == WM_MOUSEMOVE => {
                }

                _ => {}
            }
            return S_OK;
        }

        _ if uMsg == MM_MIXM_LINE_CHANGE => {
            Volume_Update(pSysTray);
            return S_OK;
        }

        _ => {
            TRACE!("Volume_Message received for unknown ID %d, ignoring.\n");
            return S_FALSE;
        }
    }

    return S_FALSE;
}
