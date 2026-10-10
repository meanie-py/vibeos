use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_1E472015D7DF: [WCHAR; 6] = [0x0025, 0x002E, 0x0033, 0x006C, 0x0066, 0x0000];
pub static CPP_COM_WSTR_505E2CF608D7: [WCHAR; 4] = [0x0064, 0x0070, 0x0069, 0x0000];
pub static CPP_COM_WSTR_8D71B3FAAB82: [WCHAR; 2] = [0x0020, 0x0000];
pub static CPP_COM_WSTR_A9EDBA2FC474: [WCHAR; 3] = [0x0025, 0x0064, 0x0000];

use core::ffi::c_void;

// ShowError(INT, ...) itself is the C trampoline in rust_varargs_shim.c (see the prelude).
#[no_mangle]
pub unsafe extern "C" fn ShowError_v(stringID: INT, ap: *mut core::ffi::c_void) {
    let va = *(ap as *mut *mut core::ffi::c_void); // the caller's va_list value

    let mut strFormat = CStringW::new();
    let mut strText = CStringW::new();
    strFormat.LoadStringW(stringID as UINT);
    strText.FormatV(strFormat.GetString(), va);

    let mut strProgramName = CStringW::new();
    strProgramName.LoadStringW(IDS_PROGRAMNAME);

    mainWindow.__base0.MessageBoxW(strText.GetString(), strProgramName.GetString(), MB_ICONERROR);
}

unsafe extern "system" fn EnumFontFamProc(lpelf: *mut ENUMLOGFONTW, lpntm: *mut NEWTEXTMETRICW, FontType: INT, lParam: LPARAM) -> INT {
    let arrFontNames: *mut CSimpleArray<CStringW> = lParam as *mut CSimpleArray<CStringW>;
    let name: LPWSTR = (*lpelf).elfLogFont.lfFaceName.as_mut_ptr();
    if *name.add(0 as usize) == 0x0040u16 { // Vertical fonts
        return (TRUE) as INT;
    }

    let mut i: INT = 0;
    while i < (*arrFontNames).GetSize() {
        let elem: *mut CStringW = (*arrFontNames).GetAt(i);
        let s: *const WCHAR = (*elem).GetString();
        // case-sensitive wide string equality, as CStringW operator== does
        let mut eq: bool = true;
        let mut k: usize = 0;
        loop {
            let a: WCHAR = *s.add(k);
            let b: WCHAR = *name.add(k);
            if a != b {
                eq = false;
                break;
            }
            if a == 0 {
                break;
            }
            k += 1;
        }
        if eq { // Already exists
            return (TRUE) as INT;
        }
        i += 1;
    }

    let _ = (*arrFontNames).Add(CStringW::from_ptr(name));
    return (TRUE) as INT;
}

impl CMirrorRotateDialog {
    pub unsafe fn OnInitDialog(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.CheckDlgButton((IDD_MIRRORROTATERB1 as i32), BST_CHECKED);
        (*this_).__base0.CheckDlgButton((IDD_MIRRORROTATERB4 as i32), BST_CHECKED);
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnClose(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).EndDialog(0);
        return 0;
    }
    pub unsafe fn OnOk(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if (*this_).__base0.IsDlgButtonChecked((IDD_MIRRORROTATERB1 as i32)) != 0 {
            (*this_).EndDialog(1);
        } else if (*this_).__base0.IsDlgButtonChecked((IDD_MIRRORROTATERB2 as i32)) != 0 {
            (*this_).EndDialog(2);
        } else if (*this_).__base0.IsDlgButtonChecked((IDD_MIRRORROTATERB4 as i32)) != 0 {
            (*this_).EndDialog(3);
        } else if (*this_).__base0.IsDlgButtonChecked((IDD_MIRRORROTATERB5 as i32)) != 0 {
            (*this_).EndDialog(4);
        } else if (*this_).__base0.IsDlgButtonChecked((IDD_MIRRORROTATERB6 as i32)) != 0 {
            (*this_).EndDialog(5);
        }
        return 0;
    }
    pub unsafe fn OnCancel(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).EndDialog(0);
        return 0;
    }
    pub unsafe fn OnRadioButton3(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if (*this_).__base0.IsDlgButtonChecked((IDD_MIRRORROTATERB3 as i32)) != BST_CHECKED {
            return 0;
        }
    
        EnableWindow((*this_).__base0.GetDlgItem((IDD_MIRRORROTATERB4 as i32)), (TRUE) as BOOL);
        EnableWindow((*this_).__base0.GetDlgItem((IDD_MIRRORROTATERB5 as i32)), (TRUE) as BOOL);
        EnableWindow((*this_).__base0.GetDlgItem((IDD_MIRRORROTATERB6 as i32)), (TRUE) as BOOL);
        return 0;
    }
    pub unsafe fn OnRadioButton12(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if (*this_).__base0.IsDlgButtonChecked((IDD_MIRRORROTATERB1 as i32)) != BST_CHECKED &&
            (*this_).__base0.IsDlgButtonChecked((IDD_MIRRORROTATERB2 as i32)) != BST_CHECKED
        {
            return 0;
        }
    
        EnableWindow((*this_).__base0.GetDlgItem((IDD_MIRRORROTATERB4 as i32)), (FALSE) as BOOL);
        EnableWindow((*this_).__base0.GetDlgItem((IDD_MIRRORROTATERB5 as i32)), (FALSE) as BOOL);
        EnableWindow((*this_).__base0.GetDlgItem((IDD_MIRRORROTATERB6 as i32)), (FALSE) as BOOL);
        return 0;
    }
}

