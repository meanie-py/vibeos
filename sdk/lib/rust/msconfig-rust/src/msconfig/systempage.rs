use crate::msconfig::msconfig::DisableAllExcept;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0C46F9E4D1DD: [u16; 12] = [
    0x005Cu16, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x002E, 0x0069, 0x006E,
    0x0069, 0x0000,
];
pub static RUSTCONV_WSTR_B85BE71CE3BC: [u16; 3] = [0x0072u16, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut hSystemPage: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

#[no_mangle]
pub static mut hSystemDialog: HWND = ::core::ptr::null::<HWND__>() as *mut HWND__;

pub const BUFFER_SIZE: i32 = 512;


unsafe extern "C" fn LoadSystemIni(szPath: *mut WCHAR, hDlg: HWND) -> BOOL {
    let mut szBuffer: [WCHAR; (BUFFER_SIZE) as usize] = [0; (BUFFER_SIZE) as usize];
    let mut hDlgCtrl: HWND = core::ptr::null_mut();
    let mut parent: HTREEITEM = core::ptr::null_mut();
    let mut file: *mut FILE = core::ptr::null_mut();
    let mut length: UINT = 0;
    let mut insert: TVINSERTSTRUCT = core::mem::zeroed();
    let mut hr: HRESULT;

    hr = StringCbCopyW(
        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
        core::mem::size_of::<[WCHAR; (BUFFER_SIZE) as usize]>(),
        szPath,
    );
    if hr < 0 {
        return (FALSE) as BOOL;
    }

    hr = StringCbCatW(
        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
        core::mem::size_of::<[WCHAR; (BUFFER_SIZE) as usize]>(),
        RUSTCONV_WSTR_0C46F9E4D1DD.as_ptr(),
    );
    if hr < 0 {
        return (FALSE) as BOOL;
    }

    file = (_wfopen(
        core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
        RUSTCONV_WSTR_B85BE71CE3BC.as_ptr(),
    )).cast();
    if file.is_null() {
        return (FALSE) as BOOL;
    }

    hDlgCtrl = GetDlgItem(hDlg, (IDC_SYSTEM_TREE as i32));

    while feof(file) == 0 {
        if !fgetws(
            core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR,
            BUFFER_SIZE,
            file,
        )
        .is_null()
        {
            length = wcslen(core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR) as UINT;
            if length > 1 {
                szBuffer[length as usize] = 0x0000u16;
                szBuffer[(length - 1) as usize] = 0x0000u16;
                insert.hInsertAfter = TVI_LAST;
                insert.__bindgen_anon_1.item.mask = TVIF_TEXT;
                insert.__bindgen_anon_1.item.pszText = core::ptr::addr_of_mut!(szBuffer) as *mut WCHAR;

                if szBuffer[0 as usize] == 0x003Bu16
                    || szBuffer[0 as usize] == 0x005Bu16
                {
                    /* Parent */
                    insert.hParent = core::ptr::null_mut();
                    parent = TreeView_InsertItem!(hDlgCtrl, core::ptr::addr_of_mut!(insert));
                } else {
                    /* Child */
                    insert.hParent = parent;
                    TreeView_InsertItem!(hDlgCtrl, core::ptr::addr_of_mut!(insert));
                }
            }
        }
    }

    fclose(file);

    return (TRUE) as BOOL;
}

unsafe extern "C" fn InitializeSystemDialog(hDlg: HWND) -> BOOL {
    let mut winDir: [WCHAR; (PATH_MAX) as _] = [0; (PATH_MAX) as _];
    GetWindowsDirectoryW(core::ptr::addr_of_mut!(winDir) as LPWSTR, PATH_MAX as UINT);
    return LoadSystemIni(core::ptr::addr_of_mut!(winDir) as *mut WCHAR, hDlg);
}

#[no_mangle]
pub unsafe extern "system" fn SystemPageWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);
    match message {
        WM_INITDIALOG => {
            hSystemDialog = hDlg;
            SetWindowPos(hDlg, core::ptr::null_mut(), 10, 32, 0, 0, SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER);
            InitializeSystemDialog(hDlg);
            DisableAllExcept(hDlg, IDC_SYSTEM_TREE); // FIXME: Implement saving
            return (TRUE) as INT_PTR;
        }
        _ => {}
    }

    return 0;
}
