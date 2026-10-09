// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_491507152B61: [u16; 13] = [
    0x004Du16, 0x0053, 0x0020, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0020, 0x0044,
    0x006C, 0x0067, 0x0000,
];
pub static RUSTCONV_WSTR_DF3F619804A9: [u16; 2] = [0x0000u16, 0x0000];

use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: *const i8, ...) -> u32;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hApplet: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

#[no_mangle]
pub static mut hTitleFont: HFONT = ::core::ptr::null::<HFONT__>() as *mut HFONT__;

#[no_mangle]
pub static mut ImageListData: SP_CLASSIMAGELIST_DATA = _SP_CLASSIMAGELIST_DATA {
    cbSize: 0,
    ImageList: ::core::ptr::null::<_IMAGELIST>() as *mut _IMAGELIST,
    Reserved: 0,
};

#[no_mangle]
pub static mut pDeviceStatusText: PWSTR = ::core::ptr::null::<WCHAR>() as *mut WCHAR;

#[no_mangle]
pub static mut hProcessHeap: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut hDevInfoTypes: HDEVINFO =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

pub type PINSTALL_NEW_DEVICE = Option<unsafe extern "system" fn(HWND, LPGUID, PDWORD) -> BOOL>;

unsafe extern "C" fn CreateTitleFont() -> HFONT {
    let mut ncm: NONCLIENTMETRICS = core::mem::zeroed();
    let mut LogFont: LOGFONT = core::mem::zeroed();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut FontSize: INT = 0;
    let mut hFont: HFONT = core::ptr::null_mut();

    ncm.cbSize = core::mem::size_of::<NONCLIENTMETRICS>() as UINT;
    SystemParametersInfo(
        SPI_GETNONCLIENTMETRICS,
        0,
        core::ptr::addr_of_mut!(ncm) as PVOID,
        0,
    );

    LogFont = ncm.lfMessageFont;
    LogFont.lfWeight = (FW_BOLD as i32);
    wcscpy(
        core::ptr::addr_of_mut!(LogFont.lfFaceName) as *mut WCHAR,
        RUSTCONV_WSTR_491507152B61.as_ptr(),
    );

    hdc = GetDC(core::ptr::null_mut());
    FontSize = 12;
    LogFont.lfHeight = (0
        - GetDeviceCaps(hdc, (LOGPIXELSY as i32)) * FontSize / 72) as LONG;
    hFont = CreateFontIndirect(core::ptr::addr_of_mut!(LogFont));
    ReleaseDC(core::ptr::null_mut(), hdc);

    return hFont;
}

unsafe extern "system" fn StartPageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            /* Set title font */
            SendDlgItemMessage(hwndDlg, (IDC_FINISHTITLE as i32), WM_SETFONT, hTitleFont as WPARAM, TRUE as LPARAM);
        }
        WM_NOTIFY => {
            let mut lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT);
                }
                _ => {}
            }
        }
        _ => {}
    }

    return (FALSE) as INT_PTR;
}

unsafe extern "system" fn SearchPageDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            /* TODO: PnP devices search */
        }
        WM_NOTIFY => {
            let mut lpnm: LPNMHDR = lParam as LPNMHDR;
            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT | PSWIZB_BACK);
                }
                _ => {}
            }
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "system" fn IsConnectedPageDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_COMMAND => {
            if HIWORD!(wParam) == (BN_CLICKED as u16) {
                if (SendDlgItemMessage(hwndDlg, (IDC_CONNECTED as i32), BM_GETCHECK, 0, 0) == (BST_CHECKED as LRESULT))
                    || (SendDlgItemMessage(hwndDlg, (IDC_NOTCONNECTED as i32), BM_GETCHECK, 0, 0) == (BST_CHECKED as LRESULT))
                {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT | PSWIZB_BACK);
                } else {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK);
                }
            }
        }
        WM_NOTIFY => {
            let lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    if (SendDlgItemMessage(hwndDlg, (IDC_CONNECTED as i32), BM_GETCHECK, 0, 0) == (BST_CHECKED as LRESULT))
                        || (SendDlgItemMessage(hwndDlg, (IDC_NOTCONNECTED as i32), BM_GETCHECK, 0, 0) == (BST_CHECKED as LRESULT))
                    {
                        PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT | PSWIZB_BACK);
                    } else {
                        PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK);
                    }
                }
                PSN_WIZNEXT => {
                    if SendDlgItemMessage(hwndDlg, (IDC_NOTCONNECTED as i32), BM_GETCHECK, 0, 0) == (BST_CHECKED as LRESULT) {
                        SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_NOTCONNECTEDPAGE as LONG_PTR);
                    } else {
                        SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_PROBELISTPAGE as LONG_PTR);
                    }

                    return (TRUE) as INT_PTR;
                }
                _ => {}
            }
        }
        _ => {}
    }

    return (FALSE) as INT_PTR;
}