impl CAttributesDialog {
    pub unsafe fn OnInitDialog(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).newWidth = imageModel.GetWidth();
        (*this_).newHeight = imageModel.GetHeight();
    
        (*this_).__base0.CheckDlgButton((IDD_ATTRIBUTESRB3 as i32), BST_CHECKED);
        (*this_).__base0.SetDlgItemInt((IDD_ATTRIBUTESEDIT1 as i32), ((*this_).newWidth) as u32, (FALSE) as i32);
        (*this_).__base0.SetDlgItemInt((IDD_ATTRIBUTESEDIT2 as i32), ((*this_).newHeight) as u32, (FALSE) as i32);
    
        if ((imageModel.IsBlackAndWhite()) != 0) {
            (*this_).__base0.CheckRadioButton((IDD_ATTRIBUTESRB4 as i32), (IDD_ATTRIBUTESRB5 as i32), (IDD_ATTRIBUTESRB4 as i32));
        } else {
            (*this_).__base0.CheckRadioButton((IDD_ATTRIBUTESRB4 as i32), (IDD_ATTRIBUTESRB5 as i32), (IDD_ATTRIBUTESRB5 as i32));
        }
    
        if g_isAFile != 0 {
            let mut date = [0u16; 100];
            let mut temp = [0u16; 100];
            GetDateFormatW(LOCALE_USER_DEFAULT, 0, core::ptr::addr_of_mut!(g_fileTime), core::ptr::null_mut(), date.as_mut_ptr(), (_countof!(date)) as i32);
            GetTimeFormatW(LOCALE_USER_DEFAULT, 0, core::ptr::addr_of_mut!(g_fileTime), core::ptr::null_mut(), temp.as_mut_ptr(), (_countof!(temp)) as i32);
            StringCchCatW(date.as_mut_ptr(), (_countof!(date)) as _, CPP_COM_WSTR_8D71B3FAAB82.as_ptr());
            StringCchCatW(date.as_mut_ptr(), (_countof!(date)) as _, temp.as_mut_ptr());
            let mut strSize = CStringW::new();
            atl_format!(strSize, IDS_FILESIZE, g_fileSize);
            (*this_).__base0.SetDlgItemTextW((IDD_ATTRIBUTESTEXT6 as i32), date.as_ptr());
            (*this_).__base0.SetDlgItemTextW((IDD_ATTRIBUTESTEXT7 as i32), strSize.GetString());
        }
    
        let mut strUnit = CStringW::new();
        (*this_).__base0.GetDlgItemTextW__1((IDD_ATTRIBUTESTEXT8 as i32), &mut strUnit);
    
        let mut strRes = CStringW::new();
        if wcscmp(strUnit.GetString(), CPP_COM_WSTR_505E2CF608D7.as_ptr()) == 0 {
            atl_format!(strRes, IDS_PRINTRES, ROUND!(g_xDpi), ROUND!(g_yDpi));
        } else {
            atl_format!(strRes, IDS_PRINTRES, ROUND!(PpcmFromDpi(g_xDpi)), ROUND!(PpcmFromDpi(g_yDpi)));
        }
    
