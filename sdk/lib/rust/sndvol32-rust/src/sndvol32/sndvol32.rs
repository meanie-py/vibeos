use crate::sndvol32::advanced::AdvancedDlgProc;
use crate::sndvol32::misc::AllocAndLoadString;
use crate::sndvol32::misc::CloseAppConfig;
use crate::sndvol32::misc::GetStyleValue;
use crate::sndvol32::misc::InitAppConfig;
use crate::sndvol32::dialog::LoadDialogCtrls;
use crate::sndvol32::misc::LoadXYCoordWnd;
use crate::sndvol32::misc::ReadLineConfig;
use crate::sndvol32::misc::SaveXYCoordWnd;
use crate::sndvol32::mixer::SndMixerCreate;
use crate::sndvol32::mixer::SndMixerDestroy;
use crate::sndvol32::mixer::SndMixerEnumConnections;
use crate::sndvol32::mixer::SndMixerEnumLines;
use crate::sndvol32::mixer::SndMixerEnumProducts;
use crate::sndvol32::mixer::SndMixerGetLineByName;
use crate::sndvol32::mixer::SndMixerGetLineName;
use crate::sndvol32::mixer::SndMixerGetProductName;
use crate::sndvol32::mixer::SndMixerGetSelection;
use crate::sndvol32::mixer::SndMixerGetVolumeControlDetails;
use crate::sndvol32::mixer::SndMixerQueryControls;
use crate::sndvol32::mixer::SndMixerSelect;
use crate::sndvol32::mixer::SndMixerSetVolumeControlDetails;
use crate::sndvol32::tray::TrayDlgProc;
use crate::sndvol32::dialog::UpdateDialogLineSliderControl;
use crate::sndvol32::dialog::UpdateDialogLineSwitchControl;
use crate::sndvol32::misc::WriteLineConfig;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hAppInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut MainWindowClass: ATOM = 0;

#[no_mangle]
pub static mut hMainWnd: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hAppHeap: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut lpAppTitle: LPTSTR = ::core::ptr::null::<WCHAR>() as *mut WCHAR;

#[no_mangle]
pub static mut Preferences: PREFERENCES_CONTEXT = _PREFERENCES_CONTEXT {
    MixerWindow: ::core::ptr::null_mut::<_MIXER_WINDOW>(),
    Mixer: ::core::ptr::null_mut::<_SND_MIXER>(),
    hwndDlg: ::core::ptr::null_mut::<HWND__>(),
    Selected: 0,
    SelectedLine: 0,
    PlaybackID: 0,
    RecordingID: 0,
    OtherLines: 0,
    DeviceName: [0; 128],
    tmp: 0,
};

#[repr(C)]
pub struct PREFERENCES_FILL_DEVICES {
    pub PrefContext: PPREFERENCES_CONTEXT,
    pub hComboBox: HWND,
    pub Selected: UINT,
}

pub type PPREFERENCES_FILL_DEVICES = *mut PREFERENCES_FILL_DEVICES;

unsafe extern "system" fn FillDeviceComboBox(
    mut Mixer: PSND_MIXER,
    mut Id: UINT,
    mut ProductName: LPCTSTR,
    mut Context: PVOID,
) -> BOOL {
    let mut lres: LRESULT = 0;
    let mut FillContext: PPREFERENCES_FILL_DEVICES = Context as PPREFERENCES_FILL_DEVICES;

    UNREFERENCED_PARAMETER!(Mixer);

    lres = SendMessage(
        (*FillContext).hComboBox,
        CB_ADDSTRING as UINT,
        0 as WPARAM,
        ProductName as LPARAM,
    );
    if lres != CB_ERR as LRESULT {
        /* save the index so we don't screw stuff when the combobox is sorted... */
        SendMessage(
            (*FillContext).hComboBox,
            CB_SETITEMDATA as UINT,
            lres as WPARAM,
            Id as LPARAM,
        );

        if Id == (*FillContext).Selected {
            SendMessage(
                (*FillContext).hComboBox,
                CB_SETCURSEL as UINT,
                lres as WPARAM,
                0 as LPARAM,
            );
        }
    }

    return TRUE as BOOL;
}

unsafe extern "system" fn PrefDlgAddLine(
    Mixer: PSND_MIXER,
    Line: LPMIXERLINE,
    DisplayControls: UINT,
    Context: PVOID,
) -> BOOL {
    let PrefContext: PPREFERENCES_CONTEXT = Context as PPREFERENCES_CONTEXT;
    let mut lres: LRESULT = 0;
    let mut hwndCbOthers: HWND = core::ptr::null_mut();
    let mut goto_others: bool = false;

    UNREFERENCED_PARAMETER!(Mixer);
    UNREFERENCED_PARAMETER!(DisplayControls);

    match (*Line).dwComponentType {
        MIXERLINE_COMPONENTTYPE_DST_SPEAKERS => {
            if (*PrefContext).PlaybackID == (0 as DWORD).wrapping_sub(1 as DWORD) {
                (*PrefContext).PlaybackID = (*Line).dwLineID;

                if (*PrefContext).SelectedLine == (0 as DWORD).wrapping_sub(1 as DWORD) {
                    (*PrefContext).SelectedLine = (*Line).dwLineID;
                }
            } else {
                goto_others = true;
            }

            // break;
        }

        MIXERLINE_COMPONENTTYPE_DST_WAVEIN => {
            if (*PrefContext).RecordingID == (0 as DWORD).wrapping_sub(1 as DWORD) {
                (*PrefContext).RecordingID = (*Line).dwLineID;

                if (*PrefContext).SelectedLine == (0 as DWORD).wrapping_sub(1 as DWORD) {
                    (*PrefContext).SelectedLine = (*Line).dwLineID;
                }
            } else {
                goto_others = true;
            }

            // break;
        }

        _ => {
            if (*PrefContext).SelectedLine == (0 as DWORD).wrapping_sub(1 as DWORD) {
                (*PrefContext).SelectedLine = (*Line).dwLineID;
            }

            goto_others = true;
            // fall through to AddToOthersLines
        }
    }

    // AddToOthersLines:
    if goto_others {
        hwndCbOthers = GetDlgItem(
            (*PrefContext).hwndDlg,
            IDC_LINE as core::ffi::c_int,
        );

        lres = SendMessage(
            hwndCbOthers,
            CB_ADDSTRING as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!((*Line).szName) as LPARAM,
        );
        if lres != CB_ERR as LRESULT {
            SendMessage(
                hwndCbOthers,
                CB_SETITEMDATA as UINT,
                lres as WPARAM,
                (*Line).dwLineID as LPARAM,
            );

            (*PrefContext).OtherLines = (*PrefContext).OtherLines.wrapping_add(1);
        }
    }

    return TRUE as BOOL;
}