unsafe extern "system" fn FinishPageDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            /* Set title font */
            SendDlgItemMessage(hwndDlg, (IDC_FINISHTITLE as i32), WM_SETFONT, hTitleFont as WPARAM, TRUE as LPARAM);
        }
        WM_NOTIFY => {
            let lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    /* Only "Finish" button */
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_FINISH);
                }
                _ => {}
            }
        }
        _ => {}
    }

    return (FALSE) as INT_PTR;
}

unsafe extern "system" fn NotConnectedPageDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            /* Set title font */
            SendDlgItemMessage(hwndDlg, (IDC_FINISHTITLE as i32), WM_SETFONT, hTitleFont as WPARAM, TRUE as LPARAM);
        }
        WM_NOTIFY => {
            let mut lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_FINISH | PSWIZB_BACK);
                }
                PSN_WIZBACK => {
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_ISCONNECTEDPAGE as LONG_PTR);
                    return TRUE as INT_PTR;
                }
                _ => {}
            }
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "C" fn TrimGuidString(szString: LPWSTR, szNewString: LPWSTR) {
    let mut szBuffer: [WCHAR; 39] = [0; 39];
    let mut Index: INT = 0;

    if wcslen(szString) == 38 {
        if (*szString.offset(0 as isize) == 0x007Bu16)
            && (*szString.offset(37 as isize) == 0x007Du16)
        {
            Index = 0;
            while (Index as usize) < wcslen(szString) {
                szBuffer[Index as usize] = *szString.offset((Index + 1) as isize);
                Index += 1;
            }

            szBuffer[36 as usize] = 0x0000u16;
            wcscpy(szNewString, szBuffer.as_mut_ptr());
            return;
        }
    }
    wcscpy(szNewString, RUSTCONV_WSTR_DF3F619804A9.as_ptr());
}

