use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn Hib_InitDialog(hwndDlg: HWND) {
    let mut PowerCaps: SYSTEM_POWER_CAPABILITIES = core::mem::zeroed();
    let mut msex: MEMORYSTATUSEX = core::mem::zeroed();
    let mut szTemp: [TCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut lpRoot: LPTSTR = core::ptr::null_mut();
    let mut FreeBytesAvailable: ULARGE_INTEGER = core::mem::zeroed();
    let mut TotalNumberOfBytes: ULARGE_INTEGER = core::mem::zeroed();
    let mut TotalNumberOfFreeBytes: ULARGE_INTEGER = core::mem::zeroed();
    let mut bHibernate: BOOLEAN;

    if GetPwrCapabilities(core::ptr::addr_of_mut!(PowerCaps)) != 0 {
        CheckDlgButton(
            hwndDlg,
            (IDC_HIBERNATEFILE as i32),
            if PowerCaps.HiberFilePresent != 0 {
                BST_CHECKED
            } else {
                BST_UNCHECKED
            },
        );

        msex.dwLength = core::mem::size_of::<MEMORYSTATUSEX>() as DWORD;
        if GlobalMemoryStatusEx(core::ptr::addr_of_mut!(msex)) == 0 {
            return; // FIXME
        }

        if GetWindowsDirectory((szTemp.as_mut_ptr()).cast(), _countof!(szTemp)) != 0 {
            lpRoot = szTemp.as_mut_ptr();
        } else {
            lpRoot = core::ptr::null_mut();
        }

        // Get available space and size of selected volume.
        if GetDiskFreeSpaceEx(
            lpRoot.cast(),
            core::ptr::addr_of_mut!(FreeBytesAvailable),
            core::ptr::addr_of_mut!(TotalNumberOfBytes),
            core::ptr::addr_of_mut!(TotalNumberOfFreeBytes),
        ) == 0
        {
            TotalNumberOfFreeBytes.QuadPart = 0;
        }

        // Print the free available space into selected volume.
        StrFormatByteSize((TotalNumberOfFreeBytes.QuadPart) as i64, (szTemp.as_mut_ptr()).cast(), _countof!(szTemp));
        SetDlgItemText(hwndDlg, (IDC_FREESPACE as i32), szTemp.as_mut_ptr().cast());

        // Print the amount of space required for hibernation.
        StrFormatByteSize((msex.ullTotalPhys) as i64, (szTemp.as_mut_ptr()).cast(), _countof!(szTemp));
        SetDlgItemText(hwndDlg, (IDC_SPACEFORHIBERNATEFILE as i32), szTemp.as_mut_ptr().cast());

        if TotalNumberOfFreeBytes.QuadPart < msex.ullTotalPhys && PowerCaps.HiberFilePresent == 0 {
            EnableWindow(
                GetDlgItem(hwndDlg, (IDC_HIBERNATEFILE as i32)),
                (FALSE) as BOOL,
            );
            ShowWindow(
                GetDlgItem(hwndDlg, (IDC_TOLESSFREESPACE as i32)),
                (SW_SHOW) as i32,
            );
        } else {
            ShowWindow(
                GetDlgItem(hwndDlg, (IDC_TOLESSFREESPACE as i32)),
                (SW_HIDE) as i32,
            );
            EnableWindow(
                GetDlgItem(hwndDlg, (IDC_HIBERNATEFILE as i32)),
                (TRUE) as BOOL,
            );
        }

        bHibernate = (PowerCaps.HiberFilePresent != 0) as BOOLEAN;
        if CallNtPowerInformation(
            SystemReserveHiberFile,
            core::ptr::addr_of_mut!(bHibernate) as PVOID,
            core::mem::size_of::<BOOLEAN>() as ULONG,
            core::ptr::null_mut(),
            0,
        ) != STATUS_SUCCESS
        {
            EnableWindow(
                GetDlgItem(hwndDlg, (IDC_HIBERNATEFILE as i32)),
                (FALSE) as BOOL,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Hib_SaveData(mut hwndDlg: HWND) -> INT_PTR {
    let mut bHibernate: BOOLEAN = 0;
    bHibernate = (IsDlgButtonChecked(hwndDlg, (IDC_HIBERNATEFILE as i32)) == BST_CHECKED as UINT)
        as ::core::ffi::c_int as BOOLEAN;
    if CallNtPowerInformation(
        SystemReserveHiberFile,
        core::ptr::addr_of_mut!(bHibernate) as PVOID,
        core::mem::size_of::<BOOLEAN>() as ULONG,
        core::ptr::null_mut(),
        0 as ULONG,
    ) == STATUS_SUCCESS
    {
        Hib_InitDialog(hwndDlg);
        return TRUE as INT_PTR;
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn HibernateDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match (uMsg as u32) {
        WM_INITDIALOG => {
            Hib_InitDialog(hwndDlg);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam) as u32) {
                IDC_HIBERNATEFILE => {
                    if (HIWORD!(wParam) as u32) == (BN_CLICKED as u32) {
                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }
                _ => {}
            }
        }
        WM_NOTIFY => {
            if (*(lParam as LPNMHDR)).code == (PSN_APPLY as UINT) {
                return Hib_SaveData(hwndDlg);
            }
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}