unsafe extern "system" fn PrefDlgAddConnection(
    Mixer: PSND_MIXER,
    LineID: DWORD,
    Line: LPMIXERLINE,
    Context: PVOID,
) -> BOOL {
    let PrefContext: PPREFERENCES_CONTEXT = Context as PPREFERENCES_CONTEXT;
    let mut hwndControls: HWND = core::ptr::null_mut();
    let mut lvi: LVITEM = core::mem::zeroed();
    let mut i: UINT = 0;

    UNREFERENCED_PARAMETER!(Mixer);
    UNREFERENCED_PARAMETER!(LineID);

    if (*Line).cControls != 0 {
        hwndControls = GetDlgItem((*PrefContext).hwndDlg, (IDC_CONTROLS as i32));

        lvi.mask = LVIF_TEXT | LVIF_PARAM;
        let fresh_tmp = (*PrefContext).tmp;
        (*PrefContext).tmp = fresh_tmp.wrapping_add(1);
        lvi.iItem = fresh_tmp as _;
        lvi.iSubItem = 0;
        lvi.pszText = core::ptr::addr_of_mut!((*Line).szName) as *mut WCHAR as LPWSTR;
        lvi.lParam = (*Line).dwSource as LPARAM;

        i = SendMessage(
            hwndControls,
            (LVM_INSERTITEM as u32),
            0,
            core::ptr::addr_of_mut!(lvi) as LPARAM,
        ) as UINT;
        if i != (-(1 as i32)) as UINT {
            let mut LineName: [TCHAR; (MIXER_LONG_NAME_CHARS as usize)] = [0; (MIXER_LONG_NAME_CHARS as usize)];
            let mut Flags: DWORD = 0;
            let mut SelLine: BOOL = (FALSE) as i32;

            if SndMixerGetLineName(
                (*PrefContext).Mixer,
                (*PrefContext).SelectedLine,
                core::ptr::addr_of_mut!(LineName) as LPTSTR,
                MIXER_LONG_NAME_CHARS,
                (TRUE) as BOOL,
            ) == -(1 as i32)
            {
                LineName[0 as usize] = ('\0' as u16);
            }

            if ReadLineConfig(
                core::ptr::addr_of_mut!((*PrefContext).DeviceName) as LPTSTR,
                core::ptr::addr_of_mut!(LineName) as LPTSTR,
                core::ptr::addr_of_mut!((*Line).szName) as LPTSTR,
                &mut Flags,
            ) != 0
            {
                if Flags != 0x4 {
                    SelLine = (TRUE) as i32;
                }
            }

            ListView_SetCheckState!(hwndControls, i, SelLine);
        }
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn UpdatePrefDlgControls(Context: PPREFERENCES_CONTEXT, mut LineID: DWORD) {
    let mut OldID: INT;
    let mut MixerID: INT = 0 as INT;
    let mut DeviceCbIndex: INT;

    /* select the mixer */
    DeviceCbIndex = SendDlgItemMessage(
        (*Context).hwndDlg,
        IDC_MIXERDEVICE as ::core::ffi::c_int,
        CB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as INT;
    if DeviceCbIndex != CB_ERR as INT {
        MixerID = SendDlgItemMessage(
            (*Context).hwndDlg,
            IDC_MIXERDEVICE as ::core::ffi::c_int,
            CB_GETITEMDATA as UINT,
            DeviceCbIndex as WPARAM,
            0 as LPARAM,
        ) as INT;
        if MixerID == CB_ERR as INT {
            MixerID = 0 as INT;
        }
    }

    OldID = (*Context).Selected as INT;
    if MixerID != OldID && SndMixerSelect((*Context).Mixer, MixerID as UINT) != 0 {
        (*Context).Selected = SndMixerGetSelection((*Context).Mixer);

        /* update the controls */
        (*Context).PlaybackID = -(1 as ::core::ffi::c_int) as DWORD;
        (*Context).RecordingID = -(1 as ::core::ffi::c_int) as DWORD;
        (*Context).OtherLines = 0 as UINT;
        (*Context).SelectedLine = -(1 as ::core::ffi::c_int) as DWORD;

        let _ = 0;
        SndMixerGetProductName(
            (*Context).Mixer,
            core::ptr::addr_of_mut!((*Context).DeviceName) as LPTSTR,
            (::core::mem::size_of::<[TCHAR; 128]>() as usize)
                .wrapping_div(::core::mem::size_of::<TCHAR>() as usize) as UINT,
        );

        if SndMixerEnumLines(
            (*Context).Mixer,
            Some(
                PrefDlgAddLine,
            ),
            Context as PVOID,
        ) != 0
        {
            let mut SelBox: UINT = 0 as UINT;

            /* enable/disable controls and make default selection */
            EnableWindow(
                GetDlgItem((*Context).hwndDlg, IDC_PLAYBACK as ::core::ffi::c_int),
                ((*Context).PlaybackID != -(1 as ::core::ffi::c_int) as DWORD)
                    as ::core::ffi::c_int,
            );
            CheckDlgButton(
                (*Context).hwndDlg,
                IDC_PLAYBACK as ::core::ffi::c_int,
                (if (*Context).PlaybackID != -(1 as ::core::ffi::c_int) as DWORD && {
                    let fresh15 = SelBox;
                    SelBox = SelBox.wrapping_add(1);
                    fresh15 == 0 as UINT
                } {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );

            EnableWindow(
                GetDlgItem((*Context).hwndDlg, IDC_RECORDING as ::core::ffi::c_int),
                ((*Context).RecordingID != -(1 as ::core::ffi::c_int) as DWORD)
                    as ::core::ffi::c_int,
            );
            CheckDlgButton(
                (*Context).hwndDlg,
                IDC_RECORDING as ::core::ffi::c_int,
                (if (*Context).RecordingID != -(1 as ::core::ffi::c_int) as DWORD && {
                    let fresh16 = SelBox;
                    SelBox = SelBox.wrapping_add(1);
                    fresh16 == 0 as UINT
                } {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );

            if (*Context).OtherLines != 0 as UINT {
                /* select the first item in the other lines combo box by default */
                SendDlgItemMessage(
                    (*Context).hwndDlg,
                    IDC_LINE as ::core::ffi::c_int,
                    CB_SETCURSEL as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                );
            }
            EnableWindow(
                GetDlgItem((*Context).hwndDlg, IDC_LINE as ::core::ffi::c_int),
                FALSE as ::core::ffi::c_int,
            );
            EnableWindow(
                GetDlgItem((*Context).hwndDlg, IDC_OTHER as ::core::ffi::c_int),
                ((*Context).OtherLines != 0 as UINT) as ::core::ffi::c_int,
            );
            CheckDlgButton(
                (*Context).hwndDlg,
                IDC_LINE as ::core::ffi::c_int,
                (if (*Context).OtherLines != 0 as UINT && {
                    let fresh17 = SelBox;
                    SelBox = SelBox.wrapping_add(1);
                    fresh17 == 0 as UINT
                } {
                    BST_CHECKED
                } else {
                    BST_UNCHECKED
                }) as UINT,
            );

            /* disable the OK button if the device doesn't have any lines */
            EnableWindow(
                GetDlgItem((*Context).hwndDlg, IDOK as ::core::ffi::c_int),
                ((*Context).PlaybackID != -(1 as ::core::ffi::c_int) as DWORD
                    || (*Context).RecordingID != -(1 as ::core::ffi::c_int) as DWORD
                    || (*Context).OtherLines != 0 as UINT) as ::core::ffi::c_int,
            );

            LineID = (*Context).SelectedLine;
        }
    }

    /* update the line sources list */
    if (MixerID != OldID
        && (*Context).SelectedLine != -(1 as ::core::ffi::c_int) as DWORD)
        || ((*Context).SelectedLine != LineID
            && LineID != -(1 as ::core::ffi::c_int) as DWORD)
    {
        (*Context).SelectedLine = LineID;

        let _ = ListView_DeleteAllItems!(GetDlgItem(
            (*Context).hwndDlg,
            IDC_CONTROLS as ::core::ffi::c_int,
        ));

        (*Context).tmp = 0 as DWORD;
        SndMixerEnumConnections(
            (*Context).Mixer,
            LineID,
            Some(
                PrefDlgAddConnection,
            ),
            Context as PVOID,
        );
    }
}

unsafe extern "C" fn WriteLineSettings(Context: PPREFERENCES_CONTEXT, hwndDlg: HWND) {
    let mut hwndControls: HWND;
    let mut Count: INT;
    let mut Index: INT;
    let mut LineName: [WCHAR; (MIXER_LONG_NAME_CHARS as usize)] = [0; (MIXER_LONG_NAME_CHARS as usize)];
    let mut DestinationName: [WCHAR; (MIXER_LONG_NAME_CHARS as usize)] = [0; (MIXER_LONG_NAME_CHARS as usize)];
    let mut Flags: DWORD;
    let mut LineStates: PSNDVOL_REG_LINESTATE;

    /* get list view */
    hwndControls = GetDlgItem(hwndDlg, (IDC_CONTROLS as i32));

    /* get list item count */
    Count = ListView_GetItemCount!(hwndControls);

    /* sanity check */
    assert!(Count != 0);

    if SndMixerGetLineName(
        (*Context).Mixer,
        (*Context).SelectedLine,
        core::ptr::addr_of_mut!(DestinationName) as LPTSTR,
        MIXER_LONG_NAME_CHARS,
        (TRUE) as BOOL,
    ) == -1
    {
        /* failed to get destination line name */
        return;
    }

    /* allocate line states array */
    LineStates = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (core::mem::size_of::<SNDVOL_REG_LINESTATE>() as SIZE_T).wrapping_mul(Count as SIZE_T),
    ) as PSNDVOL_REG_LINESTATE;
    if LineStates.is_null() {
        /* failed to allocate line states array */
        return;
    }

    Index = 0;
    while Index < Count {
        /* set to empty */
        LineName[0 as usize] = 0x0000u16;

        /* get item text */
        ListView_GetItemText!(
            hwndControls,
            Index,
            0,
            core::ptr::addr_of_mut!(LineName) as LPTSTR,
            MIXER_LONG_NAME_CHARS,
        );

        /* make sure it is null terminated */
        LineName[(MIXER_LONG_NAME_CHARS - 1) as usize] = 0x0000u16;

        /* get check state */
        Flags = (if ListView_GetCheckState!(hwndControls, Index) == 0 {
            0x4
        } else {
            0
        }) as DWORD;

        /* copy line name */
        wcscpy(
            (*LineStates.offset(Index as isize)).LineName.as_mut_ptr(),
            core::ptr::addr_of_mut!(LineName) as *mut WCHAR,
        );

        /* store flags */
        (*LineStates.offset(Index as isize)).Flags = Flags;

        Index += 1;
    }

    /* now write the line config */
    WriteLineConfig(
        core::ptr::addr_of_mut!((*Context).DeviceName) as LPTSTR,
        core::ptr::addr_of_mut!(DestinationName) as LPTSTR,
        LineStates,
        (core::mem::size_of::<SNDVOL_REG_LINESTATE>() as usize).wrapping_mul(Count as usize) as DWORD,
    );

    /* free line states */
    HeapFree(GetProcessHeap(), 0, LineStates as PVOID);
}

unsafe extern "system" fn DlgPreferencesProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut Context: PPREFERENCES_CONTEXT = core::ptr::null_mut();

    match uMsg {
        WM_COMMAND => {
            Context = GetDialogData!(hwndDlg, PPREFERENCES_CONTEXT);
            match (LOWORD!(wParam)) as u32 {
                IDC_MIXERDEVICE => {
                    if (HIWORD!(wParam)) as u32 == CBN_SELCHANGE {
                        UpdatePrefDlgControls(Context, (-(1 as INT)) as DWORD);
                    }
                }

                IDC_LINE => {
                    if (HIWORD!(wParam)) as u32 == CBN_SELCHANGE {
                        let mut LineID: INT = 0;
                        let mut Index: INT = 0;

                        Index = SendDlgItemMessage(
                            hwndDlg,
                            (IDC_LINE as i32),
                            CB_GETCURSEL,
                            0,
                            0,
                        ) as INT;
                        if Index != CB_ERR {
                            LineID = SendDlgItemMessage(
                                hwndDlg,
                                (IDC_LINE as i32),
                                CB_GETITEMDATA,
                                Index as WPARAM,
                                0,
                            ) as INT;
                            if LineID != CB_ERR {
                                UpdatePrefDlgControls(Context, LineID as DWORD);
                            }
                        }
                    }
                }

                IDC_PLAYBACK => {
                    UpdatePrefDlgControls(Context, (*Context).PlaybackID);
                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_LINE as i32)),
                        (FALSE) as BOOL,
                    );
                }

                IDC_RECORDING => {
                    UpdatePrefDlgControls(Context, (*Context).RecordingID);
                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_LINE as i32)),
                        (FALSE) as BOOL,
                    );
                }

                IDC_OTHER => {
                    let mut LineCbIndex: INT = 0;
                    let mut LineID: INT = 0;

                    EnableWindow(
                        GetDlgItem(hwndDlg, (IDC_LINE as i32)),
                        (TRUE) as BOOL,
                    );

                    LineCbIndex = SendDlgItemMessage(
                        hwndDlg,
                        (IDC_LINE as i32),
                        CB_GETCURSEL,
                        0,
                        0,
                    ) as INT;
                    if LineCbIndex != CB_ERR {
                        LineID = SendDlgItemMessage(
                            hwndDlg,
                            (IDC_LINE as i32),
                            CB_GETITEMDATA,
                            LineCbIndex as WPARAM,
                            0,
                        ) as INT;
                        if LineID != CB_ERR {
                            UpdatePrefDlgControls(Context, LineID as DWORD);
                        }
                    }
                }

                IDOK => {
                    /* write line settings */
                    WriteLineSettings(Context, hwndDlg);

                    /* fall through */
                    EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
                }
                IDCANCEL => {
                    EndDialog(hwndDlg, (LOWORD!(wParam)) as INT_PTR);
                }
                _ => {}
            }
        }

        WM_INITDIALOG => {
            let mut FillDevContext: PREFERENCES_FILL_DEVICES = core::mem::zeroed();
            let mut lvc: LVCOLUMN = core::mem::zeroed();
            let mut rcClient: RECT = core::mem::zeroed();
            let mut hwndControls: HWND = core::ptr::null_mut();

            SetWindowLongPtr(hwndDlg, DWLP_USER, lParam as LONG_PTR);
            Context = (lParam as LONG_PTR) as PPREFERENCES_CONTEXT;
            (*Context).hwndDlg = hwndDlg;
            (*Context).Mixer = SndMixerCreate(hwndDlg, (*(*Context).MixerWindow).MixerId);
            (*Context).Selected = (-(1 as INT)) as UINT;

            FillDevContext.PrefContext = Context;
            FillDevContext.hComboBox = GetDlgItem(hwndDlg, (IDC_MIXERDEVICE as i32));
            FillDevContext.Selected = SndMixerGetSelection((*Context).Mixer);
            SndMixerEnumProducts(
                (*Context).Mixer,
                Some(
                    FillDeviceComboBox,
                ),
                core::ptr::addr_of_mut!(FillDevContext) as PVOID,
            );

            /* initialize the list view control */
            hwndControls = GetDlgItem(hwndDlg, (IDC_CONTROLS as i32));
            let _ = ListView_SetExtendedListViewStyle!(hwndControls, LVS_EX_CHECKBOXES);

            GetClientRect(hwndControls, core::ptr::addr_of_mut!(rcClient));
            lvc.mask = (LVCF_TEXT | LVCF_WIDTH) as UINT;
            lvc.pszText = (&[0x0000u16]).as_ptr() as *mut u16 /* TEXT "" */;
            lvc.cx = rcClient.right;
            SendMessage(
                hwndControls,
                (LVM_INSERTCOLUMN as u32),
                0,
                core::ptr::addr_of_mut!(lvc) as LPARAM,
            );

            /* update all controls */
            UpdatePrefDlgControls(Context, (*Context).SelectedLine as DWORD);
            return (TRUE) as INT_PTR;
        }

        WM_CLOSE => {
            EndDialog(hwndDlg, (IDCANCEL) as INT_PTR);
        }

        WM_SYSCOLORCHANGE => {
            let mut hwndControls: HWND = core::ptr::null_mut();

            /* Forward WM_SYSCOLORCHANGE */
            hwndControls = GetDlgItem(hwndDlg, (IDC_CONTROLS as i32));
            SendMessage(
                hwndControls,
                WM_SYSCOLORCHANGE,
                0,
                0,
            );
        }

        _ => {}
    }

    return 0;
}

