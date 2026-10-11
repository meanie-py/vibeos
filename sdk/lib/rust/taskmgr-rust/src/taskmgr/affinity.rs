use crate::taskmgr::taskmgr::GetLastErrorText;
use crate::taskmgr::procpage::GetSelectedProcessId;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hProcessAffinityHandle: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut dwCpuTable: [DWORD; 32] = [
    IDC_CPU0 as DWORD,
    IDC_CPU1 as DWORD,
    IDC_CPU2 as DWORD,
    IDC_CPU3 as DWORD,
    IDC_CPU4 as DWORD,
    IDC_CPU5 as DWORD,
    IDC_CPU6 as DWORD,
    IDC_CPU7 as DWORD,
    IDC_CPU8 as DWORD,
    IDC_CPU9 as DWORD,
    IDC_CPU10 as DWORD,
    IDC_CPU11 as DWORD,
    IDC_CPU12 as DWORD,
    IDC_CPU13 as DWORD,
    IDC_CPU14 as DWORD,
    IDC_CPU15 as DWORD,
    IDC_CPU16 as DWORD,
    IDC_CPU17 as DWORD,
    IDC_CPU18 as DWORD,
    IDC_CPU19 as DWORD,
    IDC_CPU20 as DWORD,
    IDC_CPU21 as DWORD,
    IDC_CPU22 as DWORD,
    IDC_CPU23 as DWORD,
    IDC_CPU24 as DWORD,
    IDC_CPU25 as DWORD,
    IDC_CPU26 as DWORD,
    IDC_CPU27 as DWORD,
    IDC_CPU28 as DWORD,
    IDC_CPU29 as DWORD,
    IDC_CPU30 as DWORD,
    IDC_CPU31 as DWORD,
];