        (*this_).__base0.SetDlgItemTextW((IDD_ATTRIBUTESTEXT8 as i32), strRes.GetString());
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnClose(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).EndDialog(0);
        return 0;
    }
    pub unsafe fn OnOk(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).m_bBlackAndWhite = (((*this_).__base0.IsDlgButtonChecked((IDD_ATTRIBUTESRB4 as i32)) == BST_CHECKED) as BOOL);
        (*this_).EndDialog(1);
        return 0;
    }
    pub unsafe fn OnCancel(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).EndDialog(0);
        return 0;
    }
    pub unsafe fn OnDefault(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).newWidth = imageModel.GetWidth();
        (*this_).newHeight = imageModel.GetHeight();
        (*this_).__base0.CheckDlgButton((IDD_ATTRIBUTESRB3 as i32), BST_CHECKED);
        (*this_).__base0.CheckDlgButton((IDD_ATTRIBUTESRB5 as i32), BST_CHECKED);
        (*this_).__base0.SetDlgItemInt((IDD_ATTRIBUTESEDIT1 as i32), ((*this_).newWidth) as u32, (FALSE) as i32);
        (*this_).__base0.SetDlgItemInt((IDD_ATTRIBUTESEDIT2 as i32), ((*this_).newHeight) as u32, (FALSE) as i32);
        return 0;
    }
    pub unsafe fn OnRadioButton1(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if (*this_).__base0.IsDlgButtonChecked((IDD_ATTRIBUTESRB1 as i32)) != BST_CHECKED {
            return 0;
        }
    
        let mut strNum = CStringW::new();
        atl_format!(strNum, CPP_COM_WSTR_1E472015D7DF.as_ptr(), (*this_).newWidth / g_xDpi);
        (*this_).__base0.SetDlgItemTextW((IDD_ATTRIBUTESEDIT1 as i32), strNum.GetString());
        atl_format!(strNum, CPP_COM_WSTR_1E472015D7DF.as_ptr(), (*this_).newHeight / g_yDpi);
        (*this_).__base0.SetDlgItemTextW((IDD_ATTRIBUTESEDIT2 as i32), strNum.GetString());
        return 0;
    }
    pub unsafe fn OnRadioButton2(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if (*this_).__base0.IsDlgButtonChecked((IDD_ATTRIBUTESRB2 as i32)) != BST_CHECKED {
            return 0;
        }
    
        let mut strNum = CStringW::new();
        atl_format!(strNum, CPP_COM_WSTR_1E472015D7DF.as_ptr(), (*this_).newWidth / PpcmFromDpi(g_xDpi));
        (*this_).__base0.SetDlgItemTextW((IDD_ATTRIBUTESEDIT1 as i32), strNum.GetString());
        atl_format!(strNum, CPP_COM_WSTR_1E472015D7DF.as_ptr(), (*this_).newHeight / PpcmFromDpi(g_yDpi));
        (*this_).__base0.SetDlgItemTextW((IDD_ATTRIBUTESEDIT2 as i32), strNum.GetString());
        return 0;
    }
    pub unsafe fn OnRadioButton3(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if (*this_).__base0.IsDlgButtonChecked((IDD_ATTRIBUTESRB3 as i32)) != BST_CHECKED {
            return 0;
        }
    
        (*this_).__base0.SetDlgItemInt((IDD_ATTRIBUTESEDIT1 as i32), ((*this_).newWidth) as u32, (FALSE) as i32);
        (*this_).__base0.SetDlgItemInt((IDD_ATTRIBUTESEDIT2 as i32), ((*this_).newHeight) as u32, (FALSE) as i32);
        return 0;
    }
    pub unsafe fn OnEdit1(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if Edit_GetModify!(hWndCtl) != 0 {
            let mut tempS: [WCHAR; 100] = core::mem::zeroed();
            if (*this_).__base0.IsDlgButtonChecked((IDD_ATTRIBUTESRB1 as i32)) != 0 {
                (*this_).__base0.GetDlgItemTextW__0((IDD_ATTRIBUTESEDIT1 as i32), tempS.as_mut_ptr(), (_countof!(tempS)) as i32);
                (*this_).newWidth = max(1, (wcstod(tempS.as_mut_ptr(), core::ptr::null_mut()) * g_xDpi as f64) as i32);
            }
            else if (*this_).__base0.IsDlgButtonChecked((IDD_ATTRIBUTESRB2 as i32)) != 0 {
                (*this_).__base0.GetDlgItemTextW__0((IDD_ATTRIBUTESEDIT1 as i32), tempS.as_mut_ptr(), (_countof!(tempS)) as i32);
                (*this_).newWidth = max(1, (wcstod(tempS.as_mut_ptr(), core::ptr::null_mut()) * PpcmFromDpi(g_xDpi) as f64) as i32);
            }
            else if (*this_).__base0.IsDlgButtonChecked((IDD_ATTRIBUTESRB3 as i32)) != 0 {
                (*this_).__base0.GetDlgItemTextW__0((IDD_ATTRIBUTESEDIT1 as i32), tempS.as_mut_ptr(), (_countof!(tempS)) as i32);
                (*this_).newWidth = max(1, _wtoi(tempS.as_mut_ptr()));
            }
            Edit_SetModify!(hWndCtl, FALSE);
        }
        return 0;
    }
    pub unsafe fn OnEdit2(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if Edit_GetModify!(hWndCtl) != 0 {
            let mut tempS: [u16; 100] = [0u16; 100];
            if (*this_).__base0.IsDlgButtonChecked((IDD_ATTRIBUTESRB1 as i32)) != 0 {
                (*this_).__base0.GetDlgItemTextW__0((IDD_ATTRIBUTESEDIT2 as i32), tempS.as_mut_ptr(), _countof!(tempS) as i32);
                (*this_).newHeight = max(1, (wcstod(tempS.as_ptr(), core::ptr::null_mut()) * g_yDpi as f64) as i32);
            }
            else if (*this_).__base0.IsDlgButtonChecked((IDD_ATTRIBUTESRB2 as i32)) != 0 {
                (*this_).__base0.GetDlgItemTextW__0((IDD_ATTRIBUTESEDIT2 as i32), tempS.as_mut_ptr(), _countof!(tempS) as i32);
                (*this_).newHeight = max(1, (wcstod(tempS.as_ptr(), core::ptr::null_mut()) * PpcmFromDpi(g_yDpi) as f64) as i32);
            }
            else if (*this_).__base0.IsDlgButtonChecked((IDD_ATTRIBUTESRB3 as i32)) != 0 {
                (*this_).__base0.GetDlgItemTextW__0((IDD_ATTRIBUTESEDIT2 as i32), tempS.as_mut_ptr(), _countof!(tempS) as i32);
                (*this_).newHeight = max(1, _wtoi(tempS.as_ptr()));
            }
            Edit_SetModify!(hWndCtl, FALSE);
        }
        return 0;
    }
}