unsafe extern "C" fn DeleteMixerWindowControls(MixerWindow: PMIXER_WINDOW) {
    let mut Index: DWORD;
    Index = 0 as DWORD;
    while Index < (*MixerWindow).WindowCount as DWORD {
        /* destroys the window */
        DestroyWindow((*MixerWindow).Window.offset(Index as isize).read());
        Index = Index.wrapping_add(1);
    }

    /* free memory */
    HeapFree(GetProcessHeap(), 0 as DWORD, (*MixerWindow).Window as PVOID);

    /* set to null */
    (*MixerWindow).Window = core::ptr::null_mut();
    (*MixerWindow).WindowCount = 0 as UINT;
}

unsafe extern "C" fn RebuildMixerWindowControls(PrefContext: PPREFERENCES_CONTEXT) -> BOOL {
    /* delete existing mixer controls */
    DeleteMixerWindowControls((*PrefContext).MixerWindow);

    /* load new mixer controls */
    LoadDialogCtrls(PrefContext);

    return (TRUE) as BOOL;
}

unsafe extern "system" fn SetVolumeCallback(
    mut Mixer: PSND_MIXER,
    mut LineID: DWORD,
    mut Line: LPMIXERLINE,
    mut Ctx: PVOID,
) -> BOOL {
    let mut ControlCount: UINT = 0 as UINT;
    let mut Index: UINT = 0;
    let mut Control: LPMIXERCONTROL = core::ptr::null_mut::<MIXERCONTROLW>();
    let mut puDetails: PMIXERCONTROLDETAILS_UNSIGNED =
        core::ptr::null_mut::<tMIXERCONTROLDETAILS_UNSIGNED>();
    let mut bDetails: MIXERCONTROLDETAILS_BOOLEAN = MIXERCONTROLDETAILS_BOOLEAN { fValue: 0 };
    let mut Context: PSET_VOLUME_CONTEXT = Ctx as PSET_VOLUME_CONTEXT;

    /* check if the line name is equal */
    if _wcsicmp(
        core::ptr::addr_of_mut!((*Line).szName) as *mut WCHAR,
        core::ptr::addr_of_mut!((*Context).LineName) as *mut WCHAR,
    ) != 0
    {
        /* it is not */
        return (TRUE) as BOOL;
    }

    /* query controls */
    if (SndMixerQueryControls(
        Mixer,
        core::ptr::addr_of_mut!(ControlCount),
        Line,
        core::ptr::addr_of_mut!(Control),
    )) as u32 == FALSE
    {
        /* failed to query for controls */
        return (FALSE) as BOOL;
    }

    puDetails = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ((*Line).cChannels as SIZE_T)
            .wrapping_mul(core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as SIZE_T),
    ) as PMIXERCONTROLDETAILS_UNSIGNED;
    if puDetails.is_null() {
        return (FALSE) as BOOL;
    }

    /* now go through all controls and compare control ids */
    Index = 0 as UINT;
    while Index < ControlCount {
        if (*Context).bVertical != 0 {
            if (*Control.offset(Index as isize)).dwControlType
                == MIXERCONTROL_CONTROLTYPE_VOLUME as DWORD
            {
                let mut LineOffset: DWORD = 0;
                let mut volumePosition: DWORD = 0;
                let mut balancePosition: DWORD = 0;
                let mut volumeStep: DWORD = 0;
                let mut balanceStep: DWORD = 0;

                LineOffset = (*Context).SliderPos as DWORD;

                volumePosition = SendDlgItemMessage(
                    (*Preferences.MixerWindow).hWnd,
                    LineOffset.wrapping_mul(IDC_LINE_SLIDER_VERT as DWORD) as ::core::ffi::c_int,
                    TBM_GETPOS as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                ) as DWORD;
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

                if (*Line).cChannels == 1 as DWORD {
                    /* set up details */
                    (*puDetails.offset(0 as isize)).dwValue =
                        (VOLUME_MAX as DWORD)
                            .wrapping_sub(volumePosition)
                            .wrapping_mul(volumeStep)
                            .wrapping_add(
                                (*Control.offset(Index as isize))
                                    .Bounds
                                    
                                    .__bindgen_anon_2.dwMinimum,
                            );
                } else if (*Line).cChannels == 2 as DWORD {
                    balancePosition = SendDlgItemMessage(
                        (*Preferences.MixerWindow).hWnd,
                        LineOffset.wrapping_mul(IDC_LINE_SLIDER_HORZ as DWORD)
                            as ::core::ffi::c_int,
                        TBM_GETPOS as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) as DWORD;
                    if balancePosition == BALANCE_CENTER as DWORD {
                        (*puDetails.offset(0 as isize)).dwValue =
                            (VOLUME_MAX as DWORD)
                                .wrapping_sub(volumePosition)
                                .wrapping_mul(volumeStep)
                                .wrapping_add(
                                    (*Control.offset(Index as isize))
                                        .Bounds
                                        
                                        .__bindgen_anon_2.dwMinimum,
                                );
                        (*puDetails.offset(1 as isize)).dwValue =
                            (VOLUME_MAX as DWORD)
                                .wrapping_sub(volumePosition)
                                .wrapping_mul(volumeStep)
                                .wrapping_add(
                                    (*Control.offset(Index as isize))
                                        .Bounds
                                        
                                        .__bindgen_anon_2.dwMinimum,
                                );
                    } else if balancePosition == BALANCE_LEFT as DWORD {
                        (*puDetails.offset(0 as isize)).dwValue =
                            (VOLUME_MAX as DWORD)
                                .wrapping_sub(volumePosition)
                                .wrapping_mul(volumeStep)
                                .wrapping_add(
                                    (*Control.offset(Index as isize))
                                        .Bounds
                                        
                                        .__bindgen_anon_2.dwMinimum,
                                );
                        (*puDetails.offset(1 as isize)).dwValue =
                            (*Control.offset(Index as isize))
                                .Bounds
                                
                                .__bindgen_anon_2.dwMinimum;
                    } else if balancePosition == BALANCE_RIGHT as DWORD {
                        (*puDetails.offset(0 as isize)).dwValue =
                            (*Control.offset(Index as isize))
                                .Bounds
                                
                                .__bindgen_anon_2.dwMinimum;
                        (*puDetails.offset(1 as isize)).dwValue =
                            (VOLUME_MAX as DWORD)
                                .wrapping_sub(volumePosition)
                                .wrapping_mul(volumeStep)
                                .wrapping_add(
                                    (*Control.offset(Index as isize))
                                        .Bounds
                                        
                                        .__bindgen_anon_2.dwMinimum,
                                );
                    } else if balancePosition < BALANCE_CENTER as DWORD {
                        /* Left */
                        (*puDetails.offset(0 as isize)).dwValue =
                            (VOLUME_MAX as DWORD)
                                .wrapping_sub(volumePosition)
                                .wrapping_mul(volumeStep)
                                .wrapping_add(
                                    (*Control.offset(Index as isize))
                                        .Bounds
                                        
                                        .__bindgen_anon_2.dwMinimum,
                                );

                        balanceStep = (*puDetails.offset(0 as isize))
                            .dwValue
                            .wrapping_sub(
                                (*Control.offset(Index as isize))
                                    .Bounds
                                    
                                    .__bindgen_anon_2.dwMinimum,
                            )
                            .wrapping_div(
                                (BALANCE_STEPS / 2) as DWORD,
                            );

                        (*puDetails.offset(1 as isize)).dwValue =
                            balancePosition.wrapping_mul(balanceStep).wrapping_add(
                                (*Control.offset(Index as isize))
                                    .Bounds
                                    
                                    .__bindgen_anon_2.dwMinimum,
                            );
                    } else if balancePosition > BALANCE_CENTER as DWORD {
                        /* Right */
                        (*puDetails.offset(1 as isize)).dwValue =
                            (VOLUME_MAX as DWORD)
                                .wrapping_sub(volumePosition)
                                .wrapping_mul(volumeStep)
                                .wrapping_add(
                                    (*Control.offset(Index as isize))
                                        .Bounds
                                        
                                        .__bindgen_anon_2.dwMinimum,
                                );

                        balanceStep = (*puDetails.offset(1 as isize))
                            .dwValue
                            .wrapping_sub(
                                (*Control.offset(Index as isize))
                                    .Bounds
                                    
                                    .__bindgen_anon_2.dwMinimum,
                            )
                            .wrapping_div(
                                (BALANCE_STEPS / 2) as DWORD,
                            );

                        (*puDetails.offset(0 as isize)).dwValue =
                            (BALANCE_RIGHT as DWORD)
                                .wrapping_sub(balancePosition)
                                .wrapping_mul(balanceStep)
                                .wrapping_add(
                                    (*Control.offset(Index as isize))
                                        .Bounds
                                        
                                        .__bindgen_anon_2.dwMinimum,
                                );
                    }
                } else {
                    SndMixerGetVolumeControlDetails(
                        (*Preferences.MixerWindow).Mixer as PSND_MIXER,
                        (*Control.offset(Index as isize)).dwControlID,
                        (*Line).cChannels,
                        core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD,
                        puDetails as LPVOID,
                    );

                    /* FIXME */
                }

                /* set volume */
                SndMixerSetVolumeControlDetails(
                    (*Preferences.MixerWindow).Mixer as PSND_MIXER,
                    (*Control.offset(Index as isize)).dwControlID,
                    (*Line).cChannels,
                    core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD,
                    puDetails as LPVOID,
                );

                /* done */
                break;
            }
        } else if (*Context).bSwitch != 0 {
            if (*Control.offset(Index as isize)).dwControlType
                == MIXERCONTROL_CONTROLTYPE_MUTE as DWORD
            {
                /* set up details */
                bDetails.fValue = (*Context).SliderPos as LONG;

                /* set volume */
                SndMixerSetVolumeControlDetails(
                    (*Preferences.MixerWindow).Mixer as PSND_MIXER,
                    (*Control.offset(Index as isize)).dwControlID,
                    1 as DWORD,
                    core::mem::size_of::<MIXERCONTROLDETAILS_BOOLEAN>() as DWORD,
                    core::ptr::addr_of_mut!(bDetails) as LPVOID,
                );

                /* done */
                break;
            }
        }
        Index = Index.wrapping_add(1);
    }

    if !puDetails.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            puDetails as PVOID,
        );
    }

    /* free controls */
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        Control as PVOID,
    );

    /* done */
    return (TRUE) as BOOL;
}

