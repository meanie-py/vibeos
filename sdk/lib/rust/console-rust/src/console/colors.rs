use crate::console::console::ApplyConsoleInfo;
use crate::console::layout::PaintText;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut ActiveStaticControl: DWORD = 0 as DWORD;

unsafe extern "C" fn PaintStaticControls(
    drawItem: LPDRAWITEMSTRUCT,
    pConInfo: PCONSOLE_STATE_INFO,
) {
    let mut hBrush: HBRUSH = core::ptr::null_mut();
    let mut index: DWORD;

    index = min(
        (*drawItem).CtlID.wrapping_sub(IDC_STATIC_COLOR1 as UINT),
        ARRAYSIZE!((*pConInfo).ColorTable).wrapping_sub(1 as UINT),
    ) as DWORD;

    hBrush = CreateSolidBrush((*pConInfo).ColorTable[index as usize]);
    FillRect(
        (*drawItem).hDC,
        core::ptr::addr_of_mut!((*drawItem).rcItem) as LPCRECT,
        if !hBrush.is_null() {
            hBrush as HBRUSH
        } else {
            GetStockObject((BLACK_BRUSH as i32)) as HBRUSH
        },
    );
    if !hBrush.is_null() {
        DeleteObject(hBrush as HGDIOBJ);
    }

    if ActiveStaticControl == index {
        DrawFocusRect(
            (*drawItem).hDC,
            core::ptr::addr_of_mut!((*drawItem).rcItem) as LPCRECT,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn ColorsProc(
    hDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut colorIndex: DWORD = 0;
    let mut color: COLORREF = 0;
    'sw: {
        match uMsg {
            WM_INITDIALOG => {
                SendDlgItemMessageW(
                    hDlg,
                    (IDC_UPDOWN_COLOR_RED as i32),
                    UDM_SETRANGE as UINT,
                    0 as WPARAM,
                    MAKELONG!(255, 0) as LPARAM,
                );
                SendDlgItemMessageW(
                    hDlg,
                    (IDC_UPDOWN_COLOR_GREEN as i32),
                    UDM_SETRANGE as UINT,
                    0 as WPARAM,
                    MAKELONG!(255, 0) as LPARAM,
                );
                SendDlgItemMessageW(
                    hDlg,
                    (IDC_UPDOWN_COLOR_BLUE as i32),
                    UDM_SETRANGE as UINT,
                    0 as WPARAM,
                    MAKELONG!(255, 0) as LPARAM,
                );
                CheckRadioButton(
                    hDlg,
                    (IDC_RADIO_SCREEN_TEXT as i32),
                    (IDC_RADIO_POPUP_BACKGROUND as i32),
                    (IDC_RADIO_SCREEN_BACKGROUND as i32),
                );
                SendMessageW(
                    hDlg,
                    WM_COMMAND as UINT,
                    IDC_RADIO_SCREEN_BACKGROUND as WPARAM,
                    0 as LPARAM,
                );
                return TRUE as INT_PTR;
            }

            WM_DRAWITEM => {
                let drawItem: LPDRAWITEMSTRUCT = lParam as LPDRAWITEMSTRUCT;

                if (IDC_STATIC_COLOR1 as UINT) <= (*drawItem).CtlID
                    && (*drawItem).CtlID <= (IDC_STATIC_COLOR16 as UINT)
                {
                    PaintStaticControls(drawItem, ConInfo);
                } else if (*drawItem).CtlID == (IDC_STATIC_SCREEN_COLOR as UINT) {
                    PaintText(drawItem, ConInfo, Screen);
                } else if (*drawItem).CtlID == (IDC_STATIC_POPUP_COLOR as UINT) {
                    PaintText(drawItem, ConInfo, Popup);
                }

                return TRUE as INT_PTR;
            }

            WM_NOTIFY => {
                match ((*(lParam as LPNMHDR)).code) as i32 {
                    PSN_APPLY => {
                        ApplyConsoleInfo(hDlg);
                        return TRUE as INT_PTR;
                    }

                    UDN_DELTAPOS => {
                        let lpnmud: LPNMUPDOWN = lParam as LPNMUPDOWN;

                        /* Get the current color */
                        colorIndex = ActiveStaticControl;
                        color = (*ConInfo).ColorTable[colorIndex as usize];

                        if (*lpnmud).hdr.idFrom == (IDC_UPDOWN_COLOR_RED as UINT_PTR) {
                            (*lpnmud).iPos = min(
                                max(
                                    (*lpnmud).iPos + (*lpnmud).iDelta,
                                    0,
                                ),
                                255,
                            );
                            color = RGB!(
                                (*lpnmud).iPos,
                                GetGValue!(color),
                                GetBValue!(color),
                            );
                        } else if (*lpnmud).hdr.idFrom == (IDC_UPDOWN_COLOR_GREEN as UINT_PTR) {
                            (*lpnmud).iPos = min(
                                max(
                                    (*lpnmud).iPos + (*lpnmud).iDelta,
                                    0,
                                ),
                                255,
                            );
                            color = RGB!(
                                GetRValue!(color),
                                (*lpnmud).iPos,
                                GetBValue!(color),
                            );
                        } else if (*lpnmud).hdr.idFrom == (IDC_UPDOWN_COLOR_BLUE as UINT_PTR) {
                            (*lpnmud).iPos = min(
                                max(
                                    (*lpnmud).iPos + (*lpnmud).iDelta,
                                    0,
                                ),
                                255,
                            );
                            color = RGB!(
                                GetRValue!(color),
                                GetGValue!(color),
                                (*lpnmud).iPos,
                            );
                        } else {
                            break 'sw;
                        }

                        (*ConInfo).ColorTable[colorIndex as usize] = color;
                        InvalidateRect(
                            GetDlgItem(
                                hDlg,
                                ((IDC_STATIC_COLOR1 as DWORD).wrapping_add(colorIndex))
                                    as ::core::ffi::c_int,
                            ),
                            core::ptr::null_mut(),
                            TRUE as BOOL,
                        );
                        InvalidateRect(
                            GetDlgItem(hDlg, (IDC_STATIC_SCREEN_COLOR as i32)),
                            core::ptr::null_mut(),
                            TRUE as BOOL,
                        );
                        InvalidateRect(
                            GetDlgItem(hDlg, (IDC_STATIC_POPUP_COLOR as i32)),
                            core::ptr::null_mut(),
                            TRUE as BOOL,
                        );

                        PropSheet_Changed!(GetParent(hDlg), hDlg);
                        break 'sw;
                    }

                    _ => {}
                }

                break 'sw;
            }

            WM_COMMAND => {
                /* NOTE: both BN_CLICKED and STN_CLICKED == 0 */
                if HIWORD!(wParam) == (BN_CLICKED as WORD)
                /* || HIWORD(wParam) == STN_CLICKED */
                {
                    let ctrlIndex: WORD = LOWORD!(wParam);

                    if (ctrlIndex as ::core::ffi::c_int) == (IDC_RADIO_SCREEN_TEXT as i32)
                        || (ctrlIndex as ::core::ffi::c_int) == (IDC_RADIO_SCREEN_BACKGROUND as i32)
                        || (ctrlIndex as ::core::ffi::c_int) == (IDC_RADIO_POPUP_TEXT as i32)
                        || (ctrlIndex as ::core::ffi::c_int) == (IDC_RADIO_POPUP_BACKGROUND as i32)
                    {
                        match (ctrlIndex as ::core::ffi::c_int) as u32 {
                            IDC_RADIO_SCREEN_TEXT => {
                                /* Get the colour of the screen foreground */
                                colorIndex = TextAttribFromAttrib!((*ConInfo).ScreenAttributes) as DWORD;
                            }

                            IDC_RADIO_SCREEN_BACKGROUND => {
                                /* Get the colour of the screen background */
                                colorIndex = BkgdAttribFromAttrib!((*ConInfo).ScreenAttributes) as DWORD;
                            }

                            IDC_RADIO_POPUP_TEXT => {
                                /* Get the colour of the popup foreground */
                                colorIndex = TextAttribFromAttrib!((*ConInfo).PopupAttributes) as DWORD;
                            }

                            IDC_RADIO_POPUP_BACKGROUND => {
                                /* Get the colour of the popup background */
                                colorIndex = BkgdAttribFromAttrib!((*ConInfo).PopupAttributes) as DWORD;
                            }

                            _ => {}
                        }

                        color = (*ConInfo).ColorTable[colorIndex as usize];

                        /* Set the values of the colour indicators */
                        SetDlgItemInt(
                            hDlg,
                            (IDC_EDIT_COLOR_RED as i32),
                            GetRValue!(color) as UINT,
                            FALSE as BOOL,
                        );
                        SetDlgItemInt(
                            hDlg,
                            (IDC_EDIT_COLOR_GREEN as i32),
                            GetGValue!(color) as UINT,
                            FALSE as BOOL,
                        );
                        SetDlgItemInt(
                            hDlg,
                            (IDC_EDIT_COLOR_BLUE as i32),
                            GetBValue!(color) as UINT,
                            FALSE as BOOL,
                        );

                        InvalidateRect(
                            GetDlgItem(
                                hDlg,
                                ((IDC_STATIC_COLOR1 as DWORD).wrapping_add(ActiveStaticControl))
                                    as ::core::ffi::c_int,
                            ),
                            core::ptr::null_mut(),
                            TRUE as BOOL,
                        );
                        ActiveStaticControl = colorIndex;
                        InvalidateRect(
                            GetDlgItem(
                                hDlg,
                                ((IDC_STATIC_COLOR1 as DWORD).wrapping_add(ActiveStaticControl))
                                    as ::core::ffi::c_int,
                            ),
                            core::ptr::null_mut(),
                            TRUE as BOOL,
                        );
                        break 'sw;
                    } else if (IDC_STATIC_COLOR1 as ::core::ffi::c_int)
                        <= (ctrlIndex as ::core::ffi::c_int)
                        && (ctrlIndex as ::core::ffi::c_int)
                            <= (IDC_STATIC_COLOR16 as ::core::ffi::c_int)
                    {
                        colorIndex = ((ctrlIndex as ::core::ffi::c_int)
                            - (IDC_STATIC_COLOR1 as ::core::ffi::c_int))
                            as DWORD;

                        /* If the same static control was re-clicked, don't take it into account */
                        if colorIndex == ActiveStaticControl {
                            break 'sw;
                        }

                        color = (*ConInfo).ColorTable[colorIndex as usize];

                        /* Set the values of the colour indicators */
                        SetDlgItemInt(
                            hDlg,
                            (IDC_EDIT_COLOR_RED as i32),
                            GetRValue!(color) as UINT,
                            FALSE as BOOL,
                        );
                        SetDlgItemInt(
                            hDlg,
                            (IDC_EDIT_COLOR_GREEN as i32),
                            GetGValue!(color) as UINT,
                            FALSE as BOOL,
                        );
                        SetDlgItemInt(
                            hDlg,
                            (IDC_EDIT_COLOR_BLUE as i32),
                            GetBValue!(color) as UINT,
                            FALSE as BOOL,
                        );

                        if IsDlgButtonChecked(hDlg, (IDC_RADIO_SCREEN_TEXT as i32)) != 0 {
                            (*ConInfo).ScreenAttributes =
                                MakeAttrib!((colorIndex as WORD), BkgdAttribFromAttrib!((*ConInfo).ScreenAttributes));
                        } else if IsDlgButtonChecked(hDlg, (IDC_RADIO_SCREEN_BACKGROUND as i32)) != 0 {
                            (*ConInfo).ScreenAttributes =
                                MakeAttrib!(TextAttribFromAttrib!((*ConInfo).ScreenAttributes), (colorIndex as WORD));
                        } else if IsDlgButtonChecked(hDlg, (IDC_RADIO_POPUP_TEXT as i32)) != 0 {
                            (*ConInfo).PopupAttributes =
                                MakeAttrib!((colorIndex as WORD), BkgdAttribFromAttrib!((*ConInfo).PopupAttributes));
                        } else if IsDlgButtonChecked(hDlg, (IDC_RADIO_POPUP_BACKGROUND as i32)) != 0 {
                            (*ConInfo).PopupAttributes =
                                MakeAttrib!(TextAttribFromAttrib!((*ConInfo).PopupAttributes), (colorIndex as WORD));
                        }

                        InvalidateRect(
                            GetDlgItem(
                                hDlg,
                                ((IDC_STATIC_COLOR1 as DWORD).wrapping_add(ActiveStaticControl))
                                    as ::core::ffi::c_int,
                            ),
                            core::ptr::null_mut(),
                            TRUE as BOOL,
                        );
                        ActiveStaticControl = colorIndex;
                        InvalidateRect(
                            GetDlgItem(
                                hDlg,
                                ((IDC_STATIC_COLOR1 as DWORD).wrapping_add(ActiveStaticControl))
                                    as ::core::ffi::c_int,
                            ),
                            core::ptr::null_mut(),
                            TRUE as BOOL,
                        );
                        InvalidateRect(
                            GetDlgItem(hDlg, (IDC_STATIC_SCREEN_COLOR as i32)),
                            core::ptr::null_mut(),
                            TRUE as BOOL,
                        );
                        InvalidateRect(
                            GetDlgItem(hDlg, (IDC_STATIC_POPUP_COLOR as i32)),
                            core::ptr::null_mut(),
                            TRUE as BOOL,
                        );

                        PropSheet_Changed!(GetParent(hDlg), hDlg);
                        break 'sw;
                    }
                } else if HIWORD!(wParam) == (EN_KILLFOCUS as WORD) {
                    let ctrlIndex: WORD = LOWORD!(wParam);

                    if (ctrlIndex as ::core::ffi::c_int) == (IDC_EDIT_COLOR_RED as i32)
                        || (ctrlIndex as ::core::ffi::c_int) == (IDC_EDIT_COLOR_GREEN as i32)
                        || (ctrlIndex as ::core::ffi::c_int) == (IDC_EDIT_COLOR_BLUE as i32)
                    {
                        let mut value: DWORD = 0;

                        /* Get the current colour */
                        colorIndex = ActiveStaticControl;
                        color = (*ConInfo).ColorTable[colorIndex as usize];

                        /* Modify the colour component */
                        match (ctrlIndex as ::core::ffi::c_int) as u32 {
                            IDC_EDIT_COLOR_RED => {
                                value = GetDlgItemInt(
                                    hDlg,
                                    (IDC_EDIT_COLOR_RED as i32),
                                    ::core::ptr::null_mut::<BOOL>(),
                                    FALSE as BOOL,
                                ) as DWORD;
                                value = min(
                                    max(value, 0 as DWORD),
                                    255 as DWORD,
                                );
                                color = RGB!(value, GetGValue!(color), GetBValue!(color));
                            }

                            IDC_EDIT_COLOR_GREEN => {
                                value = GetDlgItemInt(
                                    hDlg,
                                    (IDC_EDIT_COLOR_GREEN as i32),
                                    ::core::ptr::null_mut::<BOOL>(),
                                    FALSE as BOOL,
                                ) as DWORD;
                                value = min(
                                    max(value, 0 as DWORD),
                                    255 as DWORD,
                                );
                                color = RGB!(GetRValue!(color), value, GetBValue!(color));
                            }

                            IDC_EDIT_COLOR_BLUE => {
                                value = GetDlgItemInt(
                                    hDlg,
                                    (IDC_EDIT_COLOR_BLUE as i32),
                                    ::core::ptr::null_mut::<BOOL>(),
                                    FALSE as BOOL,
                                ) as DWORD;
                                value = min(
                                    max(value, 0 as DWORD),
                                    255 as DWORD,
                                );
                                color = RGB!(GetRValue!(color), GetGValue!(color), value);
                            }

                            _ => {}
                        }

                        (*ConInfo).ColorTable[colorIndex as usize] = color;
                        InvalidateRect(
                            GetDlgItem(
                                hDlg,
                                ((IDC_STATIC_COLOR1 as DWORD).wrapping_add(colorIndex))
                                    as ::core::ffi::c_int,
                            ),
                            core::ptr::null_mut(),
                            TRUE as BOOL,
                        );
                        InvalidateRect(
                            GetDlgItem(hDlg, (IDC_STATIC_SCREEN_COLOR as i32)),
                            core::ptr::null_mut(),
                            TRUE as BOOL,
                        );
                        InvalidateRect(
                            GetDlgItem(hDlg, (IDC_STATIC_POPUP_COLOR as i32)),
                            core::ptr::null_mut(),
                            TRUE as BOOL,
                        );

                        PropSheet_Changed!(GetParent(hDlg), hDlg);
                        break 'sw;
                    }
                }

                break 'sw;
            }

            _ => {}
        }
    }

    return FALSE as INT_PTR;
}