impl CStretchSkewDialog {
    pub unsafe fn OnInitDialog(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.SetDlgItemInt((IDD_STRETCHSKEWEDITHSTRETCH as i32), 100, (FALSE) as i32);
        (*this_).__base0.SetDlgItemInt((IDD_STRETCHSKEWEDITVSTRETCH as i32), 100, (FALSE) as i32);
        (*this_).__base0.SetDlgItemInt((IDD_STRETCHSKEWEDITHSKEW as i32), 0, (FALSE) as i32);
        (*this_).__base0.SetDlgItemInt((IDD_STRETCHSKEWEDITVSKEW as i32), 0, (FALSE) as i32);
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnClose(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).EndDialog(0);
        return 0;
    }
    pub unsafe fn OnOk(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut strrcIntNumbers = CStringW::new();
        let mut strrcPercentage = CStringW::new();
        let mut strrcAngle = CStringW::new();
        let mut tr1: BOOL = 0;
        let mut tr2: BOOL = 0;
        let mut tr3: BOOL = 0;
        let mut tr4: BOOL = 0;
    
        strrcIntNumbers.LoadStringW(IDS_INTNUMBERS);
        strrcPercentage.LoadStringW(IDS_PERCENTAGE);
        strrcAngle.LoadStringW(IDS_ANGLE);
    
        (*this_).percentage.x = ((*this_).__base0.GetDlgItemInt((IDD_STRETCHSKEWEDITHSTRETCH as i32), core::ptr::addr_of_mut!(tr1), (FALSE) as i32)) as i32;
        (*this_).percentage.y = ((*this_).__base0.GetDlgItemInt((IDD_STRETCHSKEWEDITVSTRETCH as i32), core::ptr::addr_of_mut!(tr2), (FALSE) as i32)) as i32;
        (*this_).angle.x = ((*this_).__base0.GetDlgItemInt((IDD_STRETCHSKEWEDITHSKEW as i32), core::ptr::addr_of_mut!(tr3), (TRUE) as i32)) as i32;
        (*this_).angle.y = ((*this_).__base0.GetDlgItemInt((IDD_STRETCHSKEWEDITVSKEW as i32), core::ptr::addr_of_mut!(tr4), (TRUE) as i32)) as i32;
    
        if !(tr1 != 0 && tr2 != 0 && tr3 != 0 && tr4 != 0) {
            (*this_).__base0.MessageBoxW(strrcIntNumbers.GetString(), core::ptr::null_mut(), MB_ICONEXCLAMATION);
        } else if (*this_).percentage.x < 1 || (*this_).percentage.x > 500 || (*this_).percentage.y < 1 || (*this_).percentage.y > 500 {
            (*this_).__base0.MessageBoxW(strrcPercentage.GetString(), core::ptr::null_mut(), MB_ICONEXCLAMATION);
        } else if (*this_).angle.x < -89 || (*this_).angle.x > 89 || (*this_).angle.y < -89 || (*this_).angle.y > 89 {
            (*this_).__base0.MessageBoxW(strrcAngle.GetString(), core::ptr::null_mut(), MB_ICONEXCLAMATION);
        } else {
            (*this_).EndDialog(1);
        }
        return 0;
    }
    pub unsafe fn OnCancel(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).EndDialog(0);
        return 0;
    }
}