unsafe extern "system" fn MixerControlChangeCallback(Mixer: PSND_MIXER, LineID: DWORD, Line: LPMIXERLINE, Context: PVOID) -> BOOL {
    let mut pVolumeDetails: PMIXERCONTROLDETAILS_UNSIGNED = core::ptr::null_mut();
    let mut ControlCount: UINT = 0 as UINT;
    let mut Index: UINT;
    let mut Control: LPMIXERCONTROL = core::ptr::null_mut();

    /* check if the line has controls */
    if (*Line).cControls == 0 as DWORD {
        /* no controls */
        return (TRUE) as BOOL;
    }

    /* query controls */
    if (SndMixerQueryControls(Mixer, &raw mut ControlCount, Line, &raw mut Control)) as u32 == FALSE {
        /* failed to query for controls */
        return (FALSE) as BOOL;
    }

    pVolumeDetails = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ((*Line).cChannels as SIZE_T)
            .wrapping_mul(core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as SIZE_T),
    ) as PMIXERCONTROLDETAILS_UNSIGNED;

    'done: {
        if pVolumeDetails.is_null() {
            break 'done;
        }

        /* now go through all controls and compare control ids */
        Index = 0 as UINT;
        while Index < ControlCount {
            if (*Control.offset(Index as isize)).dwControlID
                == PtrToUlong!(Context as *const ::core::ffi::c_void)
            {
                if (*Control.offset(Index as isize)).dwControlType == MIXERCONTROL_CONTROLTYPE_MUTE as DWORD {
                    let mut Details: MIXERCONTROLDETAILS_BOOLEAN = core::mem::zeroed();

                    /* get volume control details */
                    if SndMixerGetVolumeControlDetails(
                        (*Preferences.MixerWindow).Mixer as PSND_MIXER,
                        (*Control.offset(Index as isize)).dwControlID,
                        1 as DWORD,
                        core::mem::size_of::<MIXERCONTROLDETAILS_BOOLEAN>() as DWORD,
                        &raw mut Details as LPVOID,
                    ) != -(1 as ::core::ffi::c_int)
                    {
                        /* update dialog control */
                        UpdateDialogLineSwitchControl(&raw mut Preferences, Line, Details.fValue);
                    }
                } else if (*Control.offset(Index as isize)).dwControlType == MIXERCONTROL_CONTROLTYPE_VOLUME as DWORD {
                    /* get volume control details */
                    if SndMixerGetVolumeControlDetails(
                        (*Preferences.MixerWindow).Mixer as PSND_MIXER,
                        (*Control.offset(Index as isize)).dwControlID,
                        (*Line).cChannels,
                        core::mem::size_of::<MIXERCONTROLDETAILS_UNSIGNED>() as DWORD,
                        pVolumeDetails as LPVOID,
                    ) != -(1 as ::core::ffi::c_int)
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

                        maxVolume = 0 as DWORD;
                        i = 0 as DWORD;
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

                        if (*Line).cChannels == 1 as DWORD {
                            balancePosition = BALANCE_CENTER as DWORD;
                        } else if (*Line).cChannels == 2 as DWORD {
                            if (*pVolumeDetails.offset(0 as isize)).dwValue
                                == (*pVolumeDetails.offset(1 as isize)).dwValue
                            {
                                balancePosition = BALANCE_CENTER as DWORD;
                            } else if (*pVolumeDetails.offset(0 as isize)).dwValue
                                == (*Control.offset(Index as isize))
                                    .Bounds
                                    
                                    .__bindgen_anon_2.dwMinimum
                            {
                                balancePosition = BALANCE_RIGHT as DWORD;
                            } else if (*pVolumeDetails.offset(1 as isize)).dwValue
                                == (*Control.offset(Index as isize))
                                    .Bounds
                                    
                                    .__bindgen_anon_2.dwMinimum
                            {
                                balancePosition = BALANCE_LEFT as DWORD;
                            } else {
                                balanceStep = maxVolume
                                    .wrapping_sub(
                                        (*Control.offset(Index as isize))
                                            .Bounds
                                            
                                            .__bindgen_anon_2.dwMinimum,
                                    )
                                    .wrapping_div(
                                        (BALANCE_STEPS / 2) as DWORD,
                                    );

                                if (*pVolumeDetails.offset(0 as isize)).dwValue
                                    < (*pVolumeDetails.offset(1 as isize)).dwValue
                                {
                                    balancePosition = (*pVolumeDetails.offset(0 as isize))
                                        .dwValue
                                        .wrapping_sub(
                                            (*Control.offset(Index as isize))
                                                .Bounds
                                                
                                                .__bindgen_anon_2.dwMinimum,
                                        )
                                        .wrapping_div(balanceStep);
                                    balancePosition =
                                        (BALANCE_RIGHT as DWORD).wrapping_sub(balancePosition);
                                } else if (*pVolumeDetails.offset(1 as isize)).dwValue
                                    < (*pVolumeDetails.offset(0 as isize)).dwValue
                                {
                                    balancePosition = (*pVolumeDetails.offset(1 as isize))
                                        .dwValue
                                        .wrapping_sub(
                                            (*Control.offset(Index as isize))
                                                .Bounds
                                                
                                                .__bindgen_anon_2.dwMinimum,
                                        )
                                        .wrapping_div(balanceStep);
                                    balancePosition =
                                        (BALANCE_LEFT as DWORD).wrapping_add(balancePosition);
                                }
                            }
                        }

                        /* Update the volume control slider */
                        UpdateDialogLineSliderControl(
                            &raw mut Preferences,
                            Line,
                            IDC_LINE_SLIDER_VERT as DWORD,
                            (VOLUME_MAX as DWORD).wrapping_sub(volumePosition),
                        );

                        /* Update the balance control slider */
                        UpdateDialogLineSliderControl(
                            &raw mut Preferences,
                            Line,
                            IDC_LINE_SLIDER_HORZ as DWORD,
                            balancePosition,
                        );
                    }
                }
                break;
            } else {
                Index = Index.wrapping_add(1);
            }
        }
    }

    /* done */
    if !pVolumeDetails.is_null() {
        /* Free the volume details */
        HeapFree(GetProcessHeap(), 0 as DWORD, pVolumeDetails as PVOID);
    }

    /* free controls */
    HeapFree(GetProcessHeap(), 0 as DWORD, Control as PVOID);

    /* done */
    return (TRUE) as BOOL;
}