unsafe extern "C" fn InitProbeListPage(hwndDlg: HWND) {
    let mut Column: LVCOLUMNW = core::mem::zeroed();
    let mut Item: LVITEMW = core::mem::zeroed();
    let mut szBuffer: [WCHAR; (MAX_STR_SIZE as usize)] = [0; (MAX_STR_SIZE as usize)];
    let mut szGuid: [WCHAR; (MAX_STR_SIZE as usize)] = [0; (MAX_STR_SIZE as usize)];
    let mut szTrimGuid: [WCHAR; (MAX_STR_SIZE as usize)] = [0; (MAX_STR_SIZE as usize)];
    let mut szStatusText: [WCHAR; (MAX_STR_SIZE as usize)] = [0; (MAX_STR_SIZE as usize)];
    let hList: HWND = GetDlgItem(hwndDlg, (IDC_PROBELIST as i32));
    let mut pstrStatusText: PWSTR = core::ptr::null_mut();
    let mut hDevInfo: HDEVINFO = core::ptr::null_mut();
    let mut DevInfoData: SP_DEVINFO_DATA = core::mem::zeroed();
    let mut ulStatus: ULONG = 0;
    let mut ulProblemNumber: ULONG = 0;
    let mut ClassGuid: GUID = core::mem::zeroed();
    let mut Rect: RECT = core::mem::zeroed();
    let mut Index: DWORD = 0;

    if hList.is_null() {
        return;
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(Column) as *mut core::ffi::c_void,
        core::mem::size_of::<LV_COLUMN>(),
    );

    GetClientRect(hList, core::ptr::addr_of_mut!(Rect));

    Column.mask = LVCF_FMT | LVCF_TEXT | LVCF_WIDTH;
    Column.fmt = (LVCFMT_LEFT as i32);
    Column.iSubItem = 0;
    Column.pszText = core::ptr::null_mut();
    Column.cx = Rect.right - GetSystemMetrics((SM_CXVSCROLL as i32));
    let _ = ListView_InsertColumn!(hList, 0, core::ptr::addr_of_mut!(Column));

    ZeroMemory!(
        core::ptr::addr_of_mut!(Item) as *mut core::ffi::c_void,
        core::mem::size_of::<LV_ITEM>(),
    );

    LoadString(
        hApplet,
        IDS_ADDNEWDEVICE,
        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
        (core::mem::size_of::<[WCHAR; (MAX_STR_SIZE as usize)]>()
            / core::mem::size_of::<WCHAR>()) as i32,
    );

    Item.mask = LVIF_TEXT | LVIF_PARAM | LVIF_STATE | LVIF_IMAGE;
    Item.pszText = core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR;
    Item.iItem = ListView_GetItemCount!(hList) as i32;
    Item.iImage = -1;
    let _ = ListView_InsertItem!(hList, core::ptr::addr_of_mut!(Item));

    hDevInfo = SetupDiGetClassDevsEx(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        DIGCF_ALLCLASSES | DIGCF_PRESENT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if hDevInfo == INVALID_HANDLE_VALUE {
        return;
    }

    /* Get the device image List */
    ImageListData.cbSize = core::mem::size_of::<SP_CLASSIMAGELIST_DATA>() as DWORD;
    SetupDiGetClassImageList(core::ptr::addr_of_mut!(ImageListData));

    DevInfoData.cbSize = (core::mem::size_of::<SP_DEVINFO_DATA>()) as _;
    Index = 0;
    'probe: loop {
        if !((TRUE) != 0) {
            break 'probe;
        }
        'body: {
            szBuffer[0] = 0x0000u16;

            if SetupDiEnumDeviceInfo(hDevInfo, Index, core::ptr::addr_of_mut!(DevInfoData)) == 0 {
                break 'probe;
            }

            if CM_Get_DevNode_Status_Ex(
                core::ptr::addr_of_mut!(ulStatus),
                core::ptr::addr_of_mut!(ulProblemNumber),
                DevInfoData.DevInst,
                0,
                core::ptr::null_mut(),
            ) == CR_SUCCESS
            {
                if (ulStatus & DN_NO_SHOW_IN_DM) != 0 {
                    break 'body;
                }
            }

            /* Get the device's friendly name */
            if SetupDiGetDeviceRegistryProperty(
                hDevInfo,
                core::ptr::addr_of_mut!(DevInfoData),
                SPDRP_FRIENDLYNAME,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(szBuffer) as *mut u8,
                MAX_STR_SIZE,
                core::ptr::null_mut(),
            ) == 0
            {
                /* If the friendly name fails, try the description instead */
                SetupDiGetDeviceRegistryProperty(
                    hDevInfo,
                    core::ptr::addr_of_mut!(DevInfoData),
                    SPDRP_DEVICEDESC,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(szBuffer) as *mut u8,
                    MAX_STR_SIZE,
                    core::ptr::null_mut(),
                );
            }

            SetupDiGetDeviceRegistryProperty(
                hDevInfo,
                core::ptr::addr_of_mut!(DevInfoData),
                SPDRP_CLASSGUID,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(szGuid) as *mut u8,
                MAX_STR_SIZE,
                core::ptr::null_mut(),
            );

            TrimGuidString(
                core::ptr::addr_of_mut!(szGuid) as *mut u16,
                core::ptr::addr_of_mut!(szTrimGuid) as *mut u16,
            );
            UuidFromStringW(
                core::ptr::addr_of_mut!(szTrimGuid) as *mut u16,
                core::ptr::addr_of_mut!(ClassGuid),
            );

            SetupDiGetClassImageIndex(
                core::ptr::addr_of_mut!(ImageListData),
                core::ptr::addr_of_mut!(ClassGuid),
                core::ptr::addr_of_mut!(Item.iImage),
            );

            DeviceProblemTextW(
                core::ptr::null_mut(),
                DevInfoData.DevInst,
                ulProblemNumber,
                core::ptr::addr_of_mut!(szStatusText) as *mut u16,
                (core::mem::size_of::<[WCHAR; (MAX_STR_SIZE as usize)]>()
                    / core::mem::size_of::<WCHAR>()) as u32,
            );

            pstrStatusText = HeapAlloc(
                hProcessHeap,
                0,
                (core::mem::size_of::<[WCHAR; (MAX_STR_SIZE as usize)]>()) as SIZE_T,
            ) as PWSTR;
            lstrcpy(
                pstrStatusText,
                core::ptr::addr_of_mut!(szStatusText) as *mut u16,
            );

            if szBuffer[0] != 0x0000u16 {
                /* Set device name */
                Item.pszText = core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR;
                Item.mask = LVIF_TEXT | LVIF_PARAM | LVIF_STATE | LVIF_IMAGE;
                Item.lParam = pstrStatusText as LPARAM;
                Item.iItem = ListView_GetItemCount!(hList) as i32;
                let _ = ListView_InsertItem!(hList, core::ptr::addr_of_mut!(Item));
            }

            DevInfoData.cbSize = (core::mem::size_of::<SP_DEVINFO_DATA>()) as _;
        }
        Index = Index.wrapping_add(1);
    }

    let _ = ListView_SetImageList!(hList, ImageListData.ImageList, LVSIL_SMALL);
    let _ = ListView_SetExtendedListViewStyle!(hList, LVS_EX_FULLROWSELECT);
    SetupDiDestroyDeviceInfoList(hDevInfo);
}

