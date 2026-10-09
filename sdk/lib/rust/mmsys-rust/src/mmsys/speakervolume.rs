use crate::mmsys::mmsys::InitPropSheetPage;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_C4C3B6321C70: [u16; 18] = [
    0x0043u16, 0x0061, 0x006E, 0x006E, 0x006F, 0x0074, 0x0020, 0x006F, 0x0070, 0x0065,
    0x006E, 0x0020, 0x006D, 0x0069, 0x0078, 0x0065, 0x0072, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

pub type PPAGE_DATA = *mut PAGE_DATA;

unsafe extern "C" fn OnInitDialog(mut pPageData: PPAGE_DATA, mut hwndDlg: HWND) -> BOOL {
    let mut szBuffer: [WCHAR; 256] = [0; 256];
    let mut mxln: MIXERLINEW = core::mem::zeroed();
    let mut mxc: MIXERCONTROLW = core::mem::zeroed();
    let mut mxlctrl: MIXERLINECONTROLSW = core::mem::zeroed();
    let mut mxcd: MIXERCONTROLDETAILS = core::mem::zeroed();
    let mut i: INT = 0;
    let mut j: INT = 0;

    /* Open the mixer */
    if mixerOpen(
        core::ptr::addr_of_mut!((*pPageData).hMixer),
        0 as UINT,
        PtrToUlong!(hwndDlg as *const core::ffi::c_void) as DWORD_PTR,
        0 as DWORD_PTR,
        (MIXER_OBJECTF_MIXER | CALLBACK_WINDOW) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        MessageBoxW(
            hwndDlg,
            RUSTCONV_WSTR_C4C3B6321C70.as_ptr(),
            core::ptr::null::<WCHAR>(),
            MB_OK as UINT,
        );
        return (FALSE) as BOOL;
    }

    /* Retrieve the mixer information */
    mxln.cbStruct = core::mem::size_of::<MIXERLINEW>() as DWORD;
    mxln.dwComponentType = MIXERLINE_COMPONENTTYPE_DST_SPEAKERS as DWORD;

    if mixerGetLineInfoW(
        (*pPageData).hMixer as HMIXEROBJ,
        core::ptr::addr_of_mut!(mxln),
        (MIXER_OBJECTF_HMIXER | MIXER_GETLINEINFOF_COMPONENTTYPE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return (FALSE) as BOOL;
    }

    (*pPageData).volumeChannels = mxln.cChannels;

    /* Retrieve the line information */
    mxlctrl.cbStruct = core::mem::size_of::<MIXERLINECONTROLSW>() as DWORD;
    mxlctrl.dwLineID = mxln.dwLineID;
    mxlctrl.__bindgen_anon_1.dwControlType = MIXERCONTROL_CONTROLTYPE_VOLUME as DWORD;
    mxlctrl.cControls = 1 as DWORD;
    mxlctrl.cbmxctrl = core::mem::size_of::<MIXERCONTROLW>() as DWORD;
    mxlctrl.pamxctrl = core::ptr::addr_of_mut!(mxc) as LPMIXERCONTROLW;

    if mixerGetLineControlsW(
        (*pPageData).hMixer as HMIXEROBJ,
        core::ptr::addr_of_mut!(mxlctrl),
        (MIXER_OBJECTF_HMIXER | MIXER_GETLINECONTROLSF_ONEBYTYPE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return (FALSE) as BOOL;
    }

    (*pPageData).volumeControlID = mxc.dwControlID;
    (*pPageData).volumeMinimum = mxc.Bounds.c2rust_unnamed_1.dwMinimum;
    (*pPageData).volumeMaximum = mxc.Bounds.c2rust_unnamed_1.dwMaximum;
    (*pPageData).volumeStep = (*pPageData)
        .volumeMaximum
        .wrapping_sub((*pPageData).volumeMinimum)
        .wrapping_div((VOLUME_MAX - VOLUME_MIN) as DWORD);

    /* Allocate a buffer for all channel volume values */
    (*pPageData).volumeValues = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (mxln.cChannels as SIZE_T)
            .wrapping_mul(core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as SIZE_T),
    ) as PMIXERCONTROLDETAILS_UNSIGNED;
    if (*pPageData).volumeValues.is_null() {
        return (FALSE) as BOOL;
    }

    /* Retrieve the channel volume values */
    mxcd.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
    mxcd.dwControlID = mxc.dwControlID;
    mxcd.cChannels = mxln.cChannels;
    mxcd.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
    mxcd.cbDetails = core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD;
    mxcd.paDetails = (*pPageData).volumeValues as PVOID;

    if mixerGetControlDetailsW(
        (*pPageData).hMixer as HMIXEROBJ,
        core::ptr::addr_of_mut!(mxcd),
        (MIXER_OBJECTF_HMIXER | MIXER_GETCONTROLDETAILSF_VALUE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return (FALSE) as BOOL;
    }

    /* Initialize the channels */
    i = 0 as INT;
    while (i as DWORD) < min(mxln.cChannels, 5 as DWORD) {
        j = (i * 4 as INT) as INT;

        /* Set the channel name */
        LoadStringW(
            hApplet,
            (IDS_SPEAKER_LEFT + (i) as u32) as UINT,
            core::ptr::addr_of_mut!(szBuffer) as LPWSTR,
            _countof!(szBuffer) as core::ffi::c_int,
        );
        SetWindowTextW(
            GetDlgItem(hwndDlg, 9472 + j),
            core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR as LPCWSTR,
        );

        /* Initialize the channel trackbar */
        SendDlgItemMessageW(
            hwndDlg,
            (9475 + j) as core::ffi::c_int,
            TBM_SETRANGE as UINT,
            TRUE as WPARAM,
            MAKELPARAM!(VOLUME_MIN, VOLUME_MAX),
        );
        SendDlgItemMessageW(
            hwndDlg,
            (9475 + j) as core::ffi::c_int,
            TBM_SETTICFREQ as UINT,
            VOLUME_TICFREQ as WPARAM,
            0 as LPARAM,
        );
        SendDlgItemMessageW(
            hwndDlg,
            (9475 + j) as core::ffi::c_int,
            TBM_SETPAGESIZE as UINT,
            0 as WPARAM,
            VOLUME_PAGESIZE as LPARAM,
        );
        SendDlgItemMessageW(
            hwndDlg,
            (9475 + j) as core::ffi::c_int,
            TBM_SETPOS as UINT,
            TRUE as WPARAM,
            ((*(*pPageData).volumeValues.offset(i as isize))
                .dwValue
                .wrapping_sub((*pPageData).volumeMinimum) as LPARAM)
                / ((*pPageData).volumeStep as LPARAM),
        );
        i += 1;
    }

    /* Hide the unused controls */
    i = mxln.cChannels as INT;
    while i < 8 as INT {
        j = (i * 4 as INT) as INT;
        ShowWindow(
            GetDlgItem(hwndDlg, 9472 + j),
            (SW_HIDE) as i32,
        );
        ShowWindow(
            GetDlgItem(hwndDlg, 9473 + j),
            (SW_HIDE) as i32,
        );
        ShowWindow(
            GetDlgItem(hwndDlg, 9474 + j),
            (SW_HIDE) as i32,
        );
        ShowWindow(
            GetDlgItem(hwndDlg, 9475 + j),
            (SW_HIDE) as i32,
        );
        i += 1;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn OnMixerControlChange(pPageData: PPAGE_DATA, hwndDlg: HWND) {
    let mut mxcd: MIXERCONTROLDETAILS = core::mem::zeroed();
    let mut i: INT = 0;
    let mut j: INT = 0;

    /* Retrieve the channel volume values */
    mxcd.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
    mxcd.dwControlID = (*pPageData).volumeControlID;
    mxcd.cChannels = (*pPageData).volumeChannels;
    mxcd.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
    mxcd.cbDetails = core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD;
    mxcd.paDetails = (*pPageData).volumeValues as PVOID;

    if mixerGetControlDetailsW(
        (*pPageData).hMixer as HMIXEROBJ,
        core::ptr::addr_of_mut!(mxcd),
        (MIXER_OBJECTF_HMIXER | MIXER_GETCONTROLDETAILSF_VALUE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }

    i = 0 as INT;
    while (i as DWORD) < (*pPageData).volumeChannels {
        j = i * 4;

        SendDlgItemMessageW(
            hwndDlg,
            9475 + j,
            TBM_SETPOS as UINT,
            TRUE as WPARAM,
            (*(*pPageData).volumeValues.offset(i as isize))
                .dwValue
                .wrapping_sub((*pPageData).volumeMinimum) as LPARAM
                / (*pPageData).volumeStep as LPARAM,
        );
        i += 1;
    }
}

unsafe extern "C" fn OnHScroll(
    pPageData: PPAGE_DATA,
    hwndDlg: HWND,
    wParam: WPARAM,
    lParam: LPARAM,
) {
    let mut mxcd: MIXERCONTROLDETAILS = core::mem::zeroed();
    let mut dwValue: DWORD = 0;
    let mut dwPosition: DWORD = 0;
    let mut id: INT = 0;
    let mut idx: INT = 0;
    let mut i: INT = 0;
    let mut j: INT = 0;

    id = GetWindowLongPtrW(lParam as HWND, GWLP_ID) as INT;
    if id < 9475 || id > 9503 {
        return;
    }

    if (id - 9475) % 4 != 0 {
        return;
    }

    dwPosition = SendDlgItemMessageW(
        hwndDlg,
        id,
        TBM_GETPOS,
        0,
        0,
    ) as DWORD;

    if dwPosition == VOLUME_MIN {
        dwValue = (*pPageData).volumeMinimum;
    } else if dwPosition == VOLUME_MAX {
        dwValue = (*pPageData).volumeMaximum;
    } else {
        dwValue = dwPosition
            .wrapping_mul((*pPageData).volumeStep)
            .wrapping_add((*pPageData).volumeMinimum);
    }

    if (*pPageData).volumeSync != 0 {
        i = 0 as INT;
        while (i as DWORD) < (*pPageData).volumeChannels {
            j = (9475 + (i * 4)) as INT;
            if j != id {
                SendDlgItemMessageW(
                    hwndDlg,
                    j,
                    TBM_SETPOS,
                    TRUE as WPARAM,
                    dwPosition as LPARAM,
                );
            }

            (*(*pPageData).volumeValues.offset(i as isize)).dwValue = dwValue;
            i += 1;
        }
    } else {
        idx = ((id - 9475) / 4) as INT;
        (*(*pPageData).volumeValues.offset(idx as isize)).dwValue = dwValue;
    }

    mxcd.cbStruct = core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
    mxcd.dwControlID = (*pPageData).volumeControlID;
    mxcd.cChannels = (*pPageData).volumeChannels;
    mxcd.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
    mxcd.cbDetails = core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD;
    mxcd.paDetails = (*pPageData).volumeValues as PVOID;

    if mixerSetControlDetails(
        (*pPageData).hMixer as HMIXEROBJ,
        &raw mut mxcd,
        (MIXER_OBJECTF_HMIXER | MIXER_SETCONTROLDETAILSF_VALUE) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }
}

unsafe extern "C" fn OnSetDefaults(mut pPageData: PPAGE_DATA, mut hwndDlg: HWND) {
    let mut mxcd: MIXERCONTROLDETAILS = tMIXERCONTROLDETAILS {
        cbStruct: 0,
        dwControlID: 0,
        cChannels: 0,
        __bindgen_anon_1: tMIXERCONTROLDETAILS__bindgen_ty_1 {
            hwndOwner: ::core::ptr::null_mut::<HWND__>(),
        },
        cbDetails: 0,
        paDetails: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut dwValue: DWORD = 0;
    let mut i: DWORD = 0;
    dwValue = ((((VOLUME_MAX - VOLUME_MIN) / (2 as ::core::ffi::c_int) as u32) as DWORD)
        .wrapping_mul((*pPageData).volumeStep))
        .wrapping_add((*pPageData).volumeMinimum);
    i = 0 as DWORD;
    while i < (*pPageData).volumeChannels {
        (*(*pPageData).volumeValues.offset(i as isize)).dwValue = dwValue;
        i = i.wrapping_add(1);
    }
    mxcd.cbStruct = ::core::mem::size_of::<MIXERCONTROLDETAILS>() as DWORD;
    mxcd.dwControlID = (*pPageData).volumeControlID;
    mxcd.cChannels = (*pPageData).volumeChannels;
    mxcd.__bindgen_anon_1.cMultipleItems = 0 as DWORD;
    mxcd.cbDetails = ::core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD;
    mxcd.paDetails = (*pPageData).volumeValues as PVOID;
    if mixerSetControlDetails(
        (*pPageData).hMixer as HMIXEROBJ,
        core::ptr::addr_of_mut!(mxcd),
        (MIXER_OBJECTF_HMIXER | MIXER_SETCONTROLDETAILSF_VALUE as ::core::ffi::c_uint) as DWORD,
    ) != MMSYSERR_NOERROR as MMRESULT
    {
        return;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SpeakerVolumeDlgProc(
    mut hwndDlg: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    let mut pPageData: PPAGE_DATA = ::core::ptr::null_mut::<_PAGE_DATA>();

    pPageData = GetWindowLongPtrW(hwndDlg, DWLP_USER as ::core::ffi::c_int) as PPAGE_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pPageData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                ::core::mem::size_of::<PAGE_DATA>() as SIZE_T,
            ) as PPAGE_DATA;
            SetWindowLongPtrW(
                hwndDlg,
                DWLP_USER as ::core::ffi::c_int,
                pPageData as LONG_PTR,
            );

            if !pPageData.is_null() {
                OnInitDialog(pPageData, hwndDlg);
            }
        }
        WM_DESTROY => {
            if !pPageData.is_null() {
                if !(*pPageData).volumeValues.is_null() {
                    HeapFree(
                        GetProcessHeap(),
                        0 as DWORD,
                        (*pPageData).volumeValues as PVOID,
                    );
                }

                if !(*pPageData).hMixer.is_null() {
                    mixerClose((*pPageData).hMixer);
                }

                HeapFree(
                    GetProcessHeap(),
                    0 as DWORD,
                    pPageData as PVOID,
                );
                pPageData = ::core::ptr::null_mut::<_PAGE_DATA>();
                SetWindowLongPtrW(
                    hwndDlg,
                    DWLP_USER as ::core::ffi::c_int,
                    NULL as LONG_PTR,
                );
            }
        }
        WM_HSCROLL => {
            if !pPageData.is_null() {
                OnHScroll(pPageData, hwndDlg, wParam, lParam);
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            }
        }
        WM_COMMAND => match LOWORD!(wParam) as ::core::ffi::c_int {
            9504 => {
                if (HIWORD!(wParam) as ::core::ffi::c_int) as u32 == BN_CLICKED {
                    (*pPageData).volumeSync = (SendDlgItemMessageW(
                        hwndDlg,
                        9504 as ::core::ffi::c_int,
                        BM_GETCHECK as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) == BST_CHECKED as LRESULT) as BOOL;
                }
            }
            9505 => {
                OnSetDefaults(pPageData, hwndDlg);
                PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
            }
            _ => {}
        },
        WM_NOTIFY => {
            return TRUE as INT_PTR;
        }
        MM_MIXM_CONTROL_CHANGE => {
            if !pPageData.is_null() {
                OnMixerControlChange(pPageData, hwndDlg);
            }
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn SpeakerVolume(hwndDlg: HWND) -> INT_PTR {
    let mut psp: [PROPSHEETPAGEW; 1] = core::mem::zeroed();
    let mut psh: PROPSHEETHEADERW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(psh) as *mut core::ffi::c_void,
        core::mem::size_of::<PROPSHEETHEADERW>(),
    );
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as DWORD;
    psh.dwFlags = PSH_PROPSHEETPAGE as DWORD;
    psh.hwndParent = hwndDlg;
    psh.hInstance = hApplet;
    psh.pszCaption = MAKEINTRESOURCE!(IDS_SPEAKER_VOLUME) as LPCWSTR;
    psh.nPages = _countof!(psp) as UINT;
    psh.__bindgen_anon_2.nStartPage = 0 as UINT;
    psh.__bindgen_anon_3.ppsp = psp.as_mut_ptr() as LPCPROPSHEETPAGEW;

    InitPropSheetPage(
        psp.as_mut_ptr().offset(0 as isize) as *mut PROPSHEETPAGEW,
        IDD_MULTICHANNEL as WORD,
        Some(SpeakerVolumeDlgProc),
    );
    psp[0 as usize].dwFlags |= PSP_USETITLE as DWORD;
    psp[0 as usize].hInstance = hApplet;
    psp[0 as usize].pszTitle =
        MAKEINTRESOURCE!(IDS_SPEAKER_VOLUME) as LPCWSTR;

    return (PropertySheetW(core::ptr::addr_of_mut!(psh) as LPCPROPSHEETHEADERW)
        != -(1) as INT_PTR) as LONG as INT_PTR;
}
