use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut ColumnDataHints: [UINT; 26] = [0; 26];

#[no_mangle]
pub static mut ColumnPresets: [PresetColumnEntry; 26] = [
    PresetColumnEntry {
        dwIdsName: IDS_TAB_IMAGENAME as DWORD,
        dwIdcCtrl: IDC_IMAGENAME as DWORD,
        size: 105 as ::core::ffi::c_int,
        bDefaults: 1 as BOOL,
        dwAlign: 0 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_PID as DWORD,
        dwIdcCtrl: IDC_PID as DWORD,
        size: 50 as ::core::ffi::c_int,
        bDefaults: 1 as BOOL,
        dwAlign: 0 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_USERNAME as DWORD,
        dwIdcCtrl: IDC_USERNAME as DWORD,
        size: 107 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_SESSIONID as DWORD,
        dwIdcCtrl: IDC_SESSIONID as DWORD,
        size: 70 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_CPUUSAGE as DWORD,
        dwIdcCtrl: IDC_CPUUSAGE as DWORD,
        size: 35 as ::core::ffi::c_int,
        bDefaults: 1 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_CPUTIME as DWORD,
        dwIdcCtrl: IDC_CPUTIME as DWORD,
        size: 70 as ::core::ffi::c_int,
        bDefaults: 1 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_MEMORYUSAGE as DWORD,
        dwIdcCtrl: IDC_MEMORYUSAGE as DWORD,
        size: 70 as ::core::ffi::c_int,
        bDefaults: 1 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_PEAKMEMORYUSAGE as DWORD,
        dwIdcCtrl: IDC_PEAKMEMORYUSAGE as DWORD,
        size: 100 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_MEMORYUSAGEDELTA as DWORD,
        dwIdcCtrl: IDC_MEMORYUSAGEDELTA as DWORD,
        size: 70 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_PAGEFAULTS as DWORD,
        dwIdcCtrl: IDC_PAGEFAULTS as DWORD,
        size: 70 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_PAGEFAULTSDELTA as DWORD,
        dwIdcCtrl: IDC_PAGEFAULTSDELTA as DWORD,
        size: 70 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_VIRTUALMEMORYSIZE as DWORD,
        dwIdcCtrl: IDC_VIRTUALMEMORYSIZE as DWORD,
        size: 70 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_PAGEDPOOL as DWORD,
        dwIdcCtrl: IDC_PAGEDPOOL as DWORD,
        size: 70 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_NONPAGEDPOOL as DWORD,
        dwIdcCtrl: IDC_NONPAGEDPOOL as DWORD,
        size: 70 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_BASEPRIORITY as DWORD,
        dwIdcCtrl: IDC_BASEPRIORITY as DWORD,
        size: 60 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_HANDLECOUNT as DWORD,
        dwIdcCtrl: IDC_HANDLECOUNT as DWORD,
        size: 60 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_THREADCOUNT as DWORD,
        dwIdcCtrl: IDC_THREADCOUNT as DWORD,
        size: 60 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_USEROBJECTS as DWORD,
        dwIdcCtrl: IDC_USEROBJECTS as DWORD,
        size: 60 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_GDIOBJECTS as DWORD,
        dwIdcCtrl: IDC_GDIOBJECTS as DWORD,
        size: 60 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_IOREADS as DWORD,
        dwIdcCtrl: IDC_IOREADS as DWORD,
        size: 70 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_IOWRITES as DWORD,
        dwIdcCtrl: IDC_IOWRITES as DWORD,
        size: 70 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_IOOTHER as DWORD,
        dwIdcCtrl: IDC_IOOTHER as DWORD,
        size: 70 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_IOREADBYTES as DWORD,
        dwIdcCtrl: IDC_IOREADBYTES as DWORD,
        size: 70 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_IOWRITEBYTES as DWORD,
        dwIdcCtrl: IDC_IOWRITEBYTES as DWORD,
        size: 70 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_IOOTHERBYTES as DWORD,
        dwIdcCtrl: IDC_IOOTHERBYTES as DWORD,
        size: 70 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0x1 as DWORD,
    },
    PresetColumnEntry {
        dwIdsName: IDS_TAB_COMMANDLINE as DWORD,
        dwIdcCtrl: IDC_COMMANDLINE as DWORD,
        size: 450 as ::core::ffi::c_int,
        bDefaults: 0 as BOOL,
        dwAlign: 0 as DWORD,
    },
];

// Field names and types for PresetColumnEntry are inferred from the initializer order;
// the struct itself is not in this preamble, so it is defined here for the table below.
pub type PPresetColumnEntry = *mut PresetColumnEntry;

// LVCFMT_LEFT = 0, LVCFMT_RIGHT = 1 (Win32 values)

