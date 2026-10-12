// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_D30E8899E227: [u16; 12] = [
    0x0073u16, 0x0068, 0x0065, 0x006C, 0x006C, 0x0033, 0x0032, 0x002E, 0x0064, 0x006C,
    0x006C, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;
#[no_mangle]
pub unsafe extern "C" fn InitializeListView(mut hDlg: HWND) {
    let mut hListView: HWND = GetDlgItem(hDlg, (IDC_CONNECTEDDRIVELIST as i32));
    let mut column: LVCOLUMNW = core::mem::zeroed::<LVCOLUMNW>();
    let mut szLetter: [WCHAR; 100] = [0; 100];
    let mut szPath: [WCHAR; 100] = [0; 100];

    if hListView.is_null() {
        return;
    }

    LoadStringW(
        hInstance,
        IDS_DRIVE_LETTER as UINT,
        &raw mut szLetter as LPWSTR,
        (core::mem::size_of::<[WCHAR; 100]>() / core::mem::size_of::<WCHAR>())
            as core::ffi::c_int,
    );
    LoadStringW(
        hInstance,
        IDS_NETWORK_PATH as UINT,
        &raw mut szPath as LPWSTR,
        (core::mem::size_of::<[WCHAR; 100]>() / core::mem::size_of::<WCHAR>())
            as core::ffi::c_int,
    );

    column.mask = (LVCF_WIDTH | LVCF_TEXT) as UINT;
    column.pszText = &raw mut szLetter as *mut WCHAR as LPWSTR;
    column.cx = 75 as core::ffi::c_int;
    ListView_InsertColumn!(hListView, 0, &raw mut column);

    column.mask = (LVCF_WIDTH | LVCF_TEXT) as UINT;
    column.pszText = &raw mut szPath as *mut WCHAR as LPWSTR;
    column.cx = 150 as core::ffi::c_int;
    ListView_InsertColumn!(hListView, 1, &raw mut column);
}
#[no_mangle]
pub unsafe extern "C" fn EnumerateConnectedDrives(hDlg: HWND) {
    let mut item: LV_ITEM = core::mem::zeroed();
    let mut hImgList: HIMAGELIST = core::ptr::null_mut();
    let mut hIconDrive: HICON = core::ptr::null_mut();
    let mut hShell32: HMODULE = core::ptr::null_mut();
    let hListView: HWND = GetDlgItem(hDlg, (IDC_CONNECTEDDRIVELIST as i32));

    let mut dRet: DWORD;
    let mut hEnum: HANDLE = core::mem::zeroed();
    let mut lpRes: LPNETRESOURCE;
    let mut dwSize: DWORD = 0x1000;
    let mut dwCount: DWORD = (1 as u32).wrapping_neg() as DWORD;
    let mut lpCur: LPNETRESOURCE;

    /* List View Icons */
    hShell32 = GetModuleHandleW(RUSTCONV_WSTR_D30E8899E227.as_ptr());
    if hShell32.is_null() {
        return;
    }

    hIconDrive = LoadImageW(
        hShell32,
        MAKEINTRESOURCEW!(10),
        IMAGE_ICON,
        GetSystemMetrics((SM_CXSMICON as i32)),
        GetSystemMetrics((SM_CYSMICON as i32)),
        LR_DEFAULTCOLOR,
    ) as HICON;
    if hIconDrive.is_null() {
        return;
    }

    hImgList = ImageList_Create(
        16,
        16,
        ILC_COLOR32 | ILC_MASK,
        1,
        1,
    );
    if hImgList.is_null() {
        DestroyIcon(hIconDrive);
        return;
    }

    ImageList_AddIcon!(hImgList, hIconDrive);
    DestroyIcon(hIconDrive);
    ListView_SetImageList!(hListView, hImgList, LVSIL_SMALL);

    dRet = WNetOpenEnum(
        RESOURCE_CONNECTED,
        RESOURCETYPE_DISK,
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hEnum),
    );
    if dRet != WN_SUCCESS {
        return;
    }

    lpRes = HeapAlloc(GetProcessHeap(), 0, dwSize as SIZE_T) as LPNETRESOURCE;
    if lpRes.is_null() {
        WNetCloseEnum(hEnum);
        return;
    }

    loop {
        ZeroMemory!(lpRes as *mut core::ffi::c_void, dwSize as usize);
        dRet = WNetEnumResource(
            hEnum,
            core::ptr::addr_of_mut!(dwCount),
            lpRes as LPVOID,
            core::ptr::addr_of_mut!(dwSize),
        );
        if dRet == WN_SUCCESS || dRet == WN_MORE_DATA {
            lpCur = lpRes;
            while dwCount != 0 {
                ZeroMemory!(
                    core::ptr::addr_of_mut!(item) as *mut core::ffi::c_void,
                    core::mem::size_of::<LV_ITEM>(),
                );
                item.mask = LVIF_TEXT | LVIF_IMAGE;
                item.iImage = 0;
                item.pszText = (*lpCur).lpLocalName;
                item.lParam = 0;
                item.iItem = ListView_InsertItem!(hListView, core::ptr::addr_of_mut!(item));
                ListView_SetItemText!(
                    hListView,
                    item.iItem,
                    1,
                    (*lpCur).lpRemoteName,
                );
                lpCur = lpCur.add(1);
                dwCount = dwCount.wrapping_sub(1);
            }
        }
        if !(dRet != WN_NO_MORE_ENTRIES) {
            break;
        }
    }

    HeapFree(GetProcessHeap(), 0, lpRes as PVOID);
    WNetCloseEnum(hEnum);
}
#[no_mangle]
pub unsafe extern "C" fn UpdateButtonStatus(wParam: WPARAM, lParam: LPARAM, hDlg: HWND) {
    let pnmh: LPNMHDR = lParam as LPNMHDR;
    let hListView: HWND = GetDlgItem(hDlg, (IDC_CONNECTEDDRIVELIST as i32));
    let hOkButton: HWND = GetDlgItem(hDlg, (ID_OK as i32));

    if (*pnmh).hwndFrom == hListView {
        match ((*pnmh).code) as i32 {
            LVN_ITEMCHANGED => {
                if ListView_GetSelectedCount!(hListView) != 0 {
                    EnableWindow(hOkButton, (TRUE) as BOOL);
                } else {
                    EnableWindow(hOkButton, (FALSE) as BOOL);
                }
            }
            _ => {}
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn DisconnectDriveExit(mut hDlg: HWND) -> DWORD {
    let mut driveLetter: [TCHAR; 10] = [0; 10];
    let mut hr: HRESULT;
    let mut hListView: HWND = GetDlgItem(hDlg, (IDC_CONNECTEDDRIVELIST as i32));
    let mut nItem: INT = ListView_GetNextItem!(hListView, -(1), LVNI_SELECTED);

    if nItem == -(1) {
        return 0 as DWORD;
    }

    ListView_GetItemText!(
        hListView,
        nItem,
        0,
        driveLetter.as_mut_ptr(),
        _countof!(driveLetter),
    );
    hr = WNetCancelConnection2(
        driveLetter.as_mut_ptr(),
        CONNECT_UPDATE_PROFILE,
        (FALSE as i32),
    ) as HRESULT;

    EndDialog(hDlg, ID_OK as INT_PTR);
    return hr as DWORD;
}

unsafe extern "system" fn DisconnectDlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut hIcon: HICON = core::ptr::null_mut();
    let mut hIconSm: HICON = core::ptr::null_mut();
    match uMsg {
        WM_INITDIALOG => {
            hIcon = LoadImageW(
                hInstance,
                MAKEINTRESOURCEW!(IDI_DISCONNECT_NET_DRIVES) as LPCWSTR,
                IMAGE_ICON as UINT,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (LR_SHARED | LR_DEFAULTSIZE) as UINT,
            ) as HICON;
            hIconSm = CopyImage(
                hIcon as HANDLE,
                IMAGE_ICON as UINT,
                GetSystemMetrics((SM_CXSMICON as i32)),
                GetSystemMetrics((SM_CYSMICON as i32)),
                LR_COPYFROMRESOURCE as UINT,
            ) as HICON;
            SendMessageW(
                hDlg,
                WM_SETICON as UINT,
                ICON_BIG as WPARAM,
                hIcon as LPARAM,
            );
            SendMessageW(
                hDlg,
                WM_SETICON as UINT,
                ICON_SMALL as WPARAM,
                hIconSm as LPARAM,
            );
            EnableWindow(
                GetDlgItem(hDlg, (ID_OK as i32)),
                (FALSE) as BOOL,
            );
            InitializeListView(hDlg);
            EnumerateConnectedDrives(hDlg);
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match (LOWORD!(wParam) as ::core::ffi::c_int) as u32 {
                ID_OK => {
                    DisconnectDriveExit(hDlg);
                }
                IDCANCEL => {
                    EndDialog(hDlg, IDCANCEL as INT_PTR);
                }
                _ => {}
            }
        }
        WM_NOTIFY => {
            UpdateButtonStatus(wParam, lParam, hDlg);
        }
        _ => {
            return FALSE as INT_PTR;
        }
    }

    return TRUE as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn SHDisconnectNetDrives(Unused: PVOID) -> HRESULT {
    DialogBoxW!(
        hInstance,
        MAKEINTRESOURCEW!(IDD_DISCONNECTDRIVES),
        core::ptr::null_mut(),
        Some(DisconnectDlgProc),
    );
    return S_OK;
}