unsafe extern "system" fn MainWindowProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut MixerWindow: PMIXER_WINDOW = core::ptr::null_mut();
    let mut CtrlID: DWORD = 0;
    let mut LineOffset: DWORD = 0;
    let mut TargetSlider: DWORD = 0;
    let mut bRet: BOOL = 0;
    let mut Result: LRESULT = 0;
    let mut Context: SET_VOLUME_CONTEXT = core::mem::zeroed();

    match uMsg {
        WM_COMMAND => {
            MixerWindow = GetWindowData!(hwnd, PMIXER_WINDOW);

            match (LOWORD!(wParam)) as u32 {
                IDM_PROPERTIES => {
                    let mut Pref: PREFERENCES_CONTEXT = core::mem::zeroed();

                    Pref.MixerWindow = MixerWindow;
                    Pref.Mixer = core::ptr::null_mut();
                    Pref.SelectedLine = Preferences.SelectedLine;

                    if DialogBoxParam(
                        hAppInstance,
                        MAKEINTRESOURCE!(IDD_PREFERENCES),
                        hwnd,
                        Some(DlgPreferencesProc),
                        core::ptr::addr_of_mut!(Pref) as LPARAM,
                    ) == IDOK as INT_PTR
                    {
                        /* update window */
                        let mut szProduct: [TCHAR; (MAXPNAMELEN as usize)] = core::mem::zeroed();

                        /* get mixer product name */
                        if SndMixerGetProductName(
                            Pref.Mixer,
                            core::ptr::addr_of_mut!(szProduct) as LPTSTR,
                            (core::mem::size_of::<[TCHAR; (MAXPNAMELEN as usize)]>()
                                / core::mem::size_of_val(&szProduct[0]))
                                as UINT,
                        ) == -1
                        {
                            /* failed to get name */
                            szProduct[0] = 0x0000u16;
                        } else {
                            /* copy product */
                            wcscpy(
                                core::ptr::addr_of_mut!(Preferences.DeviceName) as *mut TCHAR,
                                core::ptr::addr_of_mut!(szProduct) as *mut TCHAR,
                            );
                        }

                        /* destroy old status bar */
                        if (*MixerWindow).Mode == NORMAL_MODE {
                            DestroyWindow((*MixerWindow).hStatusBar);
                        }

                        /* update details */
                        Preferences.SelectedLine = Pref.SelectedLine;

                        /* destroy old mixer */
                        SndMixerDestroy((*Preferences.MixerWindow).Mixer as PSND_MIXER);

                        /* use new selected mixer */
                        (*Preferences.MixerWindow).Mixer = Pref.Mixer;

                        /* create status window */
                        if (*MixerWindow).Mode == NORMAL_MODE {
                            (*MixerWindow).hStatusBar = CreateStatusWindow(
                                (WS_VISIBLE | WS_CHILD | WS_CLIPSIBLINGS) as i32,
                                core::ptr::null_mut(),
                                hwnd,
                                0,
                            );
                            if !(*MixerWindow).hStatusBar.is_null() {
                                /* Set status bar */
                                SendMessage(
                                    (*MixerWindow).hStatusBar,
                                    WM_SETTEXT,
                                    0,
                                    core::ptr::addr_of_mut!(szProduct) as LPARAM,
                                );
                            }
                        }

                        /* rebuild dialog controls */
                        RebuildMixerWindowControls(core::ptr::addr_of_mut!(Preferences));
                    }
                }

                IDM_ADVANCED_CONTROLS => {
                    (*MixerWindow).bShowExtendedControls =
                        ((*MixerWindow).bShowExtendedControls == 0) as BOOL;
                    CheckMenuItem(
                        GetMenu(hwnd),
                        IDM_ADVANCED_CONTROLS,
                        MF_BYCOMMAND
                            | (if (*MixerWindow).bShowExtendedControls != 0 {
                                MF_CHECKED
                            } else {
                                MF_UNCHECKED
                            }),
                    );
                    RebuildMixerWindowControls(core::ptr::addr_of_mut!(Preferences));
                }

                IDM_EXIT => {
                    PostQuitMessage(0);
                    UnregisterHotKey(hwnd, (HOTKEY_CTRL_S as i32));
                }

                IDM_ABOUT => {
                    let mut hAppIcon: HICON =
                        GetClassLongPtrW(hwnd, GCLP_HICON) as HICON;
                    ShellAbout(
                        hwnd,
                        lpAppTitle,
                        core::ptr::null_mut(),
                        hAppIcon,
                    );
                }

                _ => {
                    /* get button id */
                    CtrlID = LOWORD!(wParam) as DWORD;

                    /* check if the message is from the line switch */
                    if (HIWORD!(wParam)) as u32 == BN_CLICKED {
                        if CtrlID.wrapping_rem(IDC_LINE_SWITCH) == 0 {
                            /* compute line offset */
                            LineOffset = CtrlID.wrapping_div(IDC_LINE_SWITCH);

                            /* compute window id of line name static control */
                            CtrlID = LineOffset.wrapping_mul(IDC_LINE_NAME);

                            if (*(*Preferences.MixerWindow).Mixer).MixerId == PLAY_MIXER {
                                /* get line name */
                                if GetDlgItemTextW(
                                    hwnd,
                                    CtrlID as ::core::ffi::c_int,
                                    core::ptr::addr_of_mut!(Context.LineName) as LPTSTR,
                                    (MIXER_LONG_NAME_CHARS as i32),
                                ) != 0
                                {
                                    /* setup context */
                                    Context.SliderPos = SendMessage(
                                        lParam as HWND,
                                        BM_GETCHECK,
                                        0,
                                        0,
                                    ) as UINT;
                                    Context.bVertical = (FALSE) as i32;
                                    Context.bSwitch = (TRUE) as i32;

                                    /* set volume */
                                    SndMixerEnumConnections(
                                        (*Preferences.MixerWindow).Mixer as PSND_MIXER,
                                        Preferences.SelectedLine,
                                        Some(SetVolumeCallback),
                                        core::ptr::addr_of_mut!(Context) as PVOID,
                                    );
                                }
                            } else if (*(*Preferences.MixerWindow).Mixer).MixerId == RECORD_MIXER {
                                let mut i: UINT = 0;

                                i = 0;
                                while i < (*Preferences.MixerWindow).DialogCount {
                                    SendDlgItemMessageW(
                                        hwnd,
                                        (i.wrapping_add(1)).wrapping_mul(IDC_LINE_SWITCH)
                                            as ::core::ffi::c_int,
                                        BM_SETCHECK,
                                        (i.wrapping_add(1) == LineOffset) as WPARAM,
                                        0,
                                    );
                                    i = i.wrapping_add(1);
                                }
                            }
                        } else if CtrlID.wrapping_rem(IDC_LINE_ADVANCED) == 0 {
                            let mut AdvancedContext: ADVANCED_CONTEXT = core::mem::zeroed();

                            /* compute line offset */
                            LineOffset = CtrlID.wrapping_div(IDC_LINE_ADVANCED);

                            /* compute window id of line name static control */
                            CtrlID = LineOffset.wrapping_mul(IDC_LINE_NAME);

                            /* get line name */
                            if GetDlgItemTextW(
                                hwnd,
                                CtrlID as ::core::ffi::c_int,
                                core::ptr::addr_of_mut!(AdvancedContext.LineName) as LPTSTR,
                                (MIXER_LONG_NAME_CHARS as i32),
                            ) != 0
                            {
                                AdvancedContext.MixerWindow = Preferences.MixerWindow;
                                AdvancedContext.Mixer = (*Preferences.MixerWindow).Mixer as PSND_MIXER;
                                AdvancedContext.Line = SndMixerGetLineByName(
                                    (*Preferences.MixerWindow).Mixer as PSND_MIXER,
                                    Preferences.SelectedLine,
                                    core::ptr::addr_of_mut!(AdvancedContext.LineName) as LPTSTR,
                                );
                                if !AdvancedContext.Line.is_null() {
                                    DialogBoxParam(
                                        hAppInstance,
                                        MAKEINTRESOURCE!(IDD_ADVANCED),
                                        hwnd,
                                        Some(AdvancedDlgProc),
                                        core::ptr::addr_of_mut!(AdvancedContext) as LPARAM,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        MM_MIXM_LINE_CHANGE => {
            DPRINT!(b"MM_MIXM_LINE_CHANGE\n\0".as_ptr() as *const core::ffi::c_char);
        }

        MM_MIXM_CONTROL_CHANGE => {
            DPRINT!(b"MM_MIXM_CONTROL_CHANGE\n\0".as_ptr() as *const core::ffi::c_char);

            /* get mixer window */
            MixerWindow = GetWindowData!(hwnd, PMIXER_WINDOW);

            /* sanity checks */
            assert!(!MixerWindow.is_null());
            assert!((*(*MixerWindow).Mixer).hmx == wParam as HMIXER);

            SndMixerEnumConnections(
                (*MixerWindow).Mixer,
                Preferences.SelectedLine,
                Some(MixerControlChangeCallback),
                lParam as PVOID,
            );
        }

        WM_HSCROLL | WM_VSCROLL => {
            TargetSlider = if uMsg == WM_HSCROLL {
                IDC_LINE_SLIDER_HORZ
            } else {
                IDC_LINE_SLIDER_VERT
            };

            match (LOWORD!(wParam)) as u32 {
                TB_LINEUP
                | TB_LINEDOWN
                | TB_PAGEUP
                | TB_PAGEDOWN
                | TB_TOP
                | TB_BOTTOM
                | TB_THUMBTRACK => {
                    /* get dialog item ctrl */
                    CtrlID = GetDlgCtrlID(lParam as HWND) as DWORD;

                    /* get line index */
                    LineOffset = CtrlID.wrapping_div(TargetSlider);

                    /* compute window id of line name static control */
                    CtrlID = LineOffset.wrapping_mul(IDC_LINE_NAME);

                    /* get line name */
                    if GetDlgItemTextW(
                        hwnd,
                        CtrlID as ::core::ffi::c_int,
                        core::ptr::addr_of_mut!(Context.LineName) as LPTSTR,
                        (MIXER_LONG_NAME_CHARS as i32),
                    ) != 0
                    {
                        /* setup context */
                        Context.SliderPos = LineOffset as UINT;
                        Context.bVertical = (TRUE) as i32;
                        Context.bSwitch = (FALSE) as i32;

                        /* set volume */
                        SndMixerEnumConnections(
                            (*Preferences.MixerWindow).Mixer as PSND_MIXER,
                            Preferences.SelectedLine,
                            Some(SetVolumeCallback),
                            core::ptr::addr_of_mut!(Context) as PVOID,
                        );
                    }
                }

                _ => {}
            }

            /* One more time to test if we released the slider or was there any other affection */
            match (LOWORD!(wParam)) as u32 {
                TB_LINEUP
                | TB_LINEDOWN
                | TB_PAGEUP
                | TB_PAGEDOWN
                | TB_TOP
                | TB_BOTTOM
                | TB_ENDTRACK => {
                    MixerWindow = GetWindowData!(hwnd, PMIXER_WINDOW);

                    /* get dialog item ctrl */
                    CtrlID = GetDlgCtrlID(lParam as HWND) as DWORD;

                    /* get line index */
                    LineOffset = CtrlID.wrapping_div(TargetSlider);

                    if LineOffset == 1
                        && (*(*MixerWindow).Mixer).MixerId == 0
                    {
                        PlaySound(
                            SND_ALIAS_SYSTEMDEFAULT as LPCTSTR,
                            core::ptr::null_mut(),
                            (SND_ASYNC | SND_ALIAS_ID) as DWORD,
                        );
                    }
                }

                _ => {}
            }
        }

        WM_CREATE => {
            MixerWindow = (*(lParam as LPCREATESTRUCT)).lpCreateParams as PMIXER_WINDOW;
            SetWindowLongPtr(
                hwnd,
                GWL_USERDATA,
                MixerWindow as LONG_PTR,
            );
            (*MixerWindow).hWnd = hwnd;
            (*MixerWindow).Mixer = SndMixerCreate((*MixerWindow).hWnd, (*MixerWindow).MixerId);
            if !(*MixerWindow).Mixer.is_null() {
                let mut szProduct: [TCHAR; (MAXPNAMELEN as usize)] = core::mem::zeroed();

                /* get mixer product name */
                if SndMixerGetProductName(
                    (*MixerWindow).Mixer,
                    core::ptr::addr_of_mut!(szProduct) as LPTSTR,
                    (core::mem::size_of::<[TCHAR; (MAXPNAMELEN as usize)]>()
                        / core::mem::size_of_val(&szProduct[0]))
                        as UINT,
                ) == -1
                {
                    /* failed to get name */
                    szProduct[0] = 0x0000u16;
                }

                /* initialize preferences */
                ZeroMemory!(
                    core::ptr::addr_of_mut!(Preferences) as PVOID,
                    core::mem::size_of::<PREFERENCES_CONTEXT>(),
                );

                /* store mixer */
                Preferences.Mixer = (*MixerWindow).Mixer as PSND_MIXER;

                /* store mixer window */
                Preferences.MixerWindow = MixerWindow;

                /* first destination line id */
                Preferences.SelectedLine = 0xFFFF0000;

                /* copy product */
                wcscpy(
                    core::ptr::addr_of_mut!(Preferences.DeviceName) as *mut TCHAR,
                    core::ptr::addr_of_mut!(szProduct) as *mut TCHAR,
                );

                /* Disable the 'Advanced Controls' menu item */
                EnableMenuItem(
                    GetMenu(hwnd),
                    IDM_ADVANCED_CONTROLS,
                    MF_BYCOMMAND | MF_GRAYED,
                );

                /* Load the placement coordinate data of the window */
                bRet = LoadXYCoordWnd(core::ptr::addr_of_mut!(Preferences));
                if bRet != 0 {
                    /*
                     * LoadXYCoordWnd() might fail for the first time of opening the application which is normal as
                     * the Sound Control's applet settings haven't been saved yet to the Registry. At this point SetWindowPos()
                     * call is skipped.
                     */
                    SetWindowPos(
                        hwnd,
                        core::ptr::null_mut(),
                        (*MixerWindow).WndPosX as ::core::ffi::c_int,
                        (*MixerWindow).WndPosY as ::core::ffi::c_int,
                        0,
                        0,
                        SWP_NOSIZE | SWP_NOZORDER,
                    );
                }

                /* create status window */
                if (*MixerWindow).Mode == NORMAL_MODE {
                    (*MixerWindow).hStatusBar = CreateStatusWindow(
                        (WS_VISIBLE | WS_CHILD | WS_CLIPSIBLINGS) as i32,
                        core::ptr::null_mut(),
                        hwnd,
                        0,
                    );
                    if !(*MixerWindow).hStatusBar.is_null() {
                        SendMessage(
                            (*MixerWindow).hStatusBar,
                            WM_SETTEXT,
                            0,
                            core::ptr::addr_of_mut!(szProduct) as LPARAM,
                        );
                    }
                }

                if RebuildMixerWindowControls(core::ptr::addr_of_mut!(Preferences)) == 0 {
                    DPRINT!(b"Rebuilding mixer window controls failed!\n\0".as_ptr() as *const core::ffi::c_char);
                    SndMixerDestroy((*MixerWindow).Mixer);
                    (*MixerWindow).Mixer = core::ptr::null_mut();
                    Result = -1 as LRESULT;
                }
            }

            RegisterHotKey(hwnd, (HOTKEY_CTRL_S as i32), MOD_CONTROL, 'S' as UINT);
        }

        WM_DESTROY => {
            MixerWindow = GetWindowData!(hwnd, PMIXER_WINDOW);
            if !MixerWindow.is_null() {
                if !(*MixerWindow).Mixer.is_null() {
                    SndMixerDestroy((*MixerWindow).Mixer);
                }
                if !(*MixerWindow).hFont.is_null() {
                    DeleteObject(((*MixerWindow).hFont).cast());
                }
                HeapFree(hAppHeap, 0, MixerWindow as PVOID);
            }
        }

        WM_CLOSE => {
            SaveXYCoordWnd(hwnd, core::ptr::addr_of_mut!(Preferences));
            PostQuitMessage(0);
        }

        WM_HOTKEY => {
            if wParam == HOTKEY_CTRL_S as WPARAM {
                (*Preferences.MixerWindow).Mode = (if (*Preferences.MixerWindow).Mode
                    == NORMAL_MODE
                {
                    SMALL_MODE
                } else {
                    NORMAL_MODE
                }) as WINDOW_MODE;
                RebuildMixerWindowControls(core::ptr::addr_of_mut!(Preferences));
            }
        }

        _ => {
            Result = DefWindowProc(hwnd, uMsg, wParam, lParam);
        }
    }

    return Result;
}

unsafe extern "C" fn RegisterApplicationClasses() -> BOOL {
    let mut wc: WNDCLASSEXW = core::mem::zeroed();

    wc.cbSize = core::mem::size_of::<WNDCLASSEX>() as UINT;
    wc.style = (CS_HREDRAW | CS_VREDRAW) as UINT;
    wc.lpfnWndProc = Some(MainWindowProc);
    wc.cbClsExtra = 0 as core::ffi::c_int;
    wc.cbWndExtra = core::mem::size_of::<PMIXER_WINDOW>() as core::ffi::c_int;
    wc.hInstance = hAppInstance;
    wc.hIcon = LoadIcon(
        hAppInstance,
        MAKEINTRESOURCE!(IDI_MAINAPP) as LPCTSTR,
    );
    wc.hCursor = LoadCursor(
        core::ptr::null_mut(),
        IDC_ARROW as LPCTSTR,
    );
    wc.hbrBackground = (COLOR_BTNFACE + 1) as HBRUSH;
    wc.lpszMenuName = core::ptr::null();
    wc.lpszClassName = (SZ_APP_CLASS).cast();
    wc.hIconSm = core::ptr::null_mut();
    MainWindowClass = RegisterClassEx(core::ptr::addr_of_mut!(wc));

    return (MainWindowClass != 0) as BOOL;
}

unsafe extern "C" fn UnregisterApplicationClasses() {
    UnregisterClass(SZ_APP_CLASS, hAppInstance);
}

unsafe extern "C" fn CreateApplicationWindow(
    WindowMode: WINDOW_MODE,
    MixerId: UINT,
) -> HWND {
    let mut hWnd: HWND = core::ptr::null_mut();

    let MixerWindow: PMIXER_WINDOW = HeapAlloc(
        hAppHeap,
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<MIXER_WINDOW>() as SIZE_T,
    ) as PMIXER_WINDOW;
    if MixerWindow.is_null() {
        return core::ptr::null_mut();
    }

    (*MixerWindow).Mode = WindowMode;
    (*MixerWindow).MixerId = MixerId;

    if mixerGetNumDevs() > 0 as UINT {
        hWnd = CreateWindowEx(
            (WS_EX_WINDOWEDGE | WS_EX_CONTROLPARENT) as DWORD,
            SZ_APP_CLASS as LPCWSTR,
            lpAppTitle as LPCWSTR,
            (WS_DLGFRAME
                | WS_CAPTION
                | WS_MINIMIZEBOX
                | WS_SYSMENU
                | WS_CLIPCHILDREN
                | WS_CLIPSIBLINGS
                | WS_VISIBLE) as DWORD,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            300 as ::core::ffi::c_int,
            315 as ::core::ffi::c_int,
            core::ptr::null_mut(),
            LoadMenu(
                hAppInstance,
                MAKEINTRESOURCE!(IDM_MAINMENU),
            ),
            hAppInstance,
            MixerWindow as LPVOID,
        );
    } else {
        let mut lpErrMessage: LPTSTR = core::ptr::null_mut();

        /*
         * no mixer devices are available!
         */

        hWnd = core::ptr::null_mut();
        if AllocAndLoadString(
            core::ptr::addr_of_mut!(lpErrMessage),
            hAppInstance,
            IDS_NOMIXERDEVICES as UINT,
        ) != 0
        {
            MessageBox(
                core::ptr::null_mut(),
                lpErrMessage as LPCWSTR,
                lpAppTitle as LPCWSTR,
                MB_ICONINFORMATION as UINT,
            );
            LocalFree(lpErrMessage as HLOCAL);
        }
    }

    if hWnd.is_null() {
        HeapFree(
            hAppHeap,
            0 as DWORD,
            MixerWindow as PVOID,
        );
    }

    return hWnd;
}

unsafe extern "C" fn HandleCommandLine(
    mut cmdline: LPTSTR,
    mut dwStyle: DWORD,
    mut pMode: PWINDOW_MODE,
    mut pMixerId: PUINT,
) -> BOOL {
    let mut option: TCHAR = 0;
    *pMixerId = PLAY_MIXER;
    *pMode = if (dwStyle & 0x20) != 0 {
        SMALL_MODE
    } else {
        NORMAL_MODE
    };
    while *cmdline == (' ' as u16) || *cmdline == ('-' as u16) || *cmdline == ('/' as u16) {
        let fresh19 = cmdline;
        cmdline = cmdline.offset(1);
        if *fresh19 == (' ' as u16) {
            continue;
        }
        option = *cmdline;
        if option != 0 {
            cmdline = cmdline.offset(1);
        }
        while *cmdline == (' ' as u16) {
            cmdline = cmdline.offset(1);
        }
        match option as i32 {
            100 | 68 => {}
            110 | 78 => {
                *pMode = NORMAL_MODE;
            }
            115 | 83 => {
                *pMode = SMALL_MODE;
            }
            116 | 84 => {
                *pMode = TRAY_MODE;
            }
            112 | 80 => {
                *pMixerId = PLAY_MIXER;
            }
            114 | 82 => {
                *pMixerId = RECORD_MIXER;
            }
            _ => return (FALSE) as i32,
        }
    }
    return (TRUE) as BOOL;
}

#[export_name = "wWinMain"]
pub unsafe extern "system" fn _tWinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpszCmdLine: LPTSTR,
    nCmdShow: i32,
) -> i32 {
    let mut Msg: MSG = core::mem::zeroed();
    let mut Ret: i32 = 1;
    let mut Controls: INITCOMMONCONTROLSEX = core::mem::zeroed();
    let mut WindowMode: WINDOW_MODE = SMALL_MODE;
    let mut MixerId: UINT = 0;
    let mut dwStyle: DWORD;

    UNREFERENCED_PARAMETER!(hPrevInstance);
    UNREFERENCED_PARAMETER!(nCmdShow);

    hAppInstance = hInstance;
    hAppHeap = GetProcessHeap();

    if InitAppConfig() != 0 {
        dwStyle = GetStyleValue();
        HandleCommandLine(
            lpszCmdLine,
            dwStyle,
            core::ptr::addr_of_mut!(WindowMode),
            core::ptr::addr_of_mut!(MixerId),
        );

        /* load the application title */
        if AllocAndLoadString(
            core::ptr::addr_of_mut!(lpAppTitle),
            hAppInstance,
            IDS_SNDVOL32,
        ) == 0
        {
            lpAppTitle = core::ptr::null_mut();
        }

        Controls.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
        Controls.dwICC = (ICC_BAR_CLASSES | ICC_STANDARD_CLASSES) as DWORD;

        InitCommonControlsEx(core::ptr::addr_of_mut!(Controls));

        if WindowMode == TRAY_MODE {
            DialogBoxParam(
                hAppInstance,
                MAKEINTRESOURCE!(IDD_TRAY_MASTER),
                core::ptr::null_mut(),
                Some(TrayDlgProc),
                0,
            );
        } else {
            if RegisterApplicationClasses() != 0 {
                hMainWnd = CreateApplicationWindow(WindowMode, MixerId);
                if !hMainWnd.is_null() {
                    let mut bRet: BOOL = 0;
                    while {
                        bRet = GetMessage(
                            core::ptr::addr_of_mut!(Msg),
                            core::ptr::null_mut(),
                            0,
                            0,
                        );
                        bRet
                    } != 0
                    {
                        if bRet != -1 {
                            TranslateMessage(core::ptr::addr_of_mut!(Msg));
                            DispatchMessage(core::ptr::addr_of_mut!(Msg));
                        }
                    }

                    DestroyWindow(hMainWnd);
                    Ret = 0;
                } else {
                    DPRINT!(
                        b"Failed to create application window (LastError: %d)!\n\0".as_ptr() as *const i8,
                        GetLastError(),
                    );
                }

                UnregisterApplicationClasses();
            } else {
                DPRINT!(
                    b"Failed to register application classes (LastError: %d)!\n\0".as_ptr() as *const i8,
                    GetLastError(),
                );
            }
        }

        if !lpAppTitle.is_null() {
            LocalFree(lpAppTitle as HLOCAL);
        }

        CloseAppConfig();
    } else {
        DPRINT!(
            b"Unable to open the Volume Control registry key!\n\0".as_ptr() as *const i8,
        );
    }

    return Ret;
}
