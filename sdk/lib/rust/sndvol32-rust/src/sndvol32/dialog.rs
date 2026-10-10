use crate::sndvol32::misc::ReadLineConfig;
use crate::sndvol32::mixer::SndMixerEnumConnections;
use crate::sndvol32::mixer::SndMixerGetLineName;
use crate::sndvol32::mixer::SndMixerGetVolumeControlDetails;
use crate::sndvol32::mixer::SndMixerQueryControls;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_09DBAEEA59BF: [u16; 7] = [0x0062u16, 0x0075, 0x0074, 0x0074, 0x006F, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_2157805E1164: [u16; 4] = [0x0023u16, 0x0025, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_758288F5F048: [u16; 18] = [
    0x006Du16, 0x0073, 0x0063, 0x0074, 0x006C, 0x0073, 0x005F, 0x0074, 0x0072, 0x0061,
    0x0063, 0x006B, 0x0062, 0x0061, 0x0072, 0x0033, 0x0032, 0x0000,
];
pub static RUSTCONV_WSTR_B3FF2A1B03AC: [u16; 7] = [0x0073u16, 0x0074, 0x0061, 0x0074, 0x0069, 0x0063, 0x0000];

use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn ConvertRect(mut lpRect: LPRECT, mut xBaseUnit: UINT, mut yBaseUnit: UINT) {
    (*lpRect).left = MulDiv(
        (*lpRect).left as ::core::ffi::c_int,
        xBaseUnit as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
    ) as LONG;
    (*lpRect).right = MulDiv(
        (*lpRect).right as ::core::ffi::c_int,
        xBaseUnit as ::core::ffi::c_int,
        4 as ::core::ffi::c_int,
    ) as LONG;
    (*lpRect).top = MulDiv(
        (*lpRect).top as ::core::ffi::c_int,
        yBaseUnit as ::core::ffi::c_int,
        8 as ::core::ffi::c_int,
    ) as LONG;
    (*lpRect).bottom = MulDiv(
        (*lpRect).bottom as ::core::ffi::c_int,
        yBaseUnit as ::core::ffi::c_int,
        8 as ::core::ffi::c_int,
    ) as LONG;
}
#[no_mangle]
pub unsafe extern "C" fn LoadDialogResource(
    hModule: HMODULE,
    ResourceName: LPCWSTR,
    ResourceLength: LPDWORD,
) -> LPVOID {
    let mut hSrc: HRSRC = core::ptr::null_mut();
    let mut hRes: HGLOBAL = core::ptr::null_mut();
    let mut Result: LPVOID = core::ptr::null_mut();

    /* find resource */
    hSrc = FindResourceW(hModule, ResourceName, (RT_DIALOG as usize as LPCWSTR));

    if hSrc.is_null() {
        /* failed to find resource */
        return core::ptr::null_mut();
    }

    /* now load the resource */
    hRes = LoadResource(hAppInstance, hSrc);
    if hRes.is_null() {
        /* failed to load resource */
        return core::ptr::null_mut();
    }

    /* now lock the resource */
    Result = LockResource(hRes);

    if Result.is_null() {
        /* failed to lock resource */
        return core::ptr::null_mut();
    }

    if !ResourceLength.is_null() {
        /* store output length */
        *ResourceLength = SizeofResource(hAppInstance, hSrc);
    }

    /* done */
    return Result;
}
#[no_mangle]
pub unsafe extern "C" fn AddDialogControl(
    hwndDialog: HWND,
    OutWnd: *mut HWND,
    DialogOffset: LPRECT,
    DialogItem: PDLGITEMTEMPLATE,
    DialogIdMultiplier: DWORD,
    hFont: HFONT,
    xBaseUnit: UINT,
    yBaseUnit: UINT,
    MixerId: UINT,
) -> LPWORD {
    let mut rect: RECT = core::mem::zeroed();
    let mut Offset: LPWORD = core::ptr::null_mut();
    let mut ClassName: LPWSTR = core::ptr::null_mut();
    let mut WindowName: LPWSTR = core::ptr::null_mut();
    let mut WindowIdBuf: [WCHAR; 7] = [0; 7];
    let mut hwnd: HWND = core::ptr::null_mut();
    let mut wID: DWORD = 0;
    let mut nSteps: INT = 0;
    let mut i: INT = 0;

    /* initialize client rectangle */
    rect.left = (*DialogItem).x as LONG;
    rect.top = (*DialogItem).y as LONG;
    rect.right = ((*DialogItem).x as LONG) + ((*DialogItem).cx as LONG);
    rect.bottom = ((*DialogItem).y as LONG) + ((*DialogItem).cy as LONG);

    /* Convert Dialog units to pixes */
    ConvertRect(core::ptr::addr_of_mut!(rect), xBaseUnit, yBaseUnit);

    rect.left += (*DialogOffset).left;
    rect.right += (*DialogOffset).left;
    rect.top += (*DialogOffset).top;
    rect.bottom += (*DialogOffset).top;

    /* move offset after dialog item */
    Offset = DialogItem.offset(1 as isize) as LPWORD;

    if (*Offset as i32) == (0xFFFF as i32) {
        /* class is encoded as type */
        Offset = Offset.offset(1);

        /* get control type */
        let SwitchVal: WORD = *Offset;
        if (SwitchVal as i32) == (0x80 as i32) {
            ClassName = RUSTCONV_WSTR_09DBAEEA59BF.as_ptr() as LPWSTR;
        } else if (SwitchVal as i32) == (0x82 as i32) {
            ClassName = RUSTCONV_WSTR_B3FF2A1B03AC.as_ptr() as LPWSTR;
        } else {
            /* FIXME */
            assert!((0 as i32) != 0);
            ClassName = core::ptr::null_mut();
        }
        Offset = Offset.offset(1);
    } else {
        /* class name is encoded as string */
        ClassName = Offset as LPWSTR;

        /* move offset to the end of class string */
        Offset = Offset.add(
            wcslen(ClassName as *const u16) + (1 as usize),
        );
    }

    if (*Offset as i32) == (0xFFFF as i32) {
        /* Window name is encoded as ordinal */
        Offset = Offset.offset(1);
        wsprintf(
            core::ptr::addr_of_mut!(WindowIdBuf) as LPWSTR,
            RUSTCONV_WSTR_2157805E1164.as_ptr(),
            *Offset as DWORD,
        );
        WindowName = core::ptr::addr_of_mut!(WindowIdBuf) as *mut WCHAR as LPWSTR;
        Offset = Offset.offset(1);
    } else {
        /* window name is encoded as string */
        WindowName = Offset as LPWSTR;

        /* move offset to the end of class string */
        Offset = Offset.add(
            wcslen(WindowName as *const u16) + (1 as usize),
        );
    }

    if ((*DialogItem).id as i32) == (MAXWORD as i32) {
        /* id is not important */
        wID = (*DialogItem).id as DWORD;
    } else {
        /* calculate id */
        wID = ((*DialogItem).id as DWORD)
            .wrapping_mul(DialogIdMultiplier.wrapping_add(1 as DWORD));
    }

    /* now create the window */
    hwnd = CreateWindowExW(
        (*DialogItem).dwExtendedStyle,
        ClassName as LPCWSTR,
        WindowName as LPCWSTR,
        (*DialogItem).style,
        rect.left as INT,
        rect.top as INT,
        (rect.right - rect.left) as INT,
        (rect.bottom - rect.top) as INT,
        hwndDialog,
        UlongToPtr!(wID) as HMENU,
        hAppInstance,
        core::ptr::null_mut(),
    );

    /* sanity check */
    if hwnd.is_null() {
        _assert(
            b"hwnd\0" as *const u8 as *const i8,
            b"base/applications/sndvol32/dialog.c\0"
                as *const u8 as *const i8,
            179,
        );
    }

    /* store window */
    *OutWnd = hwnd;

    /* check if this the track bar */
    if _wcsicmp(ClassName as *const u16, RUSTCONV_WSTR_758288F5F048.as_ptr()) == 0 {
        if ((*DialogItem).style & (TBS_VERT as DWORD)) != 0 {
            /* Vertical trackbar: Volume */

            /* Disable the volume trackbar by default */
            EnableWindow(hwnd, FALSE as BOOL);

            /* set up range */
            SendMessage(
                hwnd,
                TBM_SETRANGE as UINT,
                (TRUE as WPARAM),
                (MAKELONG!(VOLUME_MIN as WORD, VOLUME_MAX as WORD)) as LPARAM,
            );

            /* set up page size */
            SendMessage(
                hwnd,
                TBM_SETPAGESIZE as UINT,
                0 as WPARAM,
                (VOLUME_PAGE_SIZE as LPARAM),
            );

            /* set position */
            SendMessage(
                hwnd,
                TBM_SETPOS as UINT,
                (TRUE as WPARAM),
                (0 as LPARAM),
            );

            /* Calculate and set ticks */
            nSteps = (VOLUME_MAX as INT)
                / ((VOLUME_TICKS as INT) + (1 as INT));
            if ((VOLUME_MAX as INT)
                % ((VOLUME_TICKS as INT) + (1 as INT)))
                != (0 as INT)
            {
                nSteps += 1;
            }
            i = nSteps;
            while i < (VOLUME_MAX as INT) {
                SendMessage(
                    hwnd,
                    TBM_SETTIC as UINT,
                    0 as WPARAM,
                    i as LPARAM,
                );
                i += nSteps;
            }
        } else {
            /* Horizontal trackbar: Balance */

            /* Disable the balance trackbar by default */
            EnableWindow(hwnd, FALSE as BOOL);

            /* set up range */
            SendMessage(
                hwnd,
                TBM_SETRANGE as UINT,
                (TRUE as WPARAM),
                (MAKELONG!(0 as WORD, BALANCE_STEPS as WORD)) as LPARAM,
            );

            /* set up page size */
            SendMessage(
                hwnd,
                TBM_SETPAGESIZE as UINT,
                0 as WPARAM,
                (BALANCE_PAGE_SIZE as LPARAM),
            );

            /* set position */
            SendMessage(
                hwnd,
                TBM_SETPOS as UINT,
                (TRUE as WPARAM),
                (BALANCE_STEPS as LPARAM) / (2 as LPARAM),
            );

            /* Calculate and set ticks */
            nSteps = (BALANCE_STEPS as INT)
                / ((BALANCE_TICKS as INT) + (1 as INT));
            if ((BALANCE_STEPS as INT)
                % ((BALANCE_TICKS as INT) + (1 as INT)))
                != (0 as INT)
            {
                nSteps += 1;
            }
            i = nSteps;
            while i < (BALANCE_STEPS as INT) {
                SendMessage(
                    hwnd,
                    TBM_SETTIC as UINT,
                    0 as WPARAM,
                    i as LPARAM,
                );
                i += nSteps;
            }
        }
    } else if _wcsicmp(ClassName as *const u16, RUSTCONV_WSTR_B3FF2A1B03AC.as_ptr()) != 0 {
        /* nothing: handled below */
    } else if _wcsicmp(ClassName as *const u16, RUSTCONV_WSTR_B3FF2A1B03AC.as_ptr()) == 0 {
        /* Set font */
        SendMessageW(
            hwnd,
            WM_SETFONT as UINT,
            (hFont as WPARAM),
            TRUE as LPARAM,
        );
    }
    if false {
    } else if _wcsicmp(ClassName as *const u16, RUSTCONV_WSTR_09DBAEEA59BF.as_ptr()) == 0 {
        if ((*DialogItem).id as i32) == (IDC_LINE_SWITCH as i32) {
            if MixerId == (PLAY_MIXER as UINT) {
                /* Disable checkboxes by default, if we are in play mode */
                EnableWindow(hwnd, FALSE as INT);
            }
        } else if ((*DialogItem).id as i32) == (IDC_LINE_ADVANCED as i32) {
            ShowWindow(hwnd, SW_HIDE as INT);
        }

        /* Set font */
        SendMessageW(
            hwnd,
            WM_SETFONT as UINT,
            (hFont as WPARAM),
            TRUE as LPARAM,
        );
    }

    /* check if there is additional data */
    if (*Offset as i32) == (0 as i32) {
        /* no additional data */
        Offset = Offset.offset(1);
    } else {
        /* FIXME: Determine whether this should be "Offset += 1 + *Offset" to explicitly skip the data count too. */
        /* skip past additional data */
        Offset = Offset.offset(*Offset as isize);
    }

    /* make sure next template is word-aligned */
    Offset = ((((Offset as ULONG_PTR) + (3 as ULONG_PTR))
        & !(3 as ULONG_PTR)) as LPWORD);

    /* done */
    return Offset;
}
#[no_mangle]
pub unsafe extern "C" fn LoadDialogControls(
    mut MixerWindow: PMIXER_WINDOW,
    mut DialogOffset: LPRECT,
    mut ItemCount: WORD,
    mut DialogItem: PDLGITEMTEMPLATE,
    mut DialogIdMultiplier: DWORD,
    mut xBaseUnit: UINT,
    mut yBaseUnit: UINT,
) {
    let mut Offset: LPWORD = core::ptr::null_mut();
    let mut Index: WORD = 0;

    /* sanity check */
    assert!(ItemCount != 0);

    if !(*MixerWindow).Window.is_null() {
        (*MixerWindow).Window = HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (*MixerWindow).Window as PVOID,
            (((*MixerWindow).WindowCount.wrapping_add(ItemCount as UINT)) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<HWND>() as SIZE_T),
        ) as *mut HWND;
    } else {
        (*MixerWindow).Window = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (ItemCount as SIZE_T).wrapping_mul(core::mem::size_of::<HWND>() as SIZE_T),
        ) as *mut HWND;
    }
    if (*MixerWindow).Window.is_null() {
        /* no memory */
        return;
    }

    /* enumerate now all controls */
    Index = 0;
    while (Index as core::ffi::c_int) < ItemCount as core::ffi::c_int {
        /* add controls */
        Offset = AddDialogControl(
            (*MixerWindow).hWnd,
            (*MixerWindow).Window.offset((*MixerWindow).WindowCount as isize) as *mut HWND,
            DialogOffset,
            DialogItem,
            DialogIdMultiplier,
            (*MixerWindow).hFont,
            xBaseUnit,
            yBaseUnit,
            (*MixerWindow).MixerId,
        );

        /* sanity check */
        assert!(!Offset.is_null());

        /* move dialog item to new offset */
        DialogItem = Offset as PDLGITEMTEMPLATE;

        /* increment window count */
        (*MixerWindow).WindowCount = (*MixerWindow).WindowCount.wrapping_add(1);
        Index = Index.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn LoadDialog(
    hModule: HMODULE,
    MixerWindow: PMIXER_WINDOW,
    DialogResId: LPCWSTR,
    Index: DWORD,
) {
    let mut DlgTemplate: LPDLGTEMPLATE = core::ptr::null_mut();
    let mut DlgItem: PDLGITEMTEMPLATE = core::ptr::null_mut();
    let mut dialogRect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut Offset: LPWORD = core::ptr::null_mut();
    let mut FontSize: WORD = 0;
    let mut FontName: [WCHAR; 100] = [0 as WCHAR; 100];
    let mut Length: SIZE_T = 0;
    let mut width: core::ffi::c_int = 0;

    let mut units: DWORD = (GetDialogBaseUnits()) as u32;
    let mut xBaseUnit: UINT = LOWORD!(units) as UINT;
    let mut yBaseUnit: UINT = HIWORD!(units) as UINT;

    /* first load the dialog resource */
    DlgTemplate = LoadDialogResource(hModule, DialogResId, core::ptr::null_mut()) as LPDLGTEMPLATE;
    if DlgTemplate.is_null() {
        /* failed to load resource */
        return;
    }

    /* Now walk past the dialog header */
    Offset = DlgTemplate.offset(1 as isize) as LPWORD;

    /* FIXME: support menu */
    assert!(*Offset == 0);
    Offset = Offset.offset(1);

    /* FIXME: support classes */
    assert!(*Offset == 0);
    Offset = Offset.offset(1);

    /* FIXME: support titles */
    assert!(*Offset == 0);
    Offset = Offset.offset(1);

    /* get font size */
    FontSize = *Offset;
    Offset = Offset.offset(1);

    /* calculate font length */
    Length = (wcslen(Offset as LPWSTR) + 1) as _;
    assert!(
        Length
            < (core::mem::size_of::<[WCHAR; 100]>()
                / core::mem::size_of::<WCHAR>()) as SIZE_T,
    );

    /* copy font */
    wcscpy(core::ptr::addr_of_mut!(FontName) as LPWSTR, Offset as LPWSTR);

    if ((*DlgTemplate).style & DS_SETFONT as DWORD) != 0 {
        let mut hDC: HDC;

        hDC = GetDC(0 as HWND);

        if (*MixerWindow).hFont.is_null() {
            let mut pixels: core::ffi::c_int = MulDiv(
                FontSize as core::ffi::c_int,
                GetDeviceCaps(hDC, (LOGPIXELSY as i32)),
                72,
            );
            (*MixerWindow).hFont = CreateFontW(
                -pixels,
                0,
                0,
                0,
                (FW_NORMAL as i32),
                FALSE,
                FALSE,
                FALSE,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                DEFAULT_QUALITY,
                FF_DONTCARE,
                core::ptr::addr_of_mut!(FontName) as *mut WCHAR as LPCWSTR,
            );
        }

        if !(*MixerWindow).hFont.is_null() {
            let mut charSize: SIZE = SIZE { cx: 0, cy: 0 };
            let mut hOldFont: HFONT;

            hOldFont = SelectObject(hDC, (*MixerWindow).hFont as HGDIOBJ) as HFONT;
            charSize.cx = GdiGetCharDimensions(
                hDC,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(charSize.cy),
            );
            if charSize.cx != 0 {
                xBaseUnit = charSize.cx as UINT;
                yBaseUnit = charSize.cy as UINT;
            }
            SelectObject(hDC, hOldFont as HGDIOBJ);

            (*MixerWindow).baseUnit.cx = charSize.cx;
            (*MixerWindow).baseUnit.cy = charSize.cy;
        }

        ReleaseDC(core::ptr::null_mut(), hDC);
    }

//    assert(MixerWindow->hFont);

    /* move offset after font name */
    Offset = Offset.offset(Length as isize);

    /* offset is now at first dialog item control */
    DlgItem = Offset as PDLGITEMTEMPLATE;

    dialogRect.left = 0;
    dialogRect.right = (*DlgTemplate).cx as LONG;
    dialogRect.top = 0;
    dialogRect.bottom = (*DlgTemplate).cy as LONG;

    ConvertRect(core::ptr::addr_of_mut!(dialogRect), xBaseUnit, yBaseUnit);

    width = (dialogRect.right - dialogRect.left) as core::ffi::c_int;

    dialogRect.left += (*MixerWindow).rect.right;
    dialogRect.right += (*MixerWindow).rect.right;
    dialogRect.top += (*MixerWindow).rect.top;
    dialogRect.bottom += (*MixerWindow).rect.top;

    (*MixerWindow).rect.right += width as LONG;
    if (dialogRect.bottom - dialogRect.top) > ((*MixerWindow).rect.bottom - (*MixerWindow).rect.top) {
        (*MixerWindow).rect.bottom = (*MixerWindow).rect.top + dialogRect.bottom - dialogRect.top;
    }

    /* now add the controls */
    LoadDialogControls(
        MixerWindow,
        core::ptr::addr_of_mut!(dialogRect),
        (*DlgTemplate).cdit,
        DlgItem,
        Index,
        xBaseUnit,
        yBaseUnit,
    );
}
#[no_mangle]
pub unsafe extern "system" fn EnumConnectionsCallback(
    Mixer: PSND_MIXER,
    LineID: DWORD,
    Line: LPMIXERLINE,
    Context: PVOID,
) -> BOOL {
    let mut LineName: [WCHAR; (MIXER_LONG_NAME_CHARS as usize)] = [0; (MIXER_LONG_NAME_CHARS as usize)];
    let mut Flags: DWORD = 0;
    let mut wID: DWORD = 0;
    let mut ControlCount: UINT = 0;
    let mut Index: DWORD = 0;
    let mut Control: LPMIXERCONTROL = core::ptr::null_mut();
    let mut hDlgCtrl: HWND = core::ptr::null_mut();
    let mut pVolumeDetails: PMIXERCONTROLDETAILS_UNSIGNED = core::ptr::null_mut();
    let mut PrefContext: PPREFERENCES_CONTEXT = Context as PPREFERENCES_CONTEXT;

    if (*Line).cControls == 0 {
        return (TRUE) as BOOL;
    }

    'done: {
        /* get line name */
        if SndMixerGetLineName(
            (*(*PrefContext).MixerWindow).Mixer as PSND_MIXER,
            (*PrefContext).SelectedLine,
            LineName.as_mut_ptr(),
            MIXER_LONG_NAME_CHARS,
            (TRUE) as BOOL,
        ) == -1
        {
            /* failed to get line name */
            LineName[0] = 0x0000u16;
        }

        pVolumeDetails = HeapAlloc(
            GetProcessHeap(),
            0,
            (*Line).cChannels as SIZE_T * (core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as SIZE_T),
        ) as PMIXERCONTROLDETAILS_UNSIGNED;
        if pVolumeDetails.is_null() {
            break 'done;
        }

        /* check if line is found in registry settings */
        if ReadLineConfig(
            core::ptr::addr_of_mut!((*PrefContext).DeviceName) as LPTSTR,
            LineName.as_mut_ptr(),
            core::ptr::addr_of_mut!((*Line).szName) as LPTSTR,
            core::ptr::addr_of_mut!(Flags),
        ) != 0
        {
            /* is it selected */
            if Flags != 0x4 {
                let mut dlgId: i32;

                if (*Line).dwComponentType == MIXERLINE_COMPONENTTYPE_DST_SPEAKERS
                    || (*Line).dwComponentType == MIXERLINE_COMPONENTTYPE_DST_HEADPHONES
                {
                    dlgId = if (*(*PrefContext).MixerWindow).Mode == SMALL_MODE {
                        (IDD_SMALL_MASTER as i32)
                    } else {
                        (IDD_NORMAL_MASTER as i32)
                    };
                } else {
                    dlgId = if (*(*PrefContext).MixerWindow).Mode == SMALL_MODE {
                        (IDD_SMALL_LINE as i32)
                    } else {
                        (IDD_NORMAL_LINE as i32)
                    };
                }

                /* load dialog resource */
                LoadDialog(
                    hAppInstance as HMODULE,
                    (*PrefContext).MixerWindow,
                    MAKEINTRESOURCE!(dlgId as WORD) as LPCWSTR,
                    (*(*PrefContext).MixerWindow).DialogCount,
                );

                /* get id */
                wID = (*(*PrefContext).MixerWindow)
                    .DialogCount
                    .wrapping_add(1)
                    .wrapping_mul(IDC_LINE_NAME);

                /* set line name */
                SetDlgItemTextW(
                    (*(*PrefContext).MixerWindow).hWnd,
                    wID as i32,
                    core::ptr::addr_of_mut!((*Line).szName) as LPCWSTR,
                );

                /* query controls */
                if (SndMixerQueryControls(
                    Mixer,
                    core::ptr::addr_of_mut!(ControlCount),
                    Line,
                    core::ptr::addr_of_mut!(Control),
                )) as u32 != FALSE
                {
                    /* now go through all controls and update their states */
                    Index = 0;
                    while Index < (*Line).cControls {
                        if (*Control.offset(Index as isize)).dwControlType
                            == MIXERCONTROL_CONTROLTYPE_MUTE
                        {
                            let mut Details: MIXERCONTROLDETAILS_BOOLEAN = core::mem::zeroed();

                            /* get volume control details */
                            if SndMixerGetVolumeControlDetails(
                                Mixer,
                                (*Control.offset(Index as isize)).dwControlID,
                                1,
                                (core::mem::size_of::<MIXERCONTROLDETAILS_BOOLEAN>()) as DWORD,
                                core::ptr::addr_of_mut!(Details) as LPVOID,
                            ) != -1
                            {
                                /* update dialog control */
                                wID = (*(*PrefContext).MixerWindow)
                                    .DialogCount
                                    .wrapping_add(1)
                                    .wrapping_mul(IDC_LINE_SWITCH);

                                /* get dialog control */
                                hDlgCtrl = GetDlgItem(
                                    (*(*PrefContext).MixerWindow).hWnd,
                                    wID as i32,
                                );

                                if !hDlgCtrl.is_null() {
                                    /* Enable the 'Mute' checkbox, if we are in play mode */
                                    if (*Mixer).MixerId == PLAY_MIXER {
                                        EnableWindow(hDlgCtrl, (TRUE) as BOOL);
                                    }

                                    /* check state */
                                    if SendMessageW(
                                        hDlgCtrl,
                                        BM_GETCHECK,
                                        0,
                                        0,
                                    ) != Details.fValue as LRESULT
                                    {
                                        /* update control state */
                                        SendMessageW(
                                            hDlgCtrl,
                                            BM_SETCHECK,
                                            Details.fValue as WPARAM,
                                            0,
                                        );
                                    }
                                }
                            }
                        } else if (*Control.offset(Index as isize)).dwControlType
                            == MIXERCONTROL_CONTROLTYPE_VOLUME
                        {
                            /* get volume control details */
                            if SndMixerGetVolumeControlDetails(
                                Mixer,
                                (*Control.offset(Index as isize)).dwControlID,
                                (*Line).cChannels,
                                (core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>()) as DWORD,
                                pVolumeDetails as LPVOID,
                            ) != -1
                            {
                                /* update dialog control */
                                let mut volumePosition: DWORD = 0;
                                let mut volumeStep: DWORD = 0;
                                let mut maxVolume: DWORD = 0;
                                let mut i: DWORD = 0;
                                let mut balancePosition: DWORD = 0;
                                let mut balanceStep: DWORD = 0;

                                volumeStep = (*Control.offset(Index as isize))
                                    .Bounds
                                    
                                    .__bindgen_anon_2.dwMaximum
                                    .wrapping_sub(
                                        (*Control.offset(Index as isize))
                                            .Bounds
                                            
                                            .__bindgen_anon_2.dwMinimum,
                                    )
                                    .wrapping_div(
                                        (VOLUME_MAX - VOLUME_MIN) as DWORD,
                                    );

                                maxVolume = 0;
                                i = 0;
                                while i < (*Line).cChannels {
                                    if (*pVolumeDetails.offset(i as isize)).dwValue > maxVolume {
                                        maxVolume = (*pVolumeDetails.offset(i as isize)).dwValue;
                                    }
                                    i = i.wrapping_add(1);
                                }

                                volumePosition = maxVolume
                                    .wrapping_sub(
                                        (*Control.offset(Index as isize))
                                            .Bounds
                                            
                                            .__bindgen_anon_2.dwMinimum,
                                    )
                                    .wrapping_div(volumeStep);

                                if (*Line).cChannels == 1 {
                                    balancePosition = BALANCE_CENTER;
                                } else if (*Line).cChannels == 2 {
                                    if (*pVolumeDetails.offset(0 as isize))
                                        .dwValue
                                        == (*pVolumeDetails.offset(1 as isize))
                                            .dwValue
                                    {
                                        balancePosition = BALANCE_CENTER;
                                    } else if (*pVolumeDetails
                                        .offset(0 as isize))
                                    .dwValue
                                        == (*Control.offset(Index as isize))
                                            .Bounds
                                            
                                            .__bindgen_anon_2.dwMinimum
                                    {
                                        balancePosition = BALANCE_RIGHT;
                                    } else if (*pVolumeDetails
                                        .offset(1 as isize))
                                    .dwValue
                                        == (*Control.offset(Index as isize))
                                            .Bounds
                                            
                                            .__bindgen_anon_2.dwMinimum
                                    {
                                        balancePosition = BALANCE_LEFT;
                                    } else {
                                        balanceStep = maxVolume
                                            .wrapping_sub(
                                                (*Control.offset(Index as isize))
                                                    .Bounds
                                                    
                                                    .__bindgen_anon_2.dwMinimum,
                                            )
                                            .wrapping_div(
                                                (BALANCE_STEPS / 2)
                                                    as DWORD,
                                            );

                                        if (*pVolumeDetails.offset(0 as isize))
                                            .dwValue
                                            < (*pVolumeDetails
                                                .offset(1 as isize))
                                            .dwValue
                                        {
                                            balancePosition = (*pVolumeDetails
                                                .offset(0 as isize))
                                            .dwValue
                                            .wrapping_sub(
                                                (*Control.offset(Index as isize))
                                                    .Bounds
                                                    
                                                    .__bindgen_anon_2.dwMinimum,
                                            )
                                            .wrapping_div(balanceStep);
                                            balancePosition = (BALANCE_RIGHT as DWORD)
                                                .wrapping_sub(balancePosition);
                                        } else if (*pVolumeDetails
                                            .offset(1 as isize))
                                        .dwValue
                                            < (*pVolumeDetails
                                                .offset(0 as isize))
                                            .dwValue
                                        {
                                            balancePosition = (*pVolumeDetails
                                                .offset(1 as isize))
                                            .dwValue
                                            .wrapping_sub(
                                                (*Control.offset(Index as isize))
                                                    .Bounds
                                                    
                                                    .__bindgen_anon_2.dwMinimum,
                                            )
                                            .wrapping_div(balanceStep);
                                            balancePosition = (BALANCE_LEFT as DWORD)
                                                .wrapping_add(balancePosition);
                                        }
                                    }
                                }

                                /* Set the volume trackbar */
                                wID = (*(*PrefContext).MixerWindow)
                                    .DialogCount
                                    .wrapping_add(1)
                                    .wrapping_mul(IDC_LINE_SLIDER_VERT);

                                /* get dialog control */
                                hDlgCtrl = GetDlgItem(
                                    (*(*PrefContext).MixerWindow).hWnd,
                                    wID as i32,
                                );

                                if !hDlgCtrl.is_null() {
                                    /* check state */
                                    let mut OldPosition: LRESULT = SendMessageW(
                                        hDlgCtrl,
                                        TBM_GETPOS,
                                        0,
                                        0,
                                    );

                                    /* Enable the volume trackbar */
                                    EnableWindow(hDlgCtrl, (TRUE) as BOOL);

                                    if OldPosition
                                        != (VOLUME_MAX as DWORD)
                                            .wrapping_sub(volumePosition)
                                            as LRESULT
                                    {
                                        /* update control state */
                                        SendMessageW(
                                            hDlgCtrl,
                                            TBM_SETPOS,
                                            TRUE as WPARAM,
                                            (VOLUME_MAX as DWORD)
                                                .wrapping_sub(volumePosition)
                                                as LPARAM,
                                        );
                                    }
                                }

                                if (*Line).cChannels == 2 {
                                    /* Set the balance trackbar */
                                    wID = (*(*PrefContext).MixerWindow)
                                        .DialogCount
                                        .wrapping_add(1)
                                        .wrapping_mul(IDC_LINE_SLIDER_HORZ);

                                    /* get dialog control */
                                    hDlgCtrl = GetDlgItem(
                                        (*(*PrefContext).MixerWindow).hWnd,
                                        wID as i32,
                                    );

                                    if !hDlgCtrl.is_null() {
                                        /* check state */
                                        let mut OldPosition_0: LRESULT = SendMessageW(
                                            hDlgCtrl,
                                            TBM_GETPOS,
                                            0,
                                            0,
                                        );

                                        /* Enable the balance trackbar */
                                        EnableWindow(hDlgCtrl, (TRUE) as BOOL);

                                        if OldPosition_0 != balancePosition as LRESULT {
                                            /* update control state */
                                            SendMessageW(
                                                hDlgCtrl,
                                                TBM_SETPOS,
                                                TRUE as WPARAM,
                                                balancePosition as LPARAM,
                                            );
                                        }
                                    }
                                }
                            }
                        } else {
                            if (*(*PrefContext).MixerWindow).Mode == NORMAL_MODE {
                                (*(*PrefContext).MixerWindow).bHasExtendedControls =
                                    (TRUE) as i32;

                                wID = (*(*PrefContext).MixerWindow)
                                    .DialogCount
                                    .wrapping_add(1)
                                    .wrapping_mul(IDC_LINE_ADVANCED);

                                /* get dialog control */
                                hDlgCtrl = GetDlgItem(
                                    (*(*PrefContext).MixerWindow).hWnd,
                                    wID as i32,
                                );
                                if !hDlgCtrl.is_null() {
                                    ShowWindow(
                                        hDlgCtrl,
                                        if (*(*PrefContext).MixerWindow).bShowExtendedControls != 0 {
                                            (SW_SHOWNORMAL) as i32
                                        } else {
                                            (SW_HIDE) as i32
                                        },
                                    );
                                }
                            }
                        }
                        Index = Index.wrapping_add(1);
                    }

                    /* free controls */
                    HeapFree(
                        GetProcessHeap(),
                        0,
                        Control as PVOID,
                    );
                }

                /* increment dialog count */
                (*(*PrefContext).MixerWindow).DialogCount =
                    (*(*PrefContext).MixerWindow).DialogCount.wrapping_add(1);
            }
        }
    }

    /* Free the volume details */
    if !pVolumeDetails.is_null() {
        HeapFree(
            GetProcessHeap(),
            0,
            pVolumeDetails as PVOID,
        );
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn LoadDialogCtrls(PrefContext: PPREFERENCES_CONTEXT) {
    let mut szBuffer: [WCHAR; 64] = [0; 64];
    let mut hDlgCtrl: HWND = core::ptr::null_mut();
    let mut statusRect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut i: UINT = 0;
    let mut dy: LONG = 0;

    /* set dialog count to zero */
    (*(*PrefContext).MixerWindow).DialogCount = 0 as UINT;
    (*(*PrefContext).MixerWindow).bHasExtendedControls = FALSE as BOOL;
    SetRectEmpty(core::ptr::addr_of_mut!((*(*PrefContext).MixerWindow).rect));

    /* enumerate controls */
    SndMixerEnumConnections(
        (*(*PrefContext).MixerWindow).Mixer as PSND_MIXER,
        (*PrefContext).SelectedLine,
        Some(
            EnumConnectionsCallback,
        ),
        PrefContext as PVOID,
    );

    /* Update the 'Advanced Controls' menu item */
    EnableMenuItem(
        GetMenu((*(*PrefContext).MixerWindow).hWnd),
        IDM_ADVANCED_CONTROLS as UINT,
        (MF_BYCOMMAND
            | (if (*(*PrefContext).MixerWindow).bHasExtendedControls != 0 {
                MF_ENABLED
            } else {
                MF_GRAYED
            })) as UINT,
    );

    /* Add some height for the status bar */
    if !(*(*PrefContext).MixerWindow).hStatusBar.is_null() {
        GetWindowRect(
            (*(*PrefContext).MixerWindow).hStatusBar,
            core::ptr::addr_of_mut!(statusRect),
        );
        (*(*PrefContext).MixerWindow).rect.bottom += statusRect.bottom - statusRect.top;
    }

    /* Add height of the 'Advanced' button */
    dy = MulDiv(
        ADVANCED_BUTTON_HEIGHT as ::core::ffi::c_int,
        (*(*PrefContext).MixerWindow).baseUnit.cy as ::core::ffi::c_int,
        8 as ::core::ffi::c_int,
    ) as LONG;
    if (*(*PrefContext).MixerWindow).bShowExtendedControls != 0
        && (*(*PrefContext).MixerWindow).bHasExtendedControls != 0
    {
        (*(*PrefContext).MixerWindow).rect.bottom += dy;
    }

    /* now move the window */
    AdjustWindowRect(
        core::ptr::addr_of_mut!((*(*PrefContext).MixerWindow).rect),
        (WS_DLGFRAME
            | WS_CAPTION
            | WS_MINIMIZEBOX
            | WS_SYSMENU
            | WS_CLIPCHILDREN
            | WS_CLIPSIBLINGS
            | WS_VISIBLE) as DWORD,
        TRUE as BOOL,
    );
    SetWindowPos(
        (*(*PrefContext).MixerWindow).hWnd,
        HWND_TOP as HWND,
        (*(*PrefContext).MixerWindow).rect.left as ::core::ffi::c_int,
        (*(*PrefContext).MixerWindow).rect.top as ::core::ffi::c_int,
        ((*(*PrefContext).MixerWindow).rect.right - (*(*PrefContext).MixerWindow).rect.left)
            as ::core::ffi::c_int,
        ((*(*PrefContext).MixerWindow).rect.bottom - (*(*PrefContext).MixerWindow).rect.top)
            as ::core::ffi::c_int,
        (SWP_NOMOVE | SWP_NOZORDER) as UINT,
    );

    /* Move the status bar */
    if !(*(*PrefContext).MixerWindow).hStatusBar.is_null() {
        SetWindowPos(
            (*(*PrefContext).MixerWindow).hStatusBar,
            HWND_TOP as HWND,
            statusRect.left as ::core::ffi::c_int,
            ((*(*PrefContext).MixerWindow).rect.bottom - (statusRect.bottom - statusRect.top))
                as ::core::ffi::c_int,
            ((*(*PrefContext).MixerWindow).rect.right - (*(*PrefContext).MixerWindow).rect.left)
                as ::core::ffi::c_int,
            (statusRect.bottom - statusRect.top) as ::core::ffi::c_int,
            SWP_NOZORDER as UINT,
        );
    }

    if (*(*PrefContext).MixerWindow).MixerId == RECORD_MIXER as UINT {
        LoadStringW(
            hAppInstance,
            IDS_SELECT as UINT,
            core::ptr::addr_of_mut!(szBuffer) as LPWSTR,
            ARRAYSIZE!(szBuffer) as ::core::ffi::c_int,
        );
    }

    i = 0 as UINT;
    while i < (*(*PrefContext).MixerWindow).DialogCount {
        if (*(*PrefContext).MixerWindow).MixerId == RECORD_MIXER as UINT {
            hDlgCtrl = GetDlgItem(
                (*(*PrefContext).MixerWindow).hWnd,
                (i.wrapping_add(1 as UINT))
                    .wrapping_mul(IDC_LINE_SWITCH as UINT) as ::core::ffi::c_int,
            );

            /* Turn the autocheckbox into a checkbox */
            SetWindowLongPtr(
                hDlgCtrl,
                GWL_STYLE as ::core::ffi::c_int,
                (GetWindowLongPtr(hDlgCtrl, GWL_STYLE as ::core::ffi::c_int)
                    & !(BS_AUTOCHECKBOX as LONG_PTR))
                    | (BS_CHECKBOX as LONG_PTR),
            );

            /* Change text from 'Mute' to 'Select' */
            SetWindowTextW(
                hDlgCtrl,
                core::ptr::addr_of_mut!(szBuffer) as LPCWSTR,
            );
        }

        /* Resize the vertical line separator */
        hDlgCtrl = GetDlgItem(
            (*(*PrefContext).MixerWindow).hWnd,
            (i.wrapping_add(1 as UINT))
                .wrapping_mul(IDC_LINE_SEP as UINT) as ::core::ffi::c_int,
        );
        if !hDlgCtrl.is_null() {
            GetWindowRect(hDlgCtrl, core::ptr::addr_of_mut!(statusRect));
            if (*(*PrefContext).MixerWindow).bShowExtendedControls != 0
                && (*(*PrefContext).MixerWindow).bHasExtendedControls != 0
            {
                statusRect.bottom += dy;
            }

            SetWindowPos(
                hDlgCtrl,
                HWND_TOP as HWND,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (statusRect.right - statusRect.left) as ::core::ffi::c_int,
                (statusRect.bottom - statusRect.top) as ::core::ffi::c_int,
                (SWP_NOMOVE | SWP_NOZORDER) as UINT,
            );
        }
        i = i.wrapping_add(1);
    }

    /* Hide the last line separator */
    hDlgCtrl = GetDlgItem(
        (*(*PrefContext).MixerWindow).hWnd,
        (IDC_LINE_SEP as UINT).wrapping_mul((*(*PrefContext).MixerWindow).DialogCount)
            as ::core::ffi::c_int,
    );
    if !hDlgCtrl.is_null() {
        ShowWindow(hDlgCtrl, SW_HIDE as ::core::ffi::c_int);
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetLineDialogControl(
    mut PrefContext: PPREFERENCES_CONTEXT,
    mut Line: LPMIXERLINE,
    mut dwDialogID: DWORD,
) -> HWND {
    let mut Index: DWORD = 0;
    Index = 0 as DWORD;
    while Index < (*(*PrefContext).MixerWindow).DialogCount as DWORD {
        let mut LineName: [WCHAR; (MIXER_LONG_NAME_CHARS as usize)] = [0; (MIXER_LONG_NAME_CHARS as usize)];
        let mut wID: DWORD = Index
            .wrapping_add(1 as DWORD)
            .wrapping_mul(IDC_LINE_NAME as DWORD);
        if !(GetDlgItemText(
            (*(*PrefContext).MixerWindow).hWnd,
            wID as ::core::ffi::c_int,
            core::ptr::addr_of_mut!(LineName) as LPWSTR,
            MIXER_LONG_NAME_CHARS as ::core::ffi::c_int,
        ) == 0 as UINT)
        {
            if _wcsicmp(
                core::ptr::addr_of_mut!(LineName) as *mut WCHAR,
                core::ptr::addr_of_mut!((*Line).szName) as *mut WCHAR,
            ) == 0
            {
                wID = Index
                    .wrapping_add(1 as DWORD)
                    .wrapping_mul(dwDialogID);
                return GetDlgItem(
                    (*(*PrefContext).MixerWindow).hWnd,
                    wID as ::core::ffi::c_int,
                );
            }
        }
        Index = Index.wrapping_add(1);
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn UpdateDialogLineSwitchControl(mut PrefContext: PPREFERENCES_CONTEXT, mut Line: LPMIXERLINE, mut fValue: LONG) {
    let mut hDlgCtrl: HWND = GetLineDialogControl(PrefContext, Line, IDC_LINE_SWITCH as DWORD);
    if !hDlgCtrl.is_null()
        && SendMessageW(hDlgCtrl, BM_GETCHECK as UINT, 0 as WPARAM, 0 as LPARAM) as LONG
            != fValue
    {
        SendMessageW(hDlgCtrl, BM_SETCHECK as UINT, fValue as WPARAM, 0 as LPARAM);
    }
}
#[no_mangle]
pub unsafe extern "C" fn GetDialogLineSliderCurrentPosition(
    mut PrefContext: PPREFERENCES_CONTEXT,
    mut Line: LPMIXERLINE,
    mut dwDialogID: DWORD,
) -> DWORD {
    let mut hDlgCtrl: HWND = GetLineDialogControl(PrefContext, Line, dwDialogID);

    return (if !hDlgCtrl.is_null() {
        SendMessageW(hDlgCtrl, TBM_GETPOS as UINT, 0 as WPARAM, 0 as LPARAM)
    } else {
        0 as LRESULT
    }) as DWORD;
}
#[no_mangle]
pub unsafe extern "C" fn UpdateDialogLineSliderControl(mut PrefContext: PPREFERENCES_CONTEXT, mut Line: LPMIXERLINE, mut dwDialogID: DWORD, mut Position: DWORD) {
    let mut hDlgCtrl: HWND = GetLineDialogControl(PrefContext, Line, dwDialogID);
    let mut isPositionChanged: BOOL =
        (GetDialogLineSliderCurrentPosition(PrefContext, Line, dwDialogID) != Position) as BOOL;
    if !hDlgCtrl.is_null() && isPositionChanged != 0 {
        SendMessageW(
            hDlgCtrl,
            TBM_SETPOS as UINT,
            TRUE as WPARAM,
            Position as LPARAM,
        );
    }
}