impl CFontsDialog {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        {
        }
    }
    pub unsafe fn InitFontNames(&mut self) {
        let this_: *mut Self = self;
        // List the fonts
        let mut arrFontNames: CSimpleArray<CStringW> = CSimpleArray::<CStringW>::new();
        let hDC: HDC = CreateCompatibleDC(core::ptr::null_mut());
        if !hDC.is_null() {
            EnumFontFamiliesW(
                hDC,
                core::ptr::null_mut(),
                EnumFontFamProc,
                addr_of_mut!(arrFontNames) as LPARAM,
            );
            DeleteDC(hDC);
        }
    
        // Actually add them to the combobox
        let hwndNames: HWND = (*this_).__base0.GetDlgItem((IDD_FONTSNAMES as i32));
        SendMessageW(hwndNames, CB_RESETCONTENT, 0, 0);
        let mut i: INT = 0;
        while i < arrFontNames.GetSize() {
            ComboBox_AddString!(hwndNames, (*arrFontNames.GetAt(i)).GetString() as LPARAM);
            i += 1;
        }
    
        SetWindowTextW(hwndNames, registrySettings.strFontName.GetString());
    }
    pub unsafe fn InitFontSizes(&mut self) {
        let this_: *mut Self = self;
        static s_sizes: [INT; 16] = [
            8, 9, 10, 11, 12, 14, 16, 18, 20, 22, 24, 26, 28, 36, 48, 72
        ];
    
        let hwndSizes: HWND = (*this_).__base0.GetDlgItem((IDD_FONTSSIZES as i32));
        ComboBox_ResetContent!(hwndSizes);
    
        let mut szText: [WCHAR; 16] = [0 as WCHAR; 16];
        let mut i: UINT = 0;
        while (i as usize) < _countof!(s_sizes) as usize {
            StringCchPrintfW!(szText.as_mut_ptr(), _countof!(szText) as usize, CPP_COM_WSTR_A9EDBA2FC474.as_ptr(), s_sizes[i as usize]);
            let iItem: INT = ComboBox_AddString!(hwndSizes, szText.as_mut_ptr());
            if s_sizes[i as usize] == (registrySettings.PointSize as INT) {
                ComboBox_SetCurSel!(hwndSizes, iItem);
            }
            i += 1;
        }
    
        if ComboBox_GetCurSel!(hwndSizes) == CB_ERR {
            StringCchPrintfW!(szText.as_mut_ptr(), _countof!(szText) as usize, CPP_COM_WSTR_A9EDBA2FC474.as_ptr(), registrySettings.PointSize as INT);
            SetWindowTextW(hwndSizes, szText.as_mut_ptr());
        }
    }
    pub unsafe fn InitToolbar(&mut self) {
        let this_: *mut Self = self;
        let hwndToolbar: HWND = (*this_).__base0.GetDlgItem(IDD_FONTSTOOLBAR as _);
        SendMessageW(hwndToolbar, TB_BUTTONSTRUCTSIZE as _, (core::mem::size_of::<TBBUTTON>()) as WPARAM, 0 as _);
        SendMessageW(hwndToolbar, TB_SETBITMAPSIZE as _, 0 as _, MAKELPARAM!(16 as _, 16 as _));
        SendMessageW(hwndToolbar, TB_SETBUTTONWIDTH as _, 0 as _, MAKELPARAM!(20 as _, 20 as _));
    
        let mut AddBitmap: TBADDBITMAP = core::mem::zeroed();
        AddBitmap.hInst = g_hinstExe;
        AddBitmap.nID = IDB_FONTSTOOLBAR as _;
        SendMessageW(hwndToolbar, TB_ADDBITMAP as _, 4 as _, core::ptr::addr_of_mut!(AddBitmap) as LPARAM);
    
        let himl: HIMAGELIST = ImageList_LoadImage(g_hinstExe, MAKEINTRESOURCEW!(IDB_FONTSTOOLBAR as _),
                                              16 as _, 8 as _, RGB!(255 as _, 0 as _, 255 as _), IMAGE_BITMAP as _,
                                              LR_CREATEDIBSECTION as _);
        SendMessageW(hwndToolbar, TB_SETIMAGELIST as _, 0 as _, himl as LPARAM);
    
        let mut buttons: [TBBUTTON; 4] = [
            // { COMMENT_TOKEN_ECHO____, COMMENT_TOKEN_ECHO____, COMMENT_TOKEN_ECHO____, COMMENT_TOKEN_ECHO____ }
            TBBUTTON {
                iBitmap: 0 as _,
                idCommand: IDM_BOLD as _,
                fsState: TBSTATE_ENABLED as _,
                fsStyle: TBSTYLE_CHECK as _,
                ..core::mem::zeroed()
            },
            // { COMMENT_TOKEN_ECHO____, COMMENT_TOKEN_ECHO____, COMMENT_TOKEN_ECHO____, COMMENT_TOKEN_ECHO____ }
            TBBUTTON {
                iBitmap: 1 as _,
                idCommand: IDM_ITALIC as _,
                fsState: TBSTATE_ENABLED as _,
                fsStyle: TBSTYLE_CHECK as _,
                ..core::mem::zeroed()
            },
            // { COMMENT_TOKEN_ECHO____, COMMENT_TOKEN_ECHO____, COMMENT_TOKEN_ECHO____, COMMENT_TOKEN_ECHO____ }
            TBBUTTON {
                iBitmap: 2 as _,
                idCommand: IDM_UNDERLINE as _,
                fsState: TBSTATE_ENABLED as _,
                fsStyle: TBSTYLE_CHECK as _,
                ..core::mem::zeroed()
            },
            // { COMMENT_TOKEN_ECHO____, COMMENT_TOKEN_ECHO____, COMMENT_TOKEN_ECHO____, COMMENT_TOKEN_ECHO____ }, // TODO:
            TBBUTTON {
                iBitmap: 3 as _,
                idCommand: IDM_VERTICAL as _,
                fsState: 0 as _,
                fsStyle: TBSTYLE_CHECK as _,
                ..core::mem::zeroed()
            },
        ];
        SendMessageW(hwndToolbar, TB_ADDBUTTONS as _, _countof!(buttons) as _, core::ptr::addr_of_mut!(buttons) as LPARAM);
    
        SendMessageW(hwndToolbar, TB_CHECKBUTTON as _, IDM_BOLD as _, registrySettings.Bold as _);
        SendMessageW(hwndToolbar, TB_CHECKBUTTON as _, IDM_ITALIC as _, registrySettings.Italic as _);
        SendMessageW(hwndToolbar, TB_CHECKBUTTON as _, IDM_UNDERLINE as _, registrySettings.Underline as _);
    }
    pub unsafe fn OnInitDialog(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        // TODO: Tooltips
        (*this_).InitFontNames();
        (*this_).InitFontSizes();
        (*this_).InitToolbar();
    
        if registrySettings.FontsPositionX != 0 || registrySettings.FontsPositionY != 0
        {
            (*this_).__base0.SetWindowPos(core::ptr::null_mut(),
                         (registrySettings.FontsPositionX) as i32, (registrySettings.FontsPositionY) as i32,
                         0, 0,
                         SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER);
            (*this_).__base0.SendMessageW__0(DM_REPOSITION, 0, 0);
        }
    
        if ((registrySettings.ShowTextTool) == 0)
        {
            (*this_).__base0.ShowWindow((SW_HIDE) as i32);
        }
    
        return (TRUE) as LRESULT;
    }
    pub unsafe fn OnClose(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*this_).__base0.ShowWindow((SW_HIDE) as i32); // Just hide. Recycle for optimization
        return 0;
    }
    pub unsafe fn OnCommand(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let id: UINT = LOWORD!(wParam) as UINT;
        let codeNotify: UINT = HIWORD!(wParam) as UINT;
        let hwndToolbar: HWND = (*this_).__base0.GetDlgItem((IDD_FONTSTOOLBAR as i32));
        let bChecked: BOOL = SendMessageW(hwndToolbar, TB_ISBUTTONCHECKED, id as WPARAM, 0) as BOOL;
    
        match id {
            IDCANCEL => {
                (*this_).__base0.ShowWindow((SW_HIDE) as i32);
                registrySettings.ShowTextTool = FALSE;
            }
            IDD_FONTSNAMES => {
                (*this_).OnFontName(codeNotify);
            }
            IDD_FONTSSIZES => {
                (*this_).OnFontSize(codeNotify);
            }
            IDM_BOLD => {
                registrySettings.Bold = (bChecked) as u32;
                toolsModel.NotifyToolChanged();
            }
            IDM_ITALIC => {
                registrySettings.Italic = (bChecked) as u32;
                toolsModel.NotifyToolChanged();
            }
            IDM_UNDERLINE => {
                registrySettings.Underline = (bChecked) as u32;
                toolsModel.NotifyToolChanged();
            }
            IDM_VERTICAL => {
                // TODO:
            }
            _ => {}
        }
        return 0;
    }
    pub unsafe fn OnMove(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut rc: RECT = core::mem::zeroed();
        (*this_).__base0.GetWindowRect(core::ptr::addr_of_mut!(rc));
        registrySettings.FontsPositionX = (rc.left) as u32;
        registrySettings.FontsPositionY = (rc.top) as u32;
        return 0;
    }
    pub unsafe fn OnNotify(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let pnmhdr: *mut NMHDR = lParam as *mut NMHDR;
        if (*pnmhdr).code == TTN_NEEDTEXT {
            let pToolTip: LPTOOLTIPTEXTW = lParam as LPTOOLTIPTEXTW;
            (*pToolTip).hinst = g_hinstExe;
            match (*pnmhdr).idFrom {
                IDM_BOLD => {
                    (*pToolTip).lpszText = MAKEINTRESOURCEW!(IDS_BOLD);
                }
                IDM_ITALIC => {
                    (*pToolTip).lpszText = MAKEINTRESOURCEW!(IDS_ITALIC);
                }
                IDM_UNDERLINE => {
                    (*pToolTip).lpszText = MAKEINTRESOURCEW!(IDS_UNDERLINE);
                }
                IDM_VERTICAL => {
                    (*pToolTip).lpszText = MAKEINTRESOURCEW!(IDS_VERTICAL);
                }
                _ => {}
            }
        }
        return 0;
    }
    pub unsafe fn OnToolsModelToolChanged(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if wParam != (TOOL_TEXT) as _ {
            (*this_).__base0.ShowWindow((SW_HIDE) as i32);
        }
    
        return 0;
    }
    pub unsafe fn OnMeasureItem(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        if wParam == (IDD_FONTSNAMES as u64) {
            let pMeasureItem: LPMEASUREITEMSTRUCT = lParam as LPMEASUREITEMSTRUCT;
            let mut rc: RECT = core::mem::zeroed();
            GetClientRect((*this_).__base0.GetDlgItem((IDD_FONTSNAMES as i32)), core::ptr::addr_of_mut!(rc));
            (*pMeasureItem).itemWidth = (rc.right - rc.left) as u32;
            (*pMeasureItem).itemHeight = (GetSystemMetrics((SM_CYVSCROLL) as i32)) as u32;
            return (TRUE) as LRESULT;
        }
        return 0;
    }
    pub unsafe fn OnDrawItem(&mut self, nMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        // TODO: Owner-draw the font types
        if wParam == (IDD_FONTSNAMES as u64) {
            let pDrawItem: LPDRAWITEMSTRUCT = lParam as LPDRAWITEMSTRUCT;
            if (*pDrawItem).itemID == ((-(1)) as UINT) {
                return (TRUE) as LRESULT;
            }
    
            SetBkMode((*pDrawItem).hDC, (TRANSPARENT as i32));
    
            let hwndItem: HWND = (*pDrawItem).hwndItem;
            let mut rcItem: RECT = (*pDrawItem).rcItem;
            if ((*pDrawItem).itemState & ODS_SELECTED) != 0 {
                FillRect((*pDrawItem).hDC, &mut rcItem, GetSysColorBrush((COLOR_HIGHLIGHT) as i32));
                SetTextColor((*pDrawItem).hDC, GetSysColor((COLOR_HIGHLIGHTTEXT) as i32));
            } else {
                FillRect((*pDrawItem).hDC, &mut rcItem, GetSysColorBrush((COLOR_WINDOW) as i32));
                SetTextColor((*pDrawItem).hDC, GetSysColor((COLOR_WINDOWTEXT) as i32));
            }
    
            let mut szText = [0 as WCHAR; (LF_FACESIZE as usize)];
            if ((ComboBox_GetLBTextLen!(hwndItem, (*pDrawItem).itemID)) as UINT) < (_countof!(szText) as UINT) {
                szText[0] = 0;
                ComboBox_GetLBText!(hwndItem, (*pDrawItem).itemID, szText.as_mut_ptr());
    
                rcItem.left += 24;
                DrawTextW(
                    (*pDrawItem).hDC,
                    szText.as_ptr(),
                    (-(1)) as i32,
                    &mut rcItem,
                    (DT_LEFT | DT_VCENTER | DT_SINGLELINE) as UINT,
                );
            }
    
            if ((*pDrawItem).itemState & ODS_FOCUS) != 0 {
                DrawFocusRect((*pDrawItem).hDC, &mut (*pDrawItem).rcItem);
            }
    
            return (TRUE) as LRESULT;
        }
        return 0;
    }
    pub unsafe fn OnFontSize(&mut self, codeNotify: UINT) {
        let this_: *mut Self = self;
        let hwndSizes: HWND = (*this_).__base0.GetDlgItem((IDD_FONTSSIZES as i32));
        let mut szText: [WCHAR; 8] = [0; 8];
        let mut iItem: INT;
        let mut PointSize: INT = 0;
        let mut cch: UINT;
    
        match codeNotify {
            CBN_SELCHANGE => {
                iItem = ComboBox_GetCurSel!(hwndSizes);
                cch = ComboBox_GetLBTextLen!(hwndSizes, iItem);
                if iItem != CB_ERR && 0 < cch && cch < _countof!(szText) as UINT {
                    ComboBox_GetLBText!(hwndSizes, iItem, szText.as_mut_ptr());
                    PointSize = _wtoi(szText.as_ptr());
                }
            }
    
            CBN_EDITCHANGE => {
                GetWindowTextW(hwndSizes, szText.as_mut_ptr(), _countof!(szText) as INT);
                PointSize = _wtoi(szText.as_ptr());
            }
    
            _ => {}
        }
    
        if PointSize > 0 {
            registrySettings.PointSize = (PointSize) as u32;
            toolsModel.NotifyToolChanged();
        }
    }
    pub unsafe fn OnFontName(&mut self, codeNotify: UINT) {
        let this_: *mut Self = self;
        let hwndNames: HWND = (*this_).__base0.GetDlgItem((IDD_FONTSNAMES as i32));
        let mut iItem: INT = CB_ERR;
        let mut cch: UINT;
        let mut szText: [WCHAR; (LF_FACESIZE as usize)] = core::mem::zeroed();
    
        if codeNotify == CBN_SELCHANGE {
            iItem = ComboBox_GetCurSel!(hwndNames);
            cch = ComboBox_GetLBTextLen!(hwndNames, iItem);
            if iItem != CB_ERR && 0 < cch && cch < _countof!(szText) {
                ComboBox_GetLBText!(hwndNames, iItem, szText.as_mut_ptr());
            }
        } else if codeNotify == CBN_EDITCHANGE {
            (*this_).__base0.GetDlgItemTextW__0((IDD_FONTSNAMES as i32), szText.as_mut_ptr(), (_countof!(szText)) as i32);
            iItem = ComboBox_FindStringExact!(hwndNames, -1, szText.as_ptr());
        }
    
        if iItem != CB_ERR && registrySettings.strFontName.CompareNoCase(szText.as_ptr()) != 0 {
            registrySettings.strFontName = szText.as_ptr();
            toolsModel.NotifyToolChanged();
        }
    }
}