unsafe extern "system" fn ProbeListPageDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut Index: INT = 0;
    let mut Item: LVITEM = core::mem::zeroed();
    match uMsg {
        WM_INITDIALOG => {
            pDeviceStatusText = HeapAlloc(hProcessHeap, 0 as DWORD, MAX_STR_SIZE as SIZE_T) as PWSTR;
            InitProbeListPage(hwndDlg);
        }
        WM_NOTIFY => {
            let mut lpnm: LPNMHDR = lParam as LPNMHDR;
            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK as UINT | PSWIZB_NEXT as UINT);
                }
                PSN_WIZNEXT => {
                    Index = SendMessage(
                        GetDlgItem(hwndDlg, IDC_PROBELIST as ::core::ffi::c_int),
                        LVM_GETNEXTITEM as UINT,
                        -(1 as ::core::ffi::c_int) as WPARAM,
                        LVNI_FOCUSED as LPARAM,
                    ) as INT;
                    if Index == -(1 as ::core::ffi::c_int) {
                        Index = 0 as INT;
                    }

                    if Index == 0 as INT {
                        SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT as ::core::ffi::c_int, IDD_SELECTWAYPAGE as LONG_PTR);
                    } else {
                        let mut pts: PWSTR = core::ptr::null_mut();

                        ZeroMemory!(core::ptr::addr_of_mut!(Item) as *mut ::core::ffi::c_void, core::mem::size_of::<LV_ITEM>() as size_t);
                        Item.mask = LVIF_PARAM as UINT;
                        Item.iItem = Index as ::core::ffi::c_int;
                        (ListView_GetItem!(GetDlgItem(hwndDlg, IDC_PROBELIST as ::core::ffi::c_int), core::ptr::addr_of_mut!(Item)) as ::core::ffi::c_int);
                        pts = Item.lParam as PWSTR;
                        wcscpy(pDeviceStatusText, pts);

                        SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT as ::core::ffi::c_int, IDD_HWSTATUSPAGE as LONG_PTR);
                    }
                    return TRUE as INT_PTR;
                }
                _ => {}
            }
        }
        WM_DESTROY => {
            Index = ListView_GetItemCount!(GetDlgItem(hwndDlg, IDC_PROBELIST as ::core::ffi::c_int)) as INT;
            loop {
                Index -= 1;
                if !(Index > 0 as INT) {
                    break;
                }
                ZeroMemory!(core::ptr::addr_of_mut!(Item) as *mut ::core::ffi::c_void, core::mem::size_of::<LV_ITEM>() as size_t);
                Item.mask = LVIF_PARAM as UINT;
                Item.iItem = Index as ::core::ffi::c_int;
                (ListView_GetItem!(GetDlgItem(hwndDlg, IDC_PROBELIST as ::core::ffi::c_int), core::ptr::addr_of_mut!(Item)) as ::core::ffi::c_int);
                HeapFree(hProcessHeap, 0 as DWORD, Item.lParam as LPVOID);
            }
            HeapFree(hProcessHeap, 0 as DWORD, pDeviceStatusText as LPVOID);
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "system" fn SelectWayPageDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_NOTIFY => {
            let lpnm: LPNMHDR = lParam as LPNMHDR;
            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_NEXT | PSWIZB_BACK);
                    if SendDlgItemMessage(hwndDlg, (IDC_AUTOINSTALL as i32), BM_GETCHECK, 0, 0) == BST_CHECKED as LRESULT {
                        SendDlgItemMessage(hwndDlg, (IDC_MANUALLYINST as i32), BM_SETCHECK, 0, 0);
                    } else {
                        SendDlgItemMessage(hwndDlg, (IDC_AUTOINSTALL as i32), BM_SETCHECK, 1, 1);
                        SendDlgItemMessage(hwndDlg, (IDC_MANUALLYINST as i32), BM_SETCHECK, 0, 0);
                    }
                }
                PSN_WIZNEXT => {
                    if SendDlgItemMessage(hwndDlg, (IDC_AUTOINSTALL as i32), BM_GETCHECK, 0, 0) == BST_CHECKED as LRESULT {
                        SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_PROGRESSPAGE as LONG_PTR);
                    } else {
                        SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_HWTYPESPAGE as LONG_PTR);
                    }

                    return TRUE as INT_PTR;
                }
                _ => {}
            }
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "system" fn DevStatusPageDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            /* Set title font */
            SendDlgItemMessage(hwndDlg, (IDC_FINISHTITLE as i32), WM_SETFONT, hTitleFont as WPARAM, TRUE as LPARAM);
        }
        WM_NOTIFY => {
            let lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    /* Set status text */
                    SetWindowText(GetDlgItem(hwndDlg, (IDC_HWSTATUSEDIT as i32)), pDeviceStatusText);

                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_FINISH | PSWIZB_BACK);
                }
                PSN_WIZBACK => {
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_PROBELISTPAGE as LONG_PTR);
                    return (TRUE) as INT_PTR;
                }
                _ => {}
            }
        }
        _ => {}
    }

    return (FALSE) as INT_PTR;
}