#[no_mangle]
pub unsafe extern "C" fn ProcessPage_OnSetAffinity() {
    let mut dwProcessId: DWORD;
    let mut strErrorText: [WCHAR; 260] = [0; 260];
    let mut szTitle: [WCHAR; 256] = [0; 256];

    dwProcessId = GetSelectedProcessId();

    if dwProcessId == 0 {
        return;
    }

    hProcessAffinityHandle = OpenProcess(
        (PROCESS_QUERY_INFORMATION | PROCESS_SET_INFORMATION) as DWORD,
        (FALSE) as BOOL,
        dwProcessId,
    );
    if hProcessAffinityHandle.is_null() {
        GetLastErrorText(
            core::ptr::addr_of_mut!(strErrorText) as *mut WCHAR,
            _countof!(strErrorText) as DWORD,
        );
        LoadStringW(
            hInst,
            IDS_MSG_ACCESSPROCESSAFF as UINT,
            core::ptr::addr_of_mut!(szTitle) as *mut WCHAR,
            _countof!(szTitle) as core::ffi::c_int,
        );
        MessageBoxW(
            hMainWnd,
            core::ptr::addr_of_mut!(strErrorText) as *mut WCHAR,
            core::ptr::addr_of_mut!(szTitle) as *mut WCHAR,
            (MB_OK | MB_ICONSTOP) as UINT,
        );
        return;
    }
    DialogBoxW!(
        hInst,
        MAKEINTRESOURCEW!(IDD_AFFINITY_DIALOG),
        hMainWnd,
        Some(AffinityDialogWndProc),
    );
    if !hProcessAffinityHandle.is_null() {
        CloseHandle(hProcessAffinityHandle);
        hProcessAffinityHandle = core::ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "system" fn AffinityDialogWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut dwProcessAffinityMask: DWORD_PTR = 0;
    let mut dwSystemAffinityMask: DWORD_PTR = 0;
    let mut strErrorText: [WCHAR; 260] = [0; 260];
    let mut szTitle: [WCHAR; 256] = [0; 256];
    let mut nCpu: BYTE;

    match message {
        WM_INITDIALOG => {
            /*
             * Get the current affinity mask for the process and
             * the number of CPUs present in the system
             */
            if GetProcessAffinityMask(hProcessAffinityHandle, core::ptr::addr_of_mut!(dwProcessAffinityMask), core::ptr::addr_of_mut!(dwSystemAffinityMask)) == 0 {
                GetLastErrorText(core::ptr::addr_of_mut!(strErrorText) as LPTSTR, _countof!(strErrorText) as DWORD);
                EndDialog(hDlg, 0 as INT_PTR);
                LoadStringW(hInst, IDS_MSG_ACCESSPROCESSAFF as UINT, core::ptr::addr_of_mut!(szTitle) as LPWSTR, _countof!(szTitle) as ::core::ffi::c_int);
                MessageBoxW(hMainWnd, core::ptr::addr_of_mut!(strErrorText) as *mut WCHAR as LPCWSTR, core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR, (MB_OK | MB_ICONSTOP) as UINT);
            }

            nCpu = 0 as BYTE;
            while (nCpu as usize) < _countof!(dwCpuTable) as usize {
                /*
                 * Enable a checkbox for each processor present in the system
                 */
                if dwSystemAffinityMask & ((1 as ULONG_PTR) << nCpu) != 0 {
                    EnableWindow(GetDlgItem(hDlg, dwCpuTable[nCpu as usize] as ::core::ffi::c_int), TRUE as BOOL);
                }
                /*
                 * Check each checkbox that the current process
                 * has affinity with
                 */
                if dwProcessAffinityMask & ((1 as ULONG_PTR) << nCpu) != 0 {
                    CheckDlgButton(hDlg, dwCpuTable[nCpu as usize] as ::core::ffi::c_int, BST_CHECKED as UINT);
                }
                nCpu = nCpu.wrapping_add(1);
            }

            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            /*
             * If the user has cancelled the dialog box
             * then just close it
             */
            if (LOWORD!(wParam)) as u32 == IDCANCEL {
                EndDialog(hDlg, LOWORD!(wParam) as INT_PTR);
                return TRUE as INT_PTR;
            }

            /*
             * The user has clicked OK -- so now we have
             * to adjust the process affinity mask
             */
            if (LOWORD!(wParam)) as u32 == IDOK {
                nCpu = 0 as BYTE;
                while (nCpu as usize) < _countof!(dwCpuTable) as usize {
                    /*
                     * First we have to create a mask out of each
                     * checkbox that the user checked.
                     */
                    if IsDlgButtonChecked(hDlg, dwCpuTable[nCpu as usize] as ::core::ffi::c_int) != 0 {
                        dwProcessAffinityMask |= (1 as ULONG_PTR) << nCpu;
                    }
                    nCpu = nCpu.wrapping_add(1);
                }

                /*
                 * Make sure they are giving the process affinity
                 * with at least one processor. I'd hate to see a
                 * process that is not in a wait state get deprived
                 * of it's cpu time.
                 */
                if dwProcessAffinityMask == 0 {
                    LoadStringW(hInst, IDS_MSG_PROCESSONEPRO as UINT, core::ptr::addr_of_mut!(strErrorText) as LPWSTR, _countof!(strErrorText) as ::core::ffi::c_int);
                    LoadStringW(hInst, IDS_MSG_INVALIDOPTION as UINT, core::ptr::addr_of_mut!(szTitle) as LPWSTR, _countof!(szTitle) as ::core::ffi::c_int);
                    MessageBoxW(hDlg, core::ptr::addr_of_mut!(strErrorText) as *mut WCHAR as LPCWSTR, core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR, (MB_OK | MB_ICONSTOP) as UINT);
                    return TRUE as INT_PTR;
                }

                /*
                 * Try to set the process affinity
                 */
                if SetProcessAffinityMask(hProcessAffinityHandle, dwProcessAffinityMask) == 0 {
                    GetLastErrorText(core::ptr::addr_of_mut!(strErrorText) as LPTSTR, _countof!(strErrorText) as DWORD);
                    EndDialog(hDlg, LOWORD!(wParam) as INT_PTR);
                    LoadStringW(hInst, IDS_MSG_ACCESSPROCESSAFF as UINT, core::ptr::addr_of_mut!(szTitle) as LPWSTR, _countof!(szTitle) as ::core::ffi::c_int);
                    MessageBoxW(hMainWnd, core::ptr::addr_of_mut!(strErrorText) as *mut WCHAR as LPCWSTR, core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR, (MB_OK | MB_ICONSTOP) as UINT);
                }

                EndDialog(hDlg, LOWORD!(wParam) as INT_PTR);
                return TRUE as INT_PTR;
            }
        }

        _ => {}
    }

    return 0 as INT_PTR;
}
