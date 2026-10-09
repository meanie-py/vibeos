use crate::console::console::ApplyConsoleInfo;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_12980D763DDF: [u16; 31] = [
    0x0022u16, 0x0025, 0x0073, 0x0022, 0x0020, 0x0069, 0x0073, 0x0020, 0x006E, 0x006F,
    0x0074, 0x0020, 0x0061, 0x0020, 0x0076, 0x0061, 0x006C, 0x0069, 0x0064, 0x0020,
    0x0066, 0x006F, 0x006E, 0x0074, 0x0020, 0x0073, 0x0069, 0x007A, 0x0065, 0x002E,
    0x0000,
];
pub static RUSTCONV_WSTR_6C49BEACD7CB: [u16; 6] = [0x0045u16, 0x0072, 0x0072, 0x006F, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_9768A8408F0A: [u16; 8] = [0x0025u16, 0x0064, 0x0020, 0x0078, 0x0020, 0x0025, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_A9EDBA2FC474: [u16; 3] = [0x0025u16, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_CD5CA8EB63BB: [u16; 9] = [0x0054u16, 0x0065, 0x0072, 0x006D, 0x0069, 0x006E, 0x0061, 0x006C, 0x0000];

use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> u32;
}
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut FontPreview: FONT_PREVIEW = _FONT_PREVIEW {
    hFont: ::core::ptr::null::<HFONT__>() as *mut HFONT__,
    CharWidth: 0 as UINT,
    CharHeight: 0 as UINT,
};

static mut TrueTypePoints: [SHORT; 19] = [
    5 as ::core::ffi::c_int as SHORT,
    6 as ::core::ffi::c_int as SHORT,
    7 as ::core::ffi::c_int as SHORT,
    8 as ::core::ffi::c_int as SHORT,
    9 as ::core::ffi::c_int as SHORT,
    10 as ::core::ffi::c_int as SHORT,
    11 as ::core::ffi::c_int as SHORT,
    12 as ::core::ffi::c_int as SHORT,
    14 as ::core::ffi::c_int as SHORT,
    16 as ::core::ffi::c_int as SHORT,
    18 as ::core::ffi::c_int as SHORT,
    20 as ::core::ffi::c_int as SHORT,
    22 as ::core::ffi::c_int as SHORT,
    24 as ::core::ffi::c_int as SHORT,
    26 as ::core::ffi::c_int as SHORT,
    28 as ::core::ffi::c_int as SHORT,
    36 as ::core::ffi::c_int as SHORT,
    48 as ::core::ffi::c_int as SHORT,
    72 as ::core::ffi::c_int as SHORT,
];

static mut CurrentSelFont: INT = LB_ERR;

static mut CurrentFontType: DWORD = -(1 as ::core::ffi::c_int) as DWORD;

static mut CurrentCodePage: UINT = INVALID_CP;

// #define NDEBUG: no Rust equivalent, omitted.

#[repr(C)]
pub struct _FONTSIZE_LIST_CTL {
    pub RasterSizeList: LIST_CTL,   // ListBox for Raster font sizes; needs to handle bisection.
    pub hWndTTSizeList: HWND,       // ComboBox for TrueType font sizes.
    pub bIsTTSizeDirty: BOOL,       // TRUE or FALSE depending on whether we have edited the edit zone.
    pub UseRasterOrTTList: BOOL,    // TRUE: Use the Raster size list; FALSE: Use the TrueType size list.
    pub TTSizePixelUnit: BOOL,      // TRUE: Size in pixels (default); FALSE: Size in points.
    pub CurrentRasterSize: LONG,
    pub CurrentTTSize: LONG,        // In whatever unit (pixels or points) currently selected.
}
pub type FONTSIZE_LIST_CTL = _FONTSIZE_LIST_CTL;
pub type PFONTSIZE_LIST_CTL = *mut FONTSIZE_LIST_CTL;

#[repr(C)]
pub struct _FACE_NAMES_PROC_PARAM {
    pub hWndList: HWND,
    pub CodePage: UINT,
}
pub type FACE_NAMES_PROC_PARAM = _FACE_NAMES_PROC_PARAM;
pub type PFACE_NAMES_PROC_PARAM = *mut FACE_NAMES_PROC_PARAM;

static EnumFaceNamesProc_unused_marker: () = ();
#[no_mangle]
pub unsafe extern "C" fn RefreshFontPreview(mut Preview: *mut FONT_PREVIEW, mut pConInfo: PCONSOLE_STATE_INFO) {
    let mut hFont: HFONT = CreateConsoleFont(pConInfo);
    if hFont.is_null() {
        if DbgPrint(
            b"(%s:%d) RefreshFontPreview: CreateConsoleFont() failed\n\0" as *const u8 as PCSTR,
            b"dll/cpl/console/font.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            59 as ::core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"dll/cpl/console/font.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                59 as ::core::ffi::c_int,
            );
        }
        return;
    }
    if !(*Preview).hFont.is_null() {
        DeleteObject((*Preview).hFont as HGDIOBJ);
    }
    (*Preview).hFont = hFont;
    GetFontCellSize(
        core::ptr::null_mut(),
        hFont,
        core::ptr::addr_of_mut!((*Preview).CharHeight),
        core::ptr::addr_of_mut!((*Preview).CharWidth),
    );
}
#[no_mangle]
pub unsafe extern "C" fn UpdateFontPreview(
    Preview: *mut FONT_PREVIEW,
    hFont: HFONT,
    CharWidth: UINT,
    CharHeight: UINT,
) {
    if !(*Preview).hFont.is_null() {
        DeleteObject((*Preview).hFont as HGDIOBJ);
    }
    (*Preview).hFont = hFont;
    (*Preview).CharWidth = CharWidth;
    (*Preview).CharHeight = CharHeight;
}

unsafe extern "C" fn RasterSizeList_GetCount(mut ListCtl: PLIST_CTL) -> INT {
    return SendMessageW((*ListCtl).hWndList, LB_GETCOUNT as UINT, 0 as WPARAM, 0 as LPARAM) as INT;
}

unsafe extern "C" fn RasterSizeList_GetData(mut ListCtl: PLIST_CTL, mut Index: INT) -> ULONG_PTR {
    return SendMessageW(
        (*ListCtl).hWndList,
        LB_GETITEMDATA as UINT,
        Index as WPARAM,
        0 as LPARAM,
    ) as ULONG_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn LogicalSizeToPointSize(mut hDC: HDC, mut LogicalSize: UINT) -> INT {
    let mut PointSize: INT = 0;
    let mut hOrgDC: HDC = hDC;

    if hDC.is_null() {
        hDC = GetDC(core::ptr::null_mut());
    }

    // LogicalSize = tm.tmHeight - tm.tmInternalLeading;
    PointSize = MulDiv(LogicalSize as INT, 72 as INT, GetDeviceCaps(hDC, LOGPIXELSY as INT)) as INT;

    if hOrgDC.is_null() {
        ReleaseDC(core::ptr::null_mut(), hDC);
    }

    return PointSize;
}
#[no_mangle]
pub unsafe extern "C" fn PointSizeToLogicalSize(mut hDC: HDC, mut PointSize: INT) -> INT {
    let mut LogicalSize: INT;
    let mut hOrgDC: HDC = hDC;
    if hDC.is_null() {
        hDC = GetDC(core::ptr::null_mut());
    }
    LogicalSize = MulDiv(PointSize, GetDeviceCaps(hDC, (LOGPIXELSY as i32)), 72);
    if hOrgDC.is_null() {
        ReleaseDC(core::ptr::null_mut(), hDC);
    }
    return LogicalSize;
}

unsafe extern "C" fn FontSizeList_SelectFontSize(
    SizeList: PFONTSIZE_LIST_CTL,
    FontSize: ULONG,
) {
    let mut nSel: INT = 0;
    let mut szFontSize: [WCHAR; 100] = [0; 100];

    if (*SizeList).UseRasterOrTTList != 0 {
        let mut idx: INT = 0;

        (*SizeList).CurrentRasterSize = FontSize as LONG;

        nSel = BisectListSortedByValue(
            core::ptr::addr_of_mut!((*SizeList).RasterSizeList),
            FontSize as ULONG_PTR,
            core::ptr::null_mut(),
            FALSE as BOOL,
        ) as INT;
        idx = SendMessageW(
            (*SizeList).RasterSizeList.hWndList,
            LB_GETCOUNT as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as INT;
        if nSel == LB_ERR {
            /* Not found, select the first element of the list */
            nSel = 0;
        } else if nSel >= idx {
            /*
             * We got an index beyond the end of the list (as per Bisect* functionality),
             * so instead, select the last element of the list.
             */
            nSel = idx - 1 as INT;
        }
        SendMessageW(
            (*SizeList).RasterSizeList.hWndList,
            LB_SETCURSEL as UINT,
            nSel as WPARAM,
            0 as LPARAM,
        );
    } else {
        /* TrueType font size (in pixels or points) */
        (*SizeList).CurrentTTSize = FontSize as LONG;

        // _ultow(szFontSize, FontSize, 10);
        StringCchPrintfW!(
            core::ptr::addr_of_mut!(szFontSize) as *mut WCHAR,
            ARRAYSIZE!(szFontSize) as size_t,
            RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(),
            FontSize,
        );

        /* Find the font size in the list, or add it both in the ComboBox list, sorted by size value (string), and its edit box */
        nSel = SendMessageW(
            (*SizeList).hWndTTSizeList,
            CB_FINDSTRINGEXACT as UINT,
            0 as WPARAM,
            core::ptr::addr_of_mut!(szFontSize) as *mut WCHAR as LPARAM,
        ) as INT;
        if nSel == CB_ERR {
            nSel = SendMessageW(
                (*SizeList).hWndTTSizeList,
                CB_ADDSTRING as UINT,
                (-(1 as INT)) as WPARAM,
                core::ptr::addr_of_mut!(szFontSize) as *mut WCHAR as LPARAM,
            ) as INT;
            // ComboBox_SetText(...)
            SetWindowTextW(
                (*SizeList).hWndTTSizeList,
                core::ptr::addr_of_mut!(szFontSize) as *mut WCHAR as LPCWSTR,
            );
            (*SizeList).bIsTTSizeDirty = TRUE as BOOL;
        }
        SendMessageW(
            (*SizeList).hWndTTSizeList,
            CB_SETCURSEL as UINT,
            nSel as WPARAM,
            0 as LPARAM,
        );
    }
}

unsafe extern "C" fn FontSizeList_GetSelectedFontSize(mut SizeList: PFONTSIZE_LIST_CTL) -> LONG {
    let mut nSel: INT = 0;
    let mut FontSize: LONG = 0;
    let mut pszNext: PWCHAR = core::ptr::null_mut::<WCHAR>();
    let mut szFontSize: [WCHAR; 100] = [0; 100];

    if (*SizeList).UseRasterOrTTList != 0 {
        /* Raster font size (in pixels) */

        nSel = SendMessageW(
            (*SizeList).RasterSizeList.hWndList,
            LB_GETCURSEL as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as INT;
        if nSel == LB_ERR {
            return 0;
        }

        FontSize = (*SizeList)
            .RasterSizeList
            .GetData
            .unwrap_unchecked()(
            core::ptr::addr_of_mut!((*SizeList).RasterSizeList),
            nSel,
        ) as LONG;
        if FontSize == LB_ERR {
            return 0;
        }

        (*SizeList).CurrentRasterSize = FontSize;
    } else {
        /* TrueType font size (in pixels or points) */

        if (*SizeList).bIsTTSizeDirty == 0 {
            /*
             * The user just selected an existing size, read the ComboBox selection.
             *
             * See: https://support.microsoft.com/en-us/help/66365/how-to-process-a-cbn-selchange-notification-message (DEAD_LINK)
             * for more details.
             */
            let mut Length: INT = 0;

            nSel = SendMessageW(
                (*SizeList).hWndTTSizeList,
                CB_GETCURSEL as UINT,
                0 as WPARAM,
                0 as LPARAM,
            ) as INT;
            if nSel == CB_ERR {
                return 0;
            }

            Length = SendMessageW(
                (*SizeList).hWndTTSizeList,
                CB_GETLBTEXTLEN as UINT,
                nSel as WPARAM,
                0 as LPARAM,
            ) as INT;

            Length = SendMessageW(
                (*SizeList).hWndTTSizeList,
                CB_GETLBTEXT as UINT,
                nSel as WPARAM,
                szFontSize.as_mut_ptr() as LPARAM,
            ) as INT;
            szFontSize[Length as usize] = 0x0000u16;

            /* Validate the font size */
            FontSize = wcstoul(
                szFontSize.as_mut_ptr(),
                core::ptr::addr_of_mut!(pszNext),
                10,
            ) as LONG;
            if (FontSize == 0) || (*pszNext != 0) {
                return 0;
            }
        } else {
            /* Read the ComboBox edit string, as the user has entered a custom size */
            // ComboBox_GetText(...)
            GetWindowTextW(
                (*SizeList).hWndTTSizeList,
                szFontSize.as_mut_ptr(),
                (core::mem::size_of::<[WCHAR; 100]>()
                    / core::mem::size_of::<WCHAR>()) as INT,
            );

            /* Validate the font size */
            FontSize = wcstoul(
                szFontSize.as_mut_ptr(),
                core::ptr::addr_of_mut!(pszNext),
                10,
            ) as LONG;
            if (FontSize == 0) || (*pszNext != 0) {
                return 0;
            }

            /* Find if the font size already exists in the list; if not, add it */
            nSel = SendMessageW(
                (*SizeList).hWndTTSizeList,
                CB_FINDSTRINGEXACT as UINT,
                0 as WPARAM,
                szFontSize.as_mut_ptr() as LPARAM,
            ) as INT;
            if nSel == CB_ERR {
                nSel = SendMessageW(
                    (*SizeList).hWndTTSizeList,
                    CB_ADDSTRING as UINT,
                    -1i32 as WPARAM,
                    szFontSize.as_mut_ptr() as LPARAM,
                ) as INT;
                //// ComboBox_SetText(...)
                //SetWindowTextW(SizeList->hWndTTSizeList, szFontSize);
                //SizeList->bIsTTSizeDirty = TRUE;
            }
            SendMessageW(
                (*SizeList).hWndTTSizeList,
                CB_SETCURSEL as UINT,
                nSel as WPARAM,
                0 as LPARAM,
            );
        }

        (*SizeList).bIsTTSizeDirty = (FALSE) as i32;

        (*SizeList).CurrentTTSize = FontSize;

        /*
         * If the font size is given in points, instead of pixels,
         * convert it into logical size.
         */
        if (*SizeList).TTSizePixelUnit == 0 {
            FontSize = -PointSizeToLogicalSize(core::ptr::null_mut(), FontSize as INT) as LONG;
        }
    }

    return FontSize;
}

unsafe extern "C" fn AddFontToList(
    hWndList: HWND,
    pszFaceName: LPCWSTR,
    FontType: DWORD,
) {
    let mut iItem: INT;

    /* Make sure the font doesn't already exist in the list */
    if SendMessageW(
        hWndList,
        LB_FINDSTRINGEXACT as UINT,
        0 as WPARAM,
        pszFaceName as LPARAM,
    ) != LB_ERR as LRESULT
    {
        return;
    }

    /* Add the font */
    iItem = SendMessageW(
        hWndList,
        LB_ADDSTRING as UINT,
        0 as WPARAM,
        pszFaceName as LPARAM,
    ) as INT;
    if iItem == LB_ERR as INT {
        DbgPrint(
            b"Failed to add font '%S'\n\0".as_ptr() as PCSTR,
            pszFaceName,
        );
        return;
    }

    DbgPrint(
        b"Add font '%S'\n\0".as_ptr() as PCSTR,
        pszFaceName,
    );

    /* Store this information in the list-item's userdata area */
    // SendMessageW(hWndList, LB_SETITEMDATA, idx, MAKELPARAM(fFixed, fTrueType));
    SendMessageW(
        hWndList,
        LB_SETITEMDATA as UINT,
        iItem as WPARAM,
        FontType as LPARAM,
    );
}

unsafe extern "system" fn EnumFaceNamesProc(
    lplf: PLOGFONTW,
    lpntm: PNEWTEXTMETRICW,
    FontType: DWORD,
    lParam: LPARAM,
) -> BOOL {
    let Param: PFACE_NAMES_PROC_PARAM = lParam as PFACE_NAMES_PROC_PARAM;

    if IsValidConsoleFont2(lplf, lpntm, FontType, (*Param).CodePage) != 0 {
        /* Add the face name to the list */
        AddFontToList(
            (*Param).hWndList,
            core::ptr::addr_of_mut!((*lplf).lfFaceName) as *mut WCHAR as LPCWSTR,
            FontType,
        );
    }

    /* Continue enumerating the faces */
    return (TRUE) as BOOL;
}

unsafe extern "system" fn EnumFontSizesProc(
    lplf: PLOGFONTW,
    lpntm: PNEWTEXTMETRICW,
    FontType: DWORD,
    lParam: LPARAM,
) -> BOOL {
    let SizeList: PFONTSIZE_LIST_CTL = lParam as PFONTSIZE_LIST_CTL;
    let mut iItem: UINT = 0;
    let mut iDupItem: UINT = 0;
    let mut szFontSize: [WCHAR; 100] = [0; 100];

    if FontType != TRUETYPE_FONTTYPE {
        let mut FontSize: WPARAM = 0;

        StringCchPrintfW!(
            szFontSize.as_mut_ptr(),
            ARRAYSIZE!(szFontSize),
            RUSTCONV_WSTR_9768A8408F0A.as_ptr(),
            (*lplf).lfWidth,
            (*lplf).lfHeight,
        );
        FontSize = MAKEWPARAM!((*lplf).lfWidth as WORD, (*lplf).lfHeight as WORD);

        iDupItem = (LB_ERR) as u32;
        iItem = BisectListSortedByValue(
            core::ptr::addr_of_mut!((*SizeList).RasterSizeList),
            FontSize,
            core::ptr::addr_of_mut!(iDupItem),
            (TRUE) as BOOL,
        );
        if iItem == (LB_ERR) as u32 {
            iItem = 0;
        }
        if iDupItem == (LB_ERR) as u32 {
            iItem = SendMessageW(
                (*SizeList).RasterSizeList.hWndList,
                LB_INSERTSTRING,
                iItem as WPARAM,
                szFontSize.as_mut_ptr() as LPARAM,
            ) as UINT;
            if iItem != (LB_ERR) as u32 && iItem != (LB_ERRSPACE) as u32 {
                iItem = SendMessageW(
                    (*SizeList).RasterSizeList.hWndList,
                    LB_SETITEMDATA,
                    iItem as WPARAM,
                    FontSize as LPARAM,
                ) as UINT;
            }
        }

        return (TRUE) as BOOL;
    } else {
        let mut i: ULONG = 0;
        i = 0;
        while i < ARRAYSIZE!(TrueTypePoints) as ULONG {
            StringCchPrintfW!(
                szFontSize.as_mut_ptr(),
                ARRAYSIZE!(szFontSize),
                RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(),
                TrueTypePoints[i as usize] as ::core::ffi::c_int,
            );

            if SendMessageW(
                (*SizeList).hWndTTSizeList,
                CB_FINDSTRINGEXACT,
                0,
                szFontSize.as_mut_ptr() as LPARAM,
            ) == (CB_ERR) as _
            {
                iItem = SendMessageW(
                    (*SizeList).hWndTTSizeList,
                    CB_INSERTSTRING,
                    -(1 as ::core::ffi::c_int) as WPARAM,
                    szFontSize.as_mut_ptr() as LPARAM,
                ) as UINT;
            }
            i += 1;
        }

        return (FALSE) as BOOL;
    }
}

unsafe extern "C" fn FaceNameList_Initialize(hWndList: HWND, CodePage: UINT) {
    let mut Param: FACE_NAMES_PROC_PARAM = FACE_NAMES_PROC_PARAM {
        hWndList: core::ptr::null_mut(),
        CodePage: 0,
    };
    let mut hDC: HDC = core::ptr::null_mut();
    let mut lf: LOGFONTW = core::mem::zeroed();
    let mut idx: INT = 0;

    /* Reset the face names list */
    SendMessageW(hWndList, LB_RESETCONTENT, 0, 0);

    Param.hWndList = hWndList;
    Param.CodePage = CodePage;

    ZeroMemory!(
        core::ptr::addr_of_mut!(lf) as *mut ::core::ffi::c_void,
        core::mem::size_of::<LOGFONTW>(),
    );
    lf.lfCharSet = (DEFAULT_CHARSET as u8); // CodePageToCharSet(CodePage);
    // lf.lfPitchAndFamily = FIXED_PITCH | FF_MODERN;

    hDC = GetDC(core::ptr::null_mut());
    EnumFontFamiliesExW(
        hDC,
        core::ptr::addr_of_mut!(lf),
        core::mem::transmute::<
            unsafe extern "system" fn(PLOGFONTW, PNEWTEXTMETRICW, DWORD, LPARAM) -> BOOL,
            FONTENUMPROCW,
        >(EnumFaceNamesProc),
        core::ptr::addr_of_mut!(Param) as LPARAM,
        0,
    );
    ReleaseDC(core::ptr::null_mut(), hDC);

    idx = SendMessageW(hWndList, LB_GETCOUNT, 0, 0) as INT;
    if idx != LB_ERR && idx != 0 {
        /* We have found some faces and filled the list, we are fine! */
        return;
    }

    /* No fonts were found. Manually add default ones into the list. */
    if DbgPrint(
        b"(%s:%d) The ideal console fonts were not found; manually add default ones.\n\0".as_ptr() as PCSTR,
        b"dll/cpl/console/font.c\0".as_ptr() as *const ::core::ffi::c_char,
        429 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"dll/cpl/console/font.c\0".as_ptr() as *const ::core::ffi::c_char,
            429 as ::core::ffi::c_int,
        );
    }

    AddFontToList(hWndList, RUSTCONV_WSTR_CD5CA8EB63BB.as_ptr(), RASTER_FONTTYPE);
}

unsafe extern "C" fn FaceNameList_SelectFont(
    hDlg: HWND,
    hWndList: HWND,
    SizeList: PFONTSIZE_LIST_CTL,
    FaceName: LPCWSTR,
    FontFamily: ULONG,
    FontWeight: ULONG,
    FontSize: COORD,
) {
    let mut iItem: INT;

    iItem = SendMessageW(hWndList, LB_FINDSTRINGEXACT as UINT, 0 as WPARAM, FaceName as LPARAM) as INT;
    if iItem == LB_ERR as INT {
        iItem = SendMessageW(hWndList, LB_FINDSTRINGEXACT as UINT, 0 as WPARAM, RUSTCONV_WSTR_CD5CA8EB63BB.as_ptr() as LPARAM) as INT;
    }
    if iItem == LB_ERR as INT {
        iItem = 0 as INT;
    }
    SendMessageW(hWndList, LB_SETCURSEL as UINT, iItem as WPARAM, 0 as LPARAM);

    if FontWeight >= FW_BOLD as ULONG {
        CheckDlgButton(hDlg, IDC_CHECK_BOLD_FONTS as INT, BST_CHECKED as UINT);
    } else {
        CheckDlgButton(hDlg, IDC_CHECK_BOLD_FONTS as INT, BST_UNCHECKED as UINT);
    }

    (*SizeList).CurrentRasterSize = MAKELONG!(FontSize.X as WORD, FontSize.Y as WORD) as LONG;
    (*SizeList).CurrentTTSize = FontSize.Y as LONG;
}

unsafe extern "C" fn UpdateFontSizeList(hDlg: HWND, SizeList: PFONTSIZE_LIST_CTL) {
    let mut hDlgItem: HWND = core::ptr::null_mut();

    if (*SizeList).UseRasterOrTTList != 0 {
        /*
         * Raster font: show the Raster size list, and
         * hide the TrueType size list and the units.
         */

        // EnableDlgItem(hDlg, IDC_CHECK_BOLD_FONTS, FALSE);

        hDlgItem = GetDlgItem(hDlg, (IDC_RADIO_PIXEL_UNIT as i32));
        ShowWindow(hDlgItem, (SW_HIDE) as i32);
        EnableWindow(hDlgItem, (FALSE) as BOOL);

        hDlgItem = GetDlgItem(hDlg, (IDC_RADIO_POINT_UNIT as i32));
        ShowWindow(hDlgItem, (SW_HIDE) as i32);
        EnableWindow(hDlgItem, (FALSE) as BOOL);

        hDlgItem = (*SizeList).hWndTTSizeList;
        ShowWindow(hDlgItem, (SW_HIDE) as i32);
        EnableWindow(hDlgItem, (FALSE) as BOOL);

        hDlgItem = (*SizeList).RasterSizeList.hWndList;
        EnableWindow(hDlgItem, (TRUE) as BOOL);
        ShowWindow(hDlgItem, (SW_SHOW) as i32);
    } else {
        /*
         * TrueType font: show the TrueType size list
         * and the units, and hide the Raster size list.
         */

        // EnableDlgItem(hDlg, IDC_CHECK_BOLD_FONTS, TRUE);

        hDlgItem = (*SizeList).RasterSizeList.hWndList;
        ShowWindow(hDlgItem, (SW_HIDE) as i32);
        EnableWindow(hDlgItem, (FALSE) as BOOL);

        hDlgItem = (*SizeList).hWndTTSizeList;
        EnableWindow(hDlgItem, (TRUE) as BOOL);
        ShowWindow(hDlgItem, (SW_SHOW) as i32);

        hDlgItem = GetDlgItem(hDlg, (IDC_RADIO_PIXEL_UNIT as i32));
        EnableWindow(hDlgItem, (TRUE) as BOOL);
        ShowWindow(hDlgItem, (SW_SHOW) as i32);

        hDlgItem = GetDlgItem(hDlg, (IDC_RADIO_POINT_UNIT as i32));
        EnableWindow(hDlgItem, (TRUE) as BOOL);
        ShowWindow(hDlgItem, (SW_SHOW) as i32);
    }
}

unsafe extern "C" fn FontTypeChange(
    mut hDlg: HWND,
    mut SizeList: PFONTSIZE_LIST_CTL,
    mut pConInfo: PCONSOLE_STATE_INFO,
) -> BOOL {
    let mut hFontList: HWND = GetDlgItem(hDlg, (IDC_LBOX_FONTTYPE as i32));
    let mut Length: INT = 0;
    let mut nSel: INT = 0;
    let mut FaceName: LPWSTR = core::ptr::null_mut();
    let mut FontType: DWORD = 0;
    let mut FontGrpBoxLabelTpl: LPCWSTR = core::ptr::null();
    let mut FontGrpBoxLabel: [WCHAR; 260] = [0; 260];

    nSel = SendMessageW(hFontList, LB_GETCURSEL as UINT, 0 as WPARAM, 0 as LPARAM) as INT;
    if nSel == LB_ERR {
        return (FALSE) as BOOL;
    }

    /*
     * This is disabled, because there can be external parameters
     * that may have changed (e.g. ConInfo->FontWeight, code page, ...)
     * and that we don't control here, and that need a font refresh.
     */

    Length = SendMessageW(hFontList, LB_GETTEXTLEN as UINT, nSel as WPARAM, 0 as LPARAM) as INT;
    if Length == LB_ERR {
        return (FALSE) as BOOL;
    }

    FaceName = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (Length + 1) as SIZE_T * (core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if FaceName.is_null() {
        return (FALSE) as BOOL;
    }

    Length = SendMessageW(hFontList, LB_GETTEXT as UINT, nSel as WPARAM, FaceName as LPARAM) as INT;

    *FaceName.offset(Length as isize) = 0x0000u16;

    StringCchCopyW(
        core::ptr::addr_of_mut!((*pConInfo).FaceName) as LPWSTR,
        (ARRAYSIZE!((*pConInfo).FaceName)) as _,
        FaceName,
    );
    DPRINT!(
        b"pConInfo->FaceName = '%S'\n\0".as_ptr() as *const i8,
        core::ptr::addr_of_mut!((*pConInfo).FaceName) as LPWSTR,
    );

    /*
     * Retrieve the read-only font group box label string template,
     * and set the group box label to the name of the selected font.
     */
    Length = LoadStringW(
        hApplet,
        IDS_GROUPBOX_FONT_NAME as UINT,
        core::ptr::addr_of_mut!(FontGrpBoxLabelTpl) as LPWSTR,
        0 as INT,
    ) as INT;
    if !FontGrpBoxLabelTpl.is_null() && Length > 0 {
        StringCchCopyNW(
            core::ptr::addr_of_mut!(FontGrpBoxLabel) as LPWSTR,
            (ARRAYSIZE!(FontGrpBoxLabel)) as _,
            FontGrpBoxLabelTpl,
            Length as size_t,
        );
        StringCchCatW(
            core::ptr::addr_of_mut!(FontGrpBoxLabel) as LPWSTR,
            (ARRAYSIZE!(FontGrpBoxLabel)) as _,
            FaceName,
        );
        SetDlgItemTextW(
            hDlg,
            IDC_GROUPBOX_FONT_NAME as INT,
            core::ptr::addr_of_mut!(FontGrpBoxLabel) as *mut WCHAR as LPCWSTR,
        );
    }

    HeapFree(GetProcessHeap(), 0 as DWORD, FaceName as PVOID);

    /*
     * Reset the font size list, only:
     * - if we have changed the type of font, or
     * - if the font type is the same and is RASTER but the font has changed.
     * Otherwise, if the font type is not RASTER and has not changed,
     * we always display the TrueType default sizes and we don't need to
     * recreate the list when we change between different TrueType fonts.
     */
    FontType = SendMessageW(hFontList, LB_GETITEMDATA as UINT, nSel as WPARAM, 0 as LPARAM) as DWORD;
    if FontType != LB_ERR as DWORD {
        (*SizeList).UseRasterOrTTList = (FontType == RASTER_FONTTYPE as DWORD) as BOOL;

        /* Display the correct font size list (if needed) */
        if CurrentFontType != FontType {
            UpdateFontSizeList(hDlg, SizeList);
        }

        /* Enumerate the available sizes for the selected font */
        if (CurrentFontType != FontType)
            || (FontType == RASTER_FONTTYPE as DWORD && CurrentSelFont != nSel)
        {
            let mut lf: LOGFONTW = core::mem::zeroed();
            let mut hDC: HDC = core::ptr::null_mut();

            if (*SizeList).UseRasterOrTTList != 0 {
                SendMessageW(
                    (*SizeList).RasterSizeList.hWndList,
                    LB_RESETCONTENT as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                );
            } else {
                SendMessageW(
                    (*SizeList).hWndTTSizeList,
                    CB_RESETCONTENT as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                );
            }

            ZeroMemory!(core::ptr::addr_of_mut!(lf) as *mut core::ffi::c_void, core::mem::size_of::<LOGFONTW>() as size_t);
            lf.lfCharSet = DEFAULT_CHARSET as BYTE; // CodePageToCharSet(pConInfo->CodePage);
            // lf.lfPitchAndFamily = FIXED_PITCH | FF_MODERN;
            StringCchCopyW(
                core::ptr::addr_of_mut!(lf.lfFaceName) as LPWSTR,
                (ARRAYSIZE!(lf.lfFaceName)) as _,
                core::ptr::addr_of_mut!((*pConInfo).FaceName) as *mut WCHAR as LPCWSTR,
            );

            hDC = GetDC(core::ptr::null_mut());
            let enum_proc_src: unsafe extern "system" fn(*mut tagLOGFONTW, *mut tagNEWTEXTMETRICW, u32, LPARAM) -> i32 = EnumFontSizesProc;
            let enum_proc: Option<unsafe extern "system" fn(*const tagLOGFONTW, *const tagTEXTMETRICW, u32, LPARAM) -> i32> =
                *(core::ptr::addr_of!(enum_proc_src) as *const Option<unsafe extern "system" fn(*const tagLOGFONTW, *const tagTEXTMETRICW, u32, LPARAM) -> i32>);
            EnumFontFamiliesExW(
                hDC,
                core::ptr::addr_of_mut!(lf),
                enum_proc,
                SizeList as LPARAM,
                0 as DWORD,
            );
            ReleaseDC(core::ptr::null_mut(), hDC);

            /* Re-select the current font size */
            if (*SizeList).UseRasterOrTTList != 0 {
                FontSizeList_SelectFontSize(SizeList, (*SizeList).CurrentRasterSize as ULONG);
            } else {
                FontSizeList_SelectFontSize(SizeList, (*SizeList).CurrentTTSize as ULONG);
            }
        }
    } else {
        /* We failed, display the raster fonts size list */
        (*SizeList).UseRasterOrTTList = TRUE as BOOL;
        UpdateFontSizeList(hDlg, SizeList);
    }
    CurrentFontType = FontType;
    CurrentSelFont = nSel;

    FontSizeChange(hDlg, SizeList, pConInfo);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn FontSizeChange(
    mut hDlg: HWND,
    mut SizeList: PFONTSIZE_LIST_CTL,
    mut pConInfo: PCONSOLE_STATE_INFO,
) -> BOOL {
    let mut FontSize: LONG = 0;
    let mut CharWidth: UINT = 0;
    let mut CharHeight: UINT = 0;
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut szFontSize: [WCHAR; 100] = [0; 100];

    FontSize = FontSizeList_GetSelectedFontSize(SizeList);
    if FontSize == 0 {
        return (FALSE) as BOOL;
    }

    CharHeight = if (*SizeList).UseRasterOrTTList != 0 {
        HIWORD!(FontSize as DWORD) as UINT
    } else {
        FontSize as UINT
    };
    CharWidth = if (*SizeList).UseRasterOrTTList != 0 {
        LOWORD!(FontSize as DWORD) as UINT
    } else {
        0 as UINT
    };

    hFont = CreateConsoleFont2(CharHeight as LONG, CharWidth as LONG, pConInfo);
    if hFont.is_null() {
        DPRINT1!(b"FontSizeChange: CreateConsoleFont2() failed\n\0".as_ptr() as *const i8);
        return (FALSE) as BOOL;
    }

    GetFontCellSize(
        core::ptr::null_mut(),
        hFont,
        core::ptr::addr_of_mut!(CharHeight),
        core::ptr::addr_of_mut!(CharWidth),
    );

    UpdateFontPreview(
        core::ptr::addr_of_mut!(FontPreview),
        hFont,
        CharWidth,
        CharHeight,
    );

    (*pConInfo).FontSize.X = (if (*SizeList).UseRasterOrTTList != 0 {
        CharWidth
    } else {
        0 as UINT
    }) as SHORT;
    (*pConInfo).FontSize.Y = CharHeight as SHORT;

    DPRINT!(
        b"pConInfo->FontSize = (%d x %d) ; (CharWidth x CharHeight) = (%d x %d)\n\0".as_ptr()
            as *const i8,
        (*pConInfo).FontSize.X as i32,
        (*pConInfo).FontSize.Y as i32,
        CharWidth,
        CharHeight,
    );

    InvalidateRect(
        GetDlgItem(hDlg, (IDC_STATIC_FONT_WINDOW_PREVIEW as i32)),
        core::ptr::null(),
        (TRUE) as BOOL,
    );
    InvalidateRect(
        GetDlgItem(hDlg, (IDC_STATIC_SELECT_FONT_PREVIEW as i32)),
        core::ptr::null(),
        (TRUE) as BOOL,
    );

    StringCchPrintfW!(
        szFontSize.as_mut_ptr(),
        ARRAYSIZE!(szFontSize),
        RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(),
        CharWidth,
    );
    SetDlgItemText(hDlg, (IDC_FONT_SIZE_X as i32), szFontSize.as_ptr());
    StringCchPrintfW!(
        szFontSize.as_mut_ptr(),
        ARRAYSIZE!(szFontSize),
        RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(),
        CharHeight,
    );
    SetDlgItemText(hDlg, (IDC_FONT_SIZE_Y as i32), szFontSize.as_ptr());

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn FontProc(
    hDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut SizeList: PFONTSIZE_LIST_CTL = GetWindowLongPtrW(hDlg, DWLP_USER) as PFONTSIZE_LIST_CTL;

    'sw: {
        match uMsg {
            WM_INITDIALOG => {
                SizeList = HeapAlloc(
                    GetProcessHeap(),
                    HEAP_ZERO_MEMORY,
                    core::mem::size_of::<_FONTSIZE_LIST_CTL>() as SIZE_T,
                ) as PFONTSIZE_LIST_CTL;
                if SizeList.is_null() {
                    EndDialog(hDlg, 0);
                    return TRUE as INT_PTR;
                }
                (*SizeList).RasterSizeList.hWndList = GetDlgItem(hDlg, (IDC_LBOX_FONTSIZE as i32));
                (*SizeList).RasterSizeList.GetCount =
                    Some(RasterSizeList_GetCount as unsafe extern "C" fn(PLIST_CTL) -> INT)
                        as PLIST_GETCOUNT;
                (*SizeList).RasterSizeList.GetData =
                    Some(RasterSizeList_GetData as unsafe extern "C" fn(PLIST_CTL, INT) -> ULONG_PTR)
                        as PLIST_GETDATA;
                (*SizeList).hWndTTSizeList = GetDlgItem(hDlg, (IDC_CBOX_FONTSIZE as i32));
                (*SizeList).bIsTTSizeDirty = (FALSE) as i32;
                SetWindowLongPtrW(hDlg, DWLP_USER, SizeList as LONG_PTR);

                /* By default show the raster font size list */
                (*SizeList).UseRasterOrTTList = (TRUE) as i32;

                /* By default show the font sizes in pixel units */
                CheckRadioButton(hDlg, (IDC_RADIO_PIXEL_UNIT as i32), (IDC_RADIO_POINT_UNIT as i32), (IDC_RADIO_PIXEL_UNIT as i32));
                (*SizeList).TTSizePixelUnit = (TRUE) as i32;

                UpdateFontSizeList(hDlg, SizeList);

                DPRINT1!(
                    b"ConInfo->FaceName = '%S'\n\0".as_ptr() as *const i8,
                    &raw mut (*ConInfo).FaceName as *mut WCHAR,
                );

                /* Face names list and current font selection will be done during PSN_SETACTIVE notification */
                // CurrentCodePage = INVALID_CP;

                return TRUE as INT_PTR;
            }

            WM_DESTROY => {
                if !SizeList.is_null() {
                    HeapFree(GetProcessHeap(), 0, (SizeList).cast());
                }
                return TRUE as INT_PTR;
            }

            WM_DRAWITEM => {
                let mut drawItem: LPDRAWITEMSTRUCT = lParam as LPDRAWITEMSTRUCT;

                if (*drawItem).CtlID == IDC_STATIC_SELECT_FONT_PREVIEW {
                    PaintText(drawItem, ConInfo, Screen);
                }

                return TRUE as INT_PTR;
            }

            WM_DISPLAYCHANGE => {
                /* Retransmit to the preview window */
                SendDlgItemMessageW(
                    hDlg,
                    (IDC_STATIC_FONT_WINDOW_PREVIEW as i32),
                    WM_DISPLAYCHANGE,
                    wParam,
                    lParam,
                );
                break 'sw;
            }

            WM_FONTCHANGE => {
                /* The pool of font resources has changed, re-enumerate the fonts */
                let mut hFontList: HWND = GetDlgItem(hDlg, (IDC_LBOX_FONTTYPE as i32));

                /* Initialize the font list */
                FaceNameList_Initialize(hFontList, (*ConInfo).CodePage as UINT);

                /* Select the current font */
                FaceNameList_SelectFont(
                    hDlg,
                    hFontList,
                    SizeList,
                    &raw mut (*ConInfo).FaceName as *mut WCHAR as LPCWSTR,
                    (*ConInfo).FontFamily,
                    (*ConInfo).FontWeight,
                    (*ConInfo).FontSize,
                );

                /* Refresh everything */
                FontTypeChange(hDlg, SizeList, ConInfo);
                break 'sw;
            }

            WM_NOTIFY => {
                'nt: {
                    match ((*(lParam as LPNMHDR)).code) as i32 {
                        PSN_APPLY => {
                            ApplyConsoleInfo(hDlg);
                            return TRUE as INT_PTR;
                        }

                        PSN_SETACTIVE => {
                            /* Check whether the current code page has changed.
                             * If so, re-enumerate the fonts. */
                            if CurrentCodePage as ULONG != (*ConInfo).CodePage {
                                let mut hFontList: HWND;

                                /* Save the new code page */
                                CurrentCodePage = (*ConInfo).CodePage as UINT;

                                hFontList = GetDlgItem(hDlg, (IDC_LBOX_FONTTYPE as i32));

                                /* Initialize the font list */
                                FaceNameList_Initialize(hFontList, (*ConInfo).CodePage as UINT);

                                /* Select the current font */
                                FaceNameList_SelectFont(
                                    hDlg,
                                    hFontList,
                                    SizeList,
                                    &raw mut (*ConInfo).FaceName as *mut WCHAR as LPCWSTR,
                                    (*ConInfo).FontFamily,
                                    (*ConInfo).FontWeight,
                                    (*ConInfo).FontSize,
                                );

                                /* Refresh everything */
                                FontTypeChange(hDlg, SizeList, ConInfo);
                            }

                            /* Fall back to default behaviour */
                            break 'nt;
                        }

                        _ => {}
                    }
                }

                break 'sw;
            }

            WM_COMMAND => {
                if HIWORD!(wParam) as UINT == LBN_SELCHANGE /* || CBN_SELCHANGE */ {
                    'lw1: {
                        match LOWORD!(wParam) as UINT {
                            IDC_LBOX_FONTTYPE => {
                                /* Change the property sheet state only if the font has really changed */
                                if FontTypeChange(hDlg, SizeList, ConInfo) != 0 {
                                    PropSheet_Changed!(GetParent(hDlg), hDlg);
                                }
                                break 'lw1;
                            }

                            IDC_LBOX_FONTSIZE | IDC_CBOX_FONTSIZE => {
                                /* Change the property sheet state only if the font has really changed */
                                if FontSizeChange(hDlg, SizeList, ConInfo) != 0 {
                                    PropSheet_Changed!(GetParent(hDlg), hDlg);
                                }
                                break 'lw1;
                            }

                            _ => {}
                        }
                    }
                }
                /* NOTE: CBN_EDITUPDATE is sent first, and is followed by CBN_EDITCHANGE */
                else if HIWORD!(wParam) as UINT == CBN_EDITUPDATE && LOWORD!(wParam) as UINT == IDC_CBOX_FONTSIZE {
                    let mut FontSize: ULONG = 0;
                    let mut pszNext: PWCHAR = core::ptr::null_mut::<WCHAR>();
                    let mut szFontSize: [WCHAR; 100] = [0; 100];
                    let mut szMessage: [WCHAR; 260] = [0; 260];

                    /* Read the ComboBox edit string, as the user has entered a custom size */
                    GetWindowTextW(
                        (*SizeList).hWndTTSizeList,
                        &raw mut szFontSize as LPWSTR,
                        (ARRAYSIZE!(szFontSize)) as i32,
                    );

                    /* Validate the font size */
                    FontSize = wcstoul(
                        &raw mut szFontSize as *mut WCHAR,
                        &raw mut pszNext,
                        10,
                    ) as ULONG;
                    if (FontSize == 0) || (*pszNext != 0) {
                        // FIXME: Localize!
                        StringCchPrintfW!(
                            &raw mut szMessage as STRSAFE_LPWSTR,
                            ARRAYSIZE!(szMessage),
                            RUSTCONV_WSTR_12980D763DDF.as_ptr(),
                            &raw mut szFontSize as *mut WCHAR,
                        );
                        MessageBoxW(
                            hDlg,
                            &raw mut szMessage as *mut WCHAR as LPCWSTR,
                            RUSTCONV_WSTR_6C49BEACD7CB.as_ptr(),
                            MB_ICONINFORMATION | MB_OK,
                        );
                    }
                    /**/(*SizeList).bIsTTSizeDirty = (TRUE) as i32;/**/
                }
                else if HIWORD!(wParam) as UINT == CBN_KILLFOCUS && LOWORD!(wParam) as UINT == IDC_CBOX_FONTSIZE {
                    /* Change the property sheet state only if the font has really changed */
                    if FontSizeChange(hDlg, SizeList, ConInfo) != 0 {
                        PropSheet_Changed!(GetParent(hDlg), hDlg);
                    }
                }
                else if HIWORD!(wParam) as UINT == BN_CLICKED {
                    'lw2: {
                        match LOWORD!(wParam) as UINT {
                            IDC_CHECK_BOLD_FONTS => {
                                if IsDlgButtonChecked(hDlg, (IDC_CHECK_BOLD_FONTS as i32)) == BST_CHECKED {
                                    (*ConInfo).FontWeight = FW_BOLD;
                                } else {
                                    (*ConInfo).FontWeight = FW_NORMAL;
                                }

                                FontTypeChange(hDlg, SizeList, ConInfo);
                                PropSheet_Changed!(GetParent(hDlg), hDlg);
                                break 'lw2;
                            }

                            IDC_RADIO_PIXEL_UNIT | IDC_RADIO_POINT_UNIT => {
                                (*SizeList).TTSizePixelUnit =
                                    (LOWORD!(wParam) as UINT == IDC_RADIO_PIXEL_UNIT) as BOOL;

                                /* The call is valid only for TrueType fonts */
                                if CurrentFontType != TRUETYPE_FONTTYPE {
                                    break 'lw2;
                                }

                                /* Change the property sheet state only if the font has really changed */
                                if FontSizeChange(hDlg, SizeList, ConInfo) != 0 {
                                    PropSheet_Changed!(GetParent(hDlg), hDlg);
                                }
                                break 'lw2;
                            }

                            _ => {}
                        }
                    }
                }

                break 'sw;
            }

            _ => {}
        }
    }

    return FALSE as INT_PTR;
}