unsafe extern "C" fn EnumDeviceClasses(
    mut ClassIndex: INT,
    mut DevClassName: LPWSTR,
    mut DevClassDesc: LPWSTR,
    mut DevPresent: *mut BOOL,
    mut ClassImage: *mut INT,
) -> INT {
    let mut ClassGuid: GUID = core::mem::zeroed::<GUID>();
    let mut KeyClass: HKEY = core::ptr::null_mut();
    let mut ClassName: [WCHAR; (MAX_STR_SIZE as usize)] = [0; (MAX_STR_SIZE as usize)];
    let mut RequiredSize: DWORD = MAX_STR_SIZE;
    let mut Ret: UINT = 0;

    *DevPresent = FALSE as BOOL;
    *DevClassName = 0x0000u16;

    Ret = CM_Enumerate_Classes(
        ClassIndex as ULONG,
        core::ptr::addr_of_mut!(ClassGuid),
        0 as ULONG,
    ) as UINT;
    if Ret != CR_SUCCESS as UINT {
        /* All classes enumerated */
        if Ret == CR_NO_SUCH_VALUE as UINT {
            hDevInfoTypes = core::ptr::null_mut() as HDEVINFO;
            return -(1 as INT);
        }

        if Ret == CR_INVALID_DATA as UINT {
            ; /* FIXME: What should we do here? */
        }

        /* Handle other errors... */
    }

    if SetupDiClassNameFromGuid(
        core::ptr::addr_of_mut!(ClassGuid),
        core::ptr::addr_of_mut!(ClassName) as LPWSTR,
        RequiredSize,
        core::ptr::addr_of_mut!(RequiredSize),
    ) != 0
    {
        lstrcpy(DevClassName, core::ptr::addr_of_mut!(ClassName) as LPWSTR);
    }

    if SetupDiGetClassImageIndex(
        core::ptr::addr_of_mut!(ImageListData),
        core::ptr::addr_of_mut!(ClassGuid),
        ClassImage,
    ) == 0
    {
        /* FIXME: Can we do this?
         * Set the blank icon: IDI_SETUPAPI_BLANK = 41
         * It'll be image 24 in the imagelist */
        *ClassImage = 24 as INT;
    }

    /* Get device info for all devices of a particular class */
    hDevInfoTypes = SetupDiGetClassDevs(
        core::ptr::addr_of_mut!(ClassGuid),
        core::ptr::null(),
        core::ptr::null_mut(),
        DIGCF_PRESENT as DWORD,
    );
    if hDevInfoTypes == INVALID_HANDLE_VALUE {
        hDevInfoTypes = core::ptr::null_mut() as HDEVINFO;
        return 0 as INT;
    }

    KeyClass = SetupDiOpenClassRegKeyEx(
        core::ptr::addr_of_mut!(ClassGuid),
        MAXIMUM_ALLOWED as REGSAM,
        DIOCR_INSTALLER as DWORD,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    if KeyClass != INVALID_HANDLE_VALUE as HKEY {

        let mut dwSize: LONG = MAX_STR_SIZE as LONG;

        if RegQueryValue(
            KeyClass,
            core::ptr::null(),
            DevClassDesc,
            core::ptr::addr_of_mut!(dwSize),
        ) != ERROR_SUCCESS as LONG
        {
            *DevClassDesc = 0x0000u16;
        }
    } else {
        return -(3 as INT);
    }

    *DevPresent = TRUE as BOOL;

    RegCloseKey(KeyClass);

    return 0 as INT;
}

unsafe extern "C" fn InitHardWareTypesPage(hwndDlg: HWND) {
    let mut hList: HWND = GetDlgItem(hwndDlg, (IDC_HWTYPESLIST as i32));
    let mut DevName: [WCHAR; (MAX_STR_SIZE as usize)] = [0; (MAX_STR_SIZE as usize)];
    let mut DevDesc: [WCHAR; (MAX_STR_SIZE as usize)] = [0; (MAX_STR_SIZE as usize)];
    let mut DevExist: BOOL = (FALSE) as i32;
    let mut ClassRet: INT = 0;
    let mut DevImage: INT = 0;
    let mut Index: INT = 0;
    let mut Column: LV_COLUMN = core::mem::zeroed();
    let mut Item: LV_ITEM = core::mem::zeroed();
    let mut Rect: RECT = core::mem::zeroed();

    if hList.is_null() {
        return;
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(Column) as *mut core::ffi::c_void,
        core::mem::size_of::<LV_COLUMN>() as usize,
    );

    GetClientRect(hList, core::ptr::addr_of_mut!(Rect));

    Column.mask = LVCF_FMT | LVCF_TEXT | LVCF_WIDTH;
    Column.fmt = (LVCFMT_LEFT as i32);
    Column.iSubItem = 0;
    Column.pszText = core::ptr::null_mut();
    Column.cx = Rect.right - GetSystemMetrics((SM_CXVSCROLL as i32));
    let _ = ListView_InsertColumn!(hList, 0, core::ptr::addr_of_mut!(Column));

    ZeroMemory!(
        core::ptr::addr_of_mut!(Item) as *mut core::ffi::c_void,
        core::mem::size_of::<LV_ITEM>() as usize,
    );

    loop {
        ClassRet = EnumDeviceClasses(
            Index,
            DevName.as_mut_ptr(),
            DevDesc.as_mut_ptr(),
            core::ptr::addr_of_mut!(DevExist),
            core::ptr::addr_of_mut!(DevImage),
        );

        if (ClassRet != -1) && (DevExist != 0) {
            Item.mask = LVIF_TEXT | LVIF_PARAM | LVIF_STATE | LVIF_IMAGE;
            Item.iItem = Index;
            Item.iImage = DevImage;

            if DevDesc[0] != 0x0000u16 {
                Item.pszText = DevDesc.as_mut_ptr();
            } else {
                Item.pszText = DevName.as_mut_ptr();
            }

            let _ = ListView_InsertItem!(hList, core::ptr::addr_of_mut!(Item));

            /* Kill InfoList initialized in EnumDeviceClasses */
            if !hDevInfoTypes.is_null() {
                SetupDiDestroyDeviceInfoList(hDevInfoTypes);
                hDevInfoTypes = core::ptr::null_mut();
            }
        }
        Index += 1;
        if !(ClassRet != -1) {
            break;
        }
    }

    let _ = ListView_SetImageList!(hList, ImageListData.ImageList, LVSIL_SMALL);
}

unsafe extern "system" fn HdTypesPageDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            InitHardWareTypesPage(hwndDlg);
        }
        WM_NOTIFY => {
            let lpnm: LPNMHDR = lParam as LPNMHDR;
            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK);
                }
                PSN_WIZBACK => {
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_SELECTWAYPAGE as LONG_PTR);
                    return (TRUE) as INT_PTR;
                }
                _ => {}
            }
        }
        _ => {}
    }

    return (FALSE) as INT_PTR;
}