#[no_mangle]
pub unsafe extern "C" fn AddColumns() {
    let mut size: LRESULT = 0;
    let mut szTemp: [WCHAR; 256] = [0; 256];
    let mut n: ::core::ffi::c_uint = 0;
    n = 0 as ::core::ffi::c_uint;
    while n < COLUMN_NMAX as ::core::ffi::c_uint {
        if TaskManagerSettings.Columns[n as usize] != 0 {
            LoadStringW(
                hInst,
                ColumnPresets[n as usize].dwIdsName as UINT,
                szTemp.as_mut_ptr() as LPWSTR,
                _countof!(szTemp) as ::core::ffi::c_int,
            );
            InsertColumn(
                n as ::core::ffi::c_int,
                szTemp.as_mut_ptr() as LPCWSTR,
                ColumnPresets[n as usize].dwAlign as ::core::ffi::c_int,
                TaskManagerSettings.ColumnSizeArray[n as usize],
                -1 as ::core::ffi::c_int,
            );
        }
        n = n.wrapping_add(1);
    }
    size = SendMessageW(
        hProcessPageHeaderCtrl,
        HDM_GETITEMCOUNT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
    SendMessageW(
        hProcessPageHeaderCtrl,
        HDM_SETORDERARRAY as UINT,
        size as WPARAM,
        &raw mut TaskManagerSettings.ColumnOrderArray as LPARAM,
    );
    UpdateColumnDataHints();
}

unsafe extern "C" fn InsertColumn(nCol: i32, lpszColumnHeading: LPCWSTR, nFormat: i32, nWidth: i32, nSubItem: i32) -> i32 {
    let mut column: LVCOLUMN = core::mem::zeroed();

    column.mask = (LVCF_TEXT | LVCF_FMT) as UINT;
    column.pszText = lpszColumnHeading as LPWSTR;
    column.fmt = nFormat;

    if nWidth != -(1 as i32) {
        column.mask |= LVCF_WIDTH as UINT;
        column.cx = nWidth;
    }

    if nSubItem != -(1 as i32) {
        column.mask |= LVCF_SUBITEM as UINT;
        column.iSubItem = nSubItem;
    }

    return ListView_InsertColumn!(hProcessPageListCtrl, nCol, core::ptr::addr_of_mut!(column));
}
#[no_mangle]
pub unsafe extern "C" fn SaveColumnSettings() {
    let mut hditem: HDITEMW = core::mem::zeroed();
    let mut i: ::core::ffi::c_int = 0;
    let mut n: ::core::ffi::c_int = 0;
    let mut text: [WCHAR; 260] = [0; 260];
    let mut szTemp: [WCHAR; 256] = [0; 256];
    let mut size: LRESULT = 0;

    /* Reset column data */
    i = 0 as ::core::ffi::c_int;
    while i < COLUMN_NMAX as ::core::ffi::c_int {
        TaskManagerSettings.ColumnOrderArray[i as usize] = i;
        TaskManagerSettings.Columns[i as usize] = FALSE as BOOL;
        TaskManagerSettings.ColumnSizeArray[i as usize] = ColumnPresets[i as usize].size;
        i += 1;
    }

    /* Get header order */
    size = SendMessageW(
        hProcessPageHeaderCtrl,
        HDM_GETITEMCOUNT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
    SendMessageW(
        hProcessPageHeaderCtrl,
        HDM_GETORDERARRAY as UINT,
        size as WPARAM,
        core::ptr::addr_of_mut!(TaskManagerSettings.ColumnOrderArray) as LPARAM,
    );

    /* Get visible columns */
    i = 0 as ::core::ffi::c_int;
    while (i as LRESULT)
        < SendMessageW(
            hProcessPageHeaderCtrl,
            HDM_GETITEMCOUNT as UINT,
            0 as WPARAM,
            0 as LPARAM,
        )
    {
        memset(
            core::ptr::addr_of_mut!(hditem) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<HDITEM>() as size_t,
        );

        hditem.mask = (HDI_TEXT | HDI_WIDTH) as UINT;
        hditem.pszText = core::ptr::addr_of_mut!(text) as *mut WCHAR as LPWSTR;
        hditem.cchTextMax = 260 as ::core::ffi::c_int;

        SendMessageW(
            hProcessPageHeaderCtrl,
            HDM_GETITEM as UINT,
            i as WPARAM,
            core::ptr::addr_of_mut!(hditem) as LPARAM,
        );

        n = 0 as ::core::ffi::c_int;
        while n < COLUMN_NMAX as ::core::ffi::c_int {
            LoadStringW(
                hInst,
                ColumnPresets[n as usize].dwIdsName as UINT,
                core::ptr::addr_of_mut!(szTemp) as LPWSTR,
                _countof!(szTemp) as ::core::ffi::c_int,
            );
            if _wcsicmp(
                core::ptr::addr_of_mut!(text) as *mut WCHAR,
                core::ptr::addr_of_mut!(szTemp) as *mut WCHAR,
            ) == 0 as ::core::ffi::c_int
            {
                TaskManagerSettings.Columns[n as usize] = TRUE as BOOL;
                TaskManagerSettings.ColumnSizeArray[n as usize] = hditem.cxy;
            }
            n += 1;
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ProcessPage_OnViewSelectColumns() {
    let mut i: ::core::ffi::c_int;

    if (DialogBoxW!(hInst, MAKEINTRESOURCEW!(IDD_COLUMNS_DIALOG), hMainWnd, Some(ColumnsDialogWndProc))) as u32 == (IDOK) as _ {
        i = Header_GetItemCount!(hProcessPageHeaderCtrl) - 1;
        while i >= 0 {
            let _ = ListView_DeleteColumn!(hProcessPageListCtrl, i);
            i -= 1;
        }

        i = 0;
        while i < (COLUMN_NMAX as i32) {
            TaskManagerSettings.ColumnOrderArray[i as usize] = i;
            TaskManagerSettings.ColumnSizeArray[i as usize] = ColumnPresets[i as usize].size;
            i += 1;
        }

        AddColumns();
    }
}
#[no_mangle]
pub unsafe extern "system" fn ColumnsDialogWndProc(
    hDlg: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut i: ::core::ffi::c_uint = 0;
    match message {
        WM_INITDIALOG => {
            i = 0 as ::core::ffi::c_uint;
            while i < COLUMN_NMAX as ::core::ffi::c_uint {
                if TaskManagerSettings.Columns[i as usize] != 0 {
                    CheckDlgButton(
                        hDlg,
                        ColumnPresets[i as usize].dwIdcCtrl as ::core::ffi::c_int,
                        BST_CHECKED as UINT,
                    );
                }
                i = i.wrapping_add(1);
            }
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            if (LOWORD!(wParam) as ::core::ffi::c_int) as u32 == IDCANCEL {
                EndDialog(hDlg, LOWORD!(wParam) as INT_PTR);
                return TRUE as INT_PTR;
            }

            if (LOWORD!(wParam) as ::core::ffi::c_int) as u32 == IDOK {
                i = 0 as ::core::ffi::c_uint;
                while i < COLUMN_NMAX as ::core::ffi::c_uint {
                    TaskManagerSettings.Columns[i as usize] = IsDlgButtonChecked(
                        hDlg,
                        ColumnPresets[i as usize].dwIdcCtrl as ::core::ffi::c_int,
                    ) as BOOL;
                    i = i.wrapping_add(1);
                }

                EndDialog(hDlg, LOWORD!(wParam) as INT_PTR);
                return TRUE as INT_PTR;
            }
        }
        _ => {}
    }

    return 0 as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn UpdateColumnDataHints() {
    let mut hditem: HDITEMW = core::mem::zeroed();
    let mut text: [WCHAR; 260] = [0; 260];
    let mut Index: ULONG = 0;
    let mut uItems: ULONG = 0;
    let mut szTemp: [WCHAR; 256] = [0; 256];
    let mut i: UINT = 0;

    uItems = min!(
        SendMessageW(
            hProcessPageHeaderCtrl,
            HDM_GETITEMCOUNT as UINT,
            0 as WPARAM,
            0 as LPARAM,
        ) as ULONG,
        COLUMN_NMAX as ULONG,
    );

    Index = 0 as ULONG;
    while Index < uItems {
        memset(
            &raw mut hditem as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            core::mem::size_of::<HDITEM>() as size_t,
        );

        hditem.mask = HDI_TEXT as UINT;
        hditem.pszText = &raw mut text as *mut WCHAR as LPWSTR;
        hditem.cchTextMax = 260 as ::core::ffi::c_int;

        SendMessageW(
            hProcessPageHeaderCtrl,
            HDM_GETITEM as UINT,
            Index as WPARAM,
            &raw mut hditem as LPARAM,
        );

        i = 0 as UINT;
        while i < COLUMN_NMAX as UINT {
            LoadStringW(
                hInst,
                ColumnPresets[i as usize].dwIdsName as UINT,
                &raw mut szTemp as LPWSTR,
                _countof!(szTemp) as ::core::ffi::c_int,
            );
            if _wcsicmp(
                &raw mut text as *mut WCHAR,
                &raw mut szTemp as *mut WCHAR,
            ) == 0 as ::core::ffi::c_int
            {
                ColumnDataHints[Index as usize] = i;
            }
            i = i.wrapping_add(1);
        }
        Index = Index.wrapping_add(1);
    }
}