unsafe extern "system" fn ProgressPageDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_NOTIFY => {
            let mut lpnm: LPNMHDR = lParam as LPNMHDR;
            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK);
                }
                PSN_WIZBACK => {
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, IDD_SELECTWAYPAGE as LONG_PTR);
                    return (TRUE) as INT_PTR;
                }
                _ => {}
            }
        }
        _ => {}
    }
    return (FALSE) as INT_PTR;
}

unsafe extern "system" fn PropSheetProc(hwndDlg: HWND, uMsg: UINT, lParam: LPARAM) -> i32 {
    let mut hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW!(IDI_CPLICON));
            SendMessageW(hwndDlg, WM_SETICON, (ICON_BIG) as WPARAM, hIcon as LPARAM);
        }
        _ => {}
    }
    return 0;
}

unsafe extern "C" fn HardwareWizardInit(hwnd: HWND) {
    let mut ahpsp: [HPROPSHEETPAGE; 10] = core::mem::zeroed();
    let mut psp: PROPSHEETPAGE = core::mem::zeroed();
    let mut psh: PROPSHEETHEADERW = core::mem::zeroed();
    let mut nPages: UINT = 0 as UINT;

    /* Create the Start page, until setup is working */
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.dwFlags = (PSP_DEFAULT | PSP_HIDEHEADER) as DWORD;
    psp.hInstance = hApplet;
    psp.lParam = 0 as LPARAM;
    psp.pfnDlgProc = Some(StartPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_STARTPAGE);
    let fresh0 = nPages;
    nPages = nPages.wrapping_add(1);
    ahpsp[fresh0 as usize] = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create search page */
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as DWORD;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_SEARCHTITLE);
    psp.pszHeaderSubTitle = core::ptr::null_mut();
    psp.hInstance = hApplet;
    psp.lParam = 0 as LPARAM;
    psp.pfnDlgProc = Some(SearchPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_SEARCHPAGE);
    let fresh1 = nPages;
    nPages = nPages.wrapping_add(1);
    ahpsp[fresh1 as usize] = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create is connected page */
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as DWORD;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_ISCONNECTED);
    psp.pszHeaderSubTitle = core::ptr::null_mut();
    psp.hInstance = hApplet;
    psp.lParam = 0 as LPARAM;
    psp.pfnDlgProc = Some(IsConnectedPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_ISCONNECTEDPAGE);
    let fresh2 = nPages;
    nPages = nPages.wrapping_add(1);
    ahpsp[fresh2 as usize] = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create probe list page */
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as DWORD;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_PROBELISTTITLE);
    psp.pszHeaderSubTitle = core::ptr::null_mut();
    psp.hInstance = hApplet;
    psp.lParam = 0 as LPARAM;
    psp.pfnDlgProc = Some(ProbeListPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_PROBELISTPAGE);
    let fresh3 = nPages;
    nPages = nPages.wrapping_add(1);
    ahpsp[fresh3 as usize] = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create select search way page */
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as DWORD;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_SELECTWAYTITLE);
    psp.pszHeaderSubTitle = core::ptr::null_mut();
    psp.hInstance = hApplet;
    psp.lParam = 0 as LPARAM;
    psp.pfnDlgProc = Some(SelectWayPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_SELECTWAYPAGE);
    let fresh4 = nPages;
    nPages = nPages.wrapping_add(1);
    ahpsp[fresh4 as usize] = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create device status page */
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.dwFlags = (PSP_DEFAULT | PSP_HIDEHEADER) as DWORD;
    psp.hInstance = hApplet;
    psp.lParam = 0 as LPARAM;
    psp.pfnDlgProc = Some(DevStatusPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_HWSTATUSPAGE);
    let fresh5 = nPages;
    nPages = nPages.wrapping_add(1);
    ahpsp[fresh5 as usize] = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create hardware types page */
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as DWORD;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_HDTYPESTITLE);
    psp.pszHeaderSubTitle = core::ptr::null_mut();
    psp.hInstance = hApplet;
    psp.lParam = 0 as LPARAM;
    psp.pfnDlgProc = Some(HdTypesPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_HWTYPESPAGE);
    let fresh6 = nPages;
    nPages = nPages.wrapping_add(1);
    ahpsp[fresh6 as usize] = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create progress page */
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.dwFlags = (PSP_DEFAULT | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as DWORD;
    psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_SEARCHTITLE);
    psp.pszHeaderSubTitle = core::ptr::null_mut();
    psp.hInstance = hApplet;
    psp.lParam = 0 as LPARAM;
    psp.pfnDlgProc = Some(ProgressPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_PROGRESSPAGE);
    let fresh7 = nPages;
    nPages = nPages.wrapping_add(1);
    ahpsp[fresh7 as usize] = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create finish page */
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.dwFlags = (PSP_DEFAULT | PSP_HIDEHEADER) as DWORD;
    psp.hInstance = hApplet;
    psp.lParam = 0 as LPARAM;
    psp.pfnDlgProc = Some(FinishPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_FINISHPAGE);
    let fresh8 = nPages;
    nPages = nPages.wrapping_add(1);
    ahpsp[fresh8 as usize] = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create not connected page */
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    psp.dwFlags = (PSP_DEFAULT | PSP_HIDEHEADER) as DWORD;
    psp.hInstance = hApplet;
    psp.lParam = 0 as LPARAM;
    psp.pfnDlgProc = Some(NotConnectedPageDlgProc);
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_NOTCONNECTEDPAGE);
    let fresh9 = nPages;
    nPages = nPages.wrapping_add(1);
    ahpsp[fresh9 as usize] = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));

    /* Create the property sheet */
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as DWORD;
    psh.dwFlags = (PSH_WIZARD97 | PSH_WATERMARK | PSH_HEADER | PSH_USEICONID | PSH_USECALLBACK) as DWORD;
    psh.hInstance = hApplet;
    psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDI_CPLICON);
    psh.hwndParent = hwnd;
    psh.nPages = nPages;
    psh.__bindgen_anon_2.nStartPage = 0 as UINT;
    psh.__bindgen_anon_3.phpage = ahpsp.as_mut_ptr();
    psh.__bindgen_anon_4.pszbmWatermark = MAKEINTRESOURCE!(IDB_WATERMARK);
    psh.__bindgen_anon_5.pszbmHeader = MAKEINTRESOURCE!(IDB_HEADER);
    psh.pfnCallback = Some(PropSheetProc);

    /* Create title font */
    hTitleFont = CreateTitleFont();

    /* Display the wizard */
    PropertySheet(core::ptr::addr_of_mut!(psh));

    DeleteObject(hTitleFont as HGDIOBJ);
}

#[no_mangle]
pub unsafe extern "system" fn InstallNewDevice(
    hwndParent: HWND,
    ClassGuid: LPGUID,
    pReboot: PDWORD,
) -> BOOL {
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn AddHardwareWizard(hwnd: HWND, lpName: LPWSTR) {
    if !lpName.is_null() {
        if DbgPrint(
            b"(%s:%d) No support of remote installation yet!\n\0".as_ptr() as *const i8,
            b"dll/cpl/hdwwiz/hdwwiz.c\0".as_ptr() as *const i8,
            938i32,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"dll/cpl/hdwwiz/hdwwiz.c\0".as_ptr() as *const i8,
                938i32,
            );
        }
        return;
    }
    HardwareWizardInit(hwnd);
}

#[no_mangle]
pub unsafe extern "system" fn CPlApplet(
    hwndCpl: HWND,
    uMsg: UINT,
    lParam1: LPARAM,
    lParam2: LPARAM,
) -> LONG {
    match uMsg {
        CPL_INIT => {
            return (TRUE) as LONG;
        }
        CPL_GETCOUNT => {
            return 1;
        }
        CPL_INQUIRE => {
            let CPlInfo: *mut CPLINFO = lParam2 as *mut CPLINFO;
            (*CPlInfo).lData = 0;
            (*CPlInfo).idIcon = (IDI_CPLICON as i32);
            (*CPlInfo).idName = (IDS_CPLNAME as i32);
            (*CPlInfo).idInfo = (IDS_CPLDESCRIPTION as i32);
        }
        CPL_DBLCLK => {
            AddHardwareWizard(hwndCpl, core::ptr::null_mut::<WCHAR>());
        }
        _ => {}
    }

    return (FALSE) as LONG;
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, dwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    UNREFERENCED_PARAMETER!(lpvReserved);

    match dwReason {
        DLL_PROCESS_ATTACH => {
            hApplet = hinstDLL;
            hProcessHeap = GetProcessHeap();
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
        }
        _ => {}
    }

    return (TRUE) as BOOL;
}
