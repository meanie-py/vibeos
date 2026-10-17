// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_0BCCEA770ECC: [WCHAR; 10] = [0x005C, 0x0041, 0x0064, 0x0076, 0x0061, 0x006E, 0x0063, 0x0065, 0x0064, 0x0000];
pub static CPP_COM_WSTR_7289CE32E451: [WCHAR; 8] = [0x0050, 0x0072, 0x006F, 0x0067, 0x006D, 0x0061, 0x006E, 0x0000];
pub static CPP_COM_WSTR_8D5195F83F4F: [WCHAR; 14] = [0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0057, 0x0043, 0x006C, 0x0061, 0x0073, 0x0073, 0x0000];
pub static CPP_COM_WSTR_B7E9D20DE7F1: [WCHAR; 14] = [0x0043, 0x0061, 0x0062, 0x0069, 0x006E, 0x0065, 0x0074, 0x0057, 0x0043, 0x006C, 0x0061, 0x0073, 0x0073, 0x0000];

use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;
use crate::win32::*;

// ---- file-scope preamble (translated) ----
pub static __wine_dbch_fprop: [u8; 6] = *b"fprop\0";
unsafe extern "C" fn GetFolderOptions(hwndDlg: HWND) -> *mut core::ffi::c_void {
    GetWindowLongPtr(hwndDlg, GWLP_USERDATA) as *mut core::ffi::c_void
}
unsafe extern "C" fn GetRegTreeOptions(hwndDlg: HWND) -> *mut IRegTreeOptions {
    GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut IRegTreeOptions
}
unsafe extern "C" fn ViewDlg_OnDestroy(hwndDlg: HWND) {
    let pRTO = GetRegTreeOptions(hwndDlg);
    if !pRTO.is_null() {
        (*pRTO).WalkTree(WALK_TREE_DESTROY);
        (*pRTO).Release();
        SetWindowLongPtr(hwndDlg, DWLP_USER, 0);
    }
}
unsafe extern "C" fn ViewDlg_OnInitDialog(hwndDlg: HWND, psp: *mut PROPSHEETPAGE) -> BOOL {
    SetWindowLongPtr(hwndDlg, GWLP_USERDATA, (*psp).lParam as LPARAM);
    let pFO = (*psp).lParam as *mut core::ffi::c_void;

    if pFO.is_null() || CFolderOptions_CanSetDefFolderSettings_RustBridge(pFO) == 0 {
        let hwndItem = GetDlgItem(hwndDlg, (IDC_VIEW_APPLY_TO_ALL) as i32);
        EnableWindow(hwndItem, FALSE);
    }

    let mut pRTO: *mut IRegTreeOptions = core::ptr::null_mut();
    // SHCoCreateInstance lives in wine/shellole.rs, which declares its OWN `GUID`
    // record separate from shellcom's. The two are byte-identical (u32,u16,u16,[u8;8],
    // both #[repr(C)]) but are distinct types to rustc, so the GUID pointers cast at
    // this boundary. Collapsing the duplicate GUID is a separate crate-wide sweep.
    let hr = SHCoCreateInstance(
        core::ptr::null_mut(),
        core::ptr::addr_of!(CLSID_CRegTreeOptions).cast(),
        core::ptr::null_mut(),
        core::ptr::addr_of!(IID_IRegTreeOptions).cast(),
        core::ptr::addr_of_mut!(pRTO).cast(),
    );
    if hr >= 0 {
        SetWindowLongPtr(hwndDlg, DWLP_USER, pRTO as LPARAM);

        let mut apath: [core::ffi::c_char; MAX_PATH] = [0; MAX_PATH];
        SHUnicodeToAnsi(
            CPP_COM_WSTR_0BCCEA770ECC.as_ptr(),
            apath.as_mut_ptr(),
            (MAX_PATH) as i32,
        );
        let hwndTreeView = GetDlgItem(hwndDlg, (IDC_VIEW_TREEVIEW) as i32);
        (*pRTO).InitTree(
            hwndTreeView,
            HKEY_LOCAL_MACHINE,
            apath.as_mut_ptr() as *mut core::ffi::c_char,
            core::ptr::null_mut(),
        );
    }

    TRUE
}
unsafe extern "C" fn ViewDlg_ToggleCheckItem(hwndDlg: HWND, hItem: HTREEITEM) -> i32 {
    let pRTO = GetRegTreeOptions(hwndDlg);
    if pRTO.is_null() {
        return 0;
    }
    let result = (*pRTO).ToggleItem(hItem);
    (result >= 0) as i32
}
unsafe extern "C" fn ViewDlg_OnTreeViewClick(hwndDlg: HWND) {
    let hwndTreeView = GetDlgItem(hwndDlg, (IDC_VIEW_TREEVIEW) as i32);
    let mut HitTest: TV_HITTESTINFO = core::mem::zeroed();
    ZeroMemory(
        &mut HitTest as *mut TV_HITTESTINFO as *mut core::ffi::c_void,
        core::mem::size_of::<TV_HITTESTINFO>(),
    );
    let dwPos: u32 = GetMessagePos();
    HitTest.pt.x = GET_X_LPARAM(((dwPos) as isize) as _);
    HitTest.pt.y = GET_Y_LPARAM(((dwPos) as isize) as _);
    ScreenToClient(hwndTreeView, &mut HitTest.pt as *mut POINT);
    let hItem = TreeView_HitTest(hwndTreeView, &mut HitTest as *mut TV_HITTESTINFO);
    if ViewDlg_ToggleCheckItem(hwndDlg, hItem) != 0 {
        PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
    }
}
unsafe extern "C" fn ViewDlg_OnTreeViewKeyDown(hwndDlg: HWND, KeyDown: *mut TV_KEYDOWN) {
    let hwndTreeView = GetDlgItem(hwndDlg, (IDC_VIEW_TREEVIEW) as i32);

    if (*KeyDown).wVKey == (VK_SPACE) as u16 {
        let hItem = TreeView_GetSelection(hwndTreeView);
        if ViewDlg_ToggleCheckItem(hwndDlg, hItem) != 0 {
            PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
        }
    } else if (*KeyDown).wVKey == (VK_F5) as u16 {
        let pRTO = GetRegTreeOptions(hwndDlg);
        if !pRTO.is_null() {
            let vtbl = (*pRTO).lpVtbl;
            ((*vtbl).WalkTree)(pRTO, WALK_TREE_REFRESH);
        }
    }
}
unsafe extern "C" fn ViewDlg_OnTreeCustomDraw(_hwndDlg: HWND, Draw: *mut NMTVCUSTOMDRAW) -> isize {
    let nmcd: *mut NMCUSTOMDRAW = core::ptr::addr_of_mut!((*Draw).nmcd);
    if (*nmcd).dwDrawStage == CDDS_PREPAINT {
        // for CDDS_ITEMPREPAINT
        return (CDRF_NOTIFYITEMDRAW) as isize;
    } else if (*nmcd).dwDrawStage == CDDS_ITEMPREPAINT {
        if ((*nmcd).uItemState & CDIS_SELECTED) == 0 {
            let lParam: LPARAM = (*nmcd).lItemlParam;
            if (lParam as HKEY) == HKEY_REGTREEOPTION_GRAYED {
                (*Draw).clrText = GetSysColor((COLOR_GRAYTEXT) as i32);
                (*Draw).clrTextBk = GetSysColor((COLOR_WINDOW) as i32);
                return (CDRF_NEWFONT) as isize;
            }
        }
    }
    (CDRF_DODEFAULT) as isize
}
unsafe extern "C" fn ViewDlg_RestoreDefaults(hwndDlg: HWND) {
    let pRTO = GetRegTreeOptions(hwndDlg);
    if !pRTO.is_null() {
        (*pRTO).WalkTree(WALK_TREE_DEFAULT);
        PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
    }
}
// C: `static BOOL CALLBACK PostCabinetMessageCallback(HWND hWnd, LPARAM param)`.
// `CALLBACK` is __stdcall on i386 and cdecl elsewhere — which is exactly what Rust's
// `extern "system"` already means, so this is ONE function, not a cfg-split pair.
unsafe extern "system" fn PostCabinetMessageCallback(hWnd: HWND, param: LPARAM) -> BOOL {
    let data = &*(param as *const MSG);
    let mut ClassName: [u16; 100 as usize] = [0u16; 100 as usize];
    if GetClassNameW(hWnd, ClassName.as_mut_ptr(), ClassName.len() as i32) != 0 {
        if wcscmp(ClassName.as_ptr(), CPP_COM_WSTR_7289CE32E451.as_ptr()) == 0
            || wcscmp(ClassName.as_ptr(), CPP_COM_WSTR_B7E9D20DE7F1.as_ptr()) == 0
            || wcscmp(ClassName.as_ptr(), CPP_COM_WSTR_8D5195F83F4F.as_ptr()) == 0
        {
            PostMessage(hWnd, data.message, data.wParam as WPARAM, data.lParam as LPARAM);
        }
    }
    TRUE
}
#[no_mangle]
pub unsafe extern "C" fn PostCabinetMessage(Msg: u32, wParam: usize, lParam: crate::shellcom::LPARAM) {
    #[repr(C)]
    struct POINT {
        x: i32,
        y: i32,
    }
    #[repr(C)]
    struct MSG {
        hwnd: *mut core::ffi::c_void,
        message: u32,
        wParam: usize,
        lParam: isize,
        time: u32,
        pt: POINT,
    }
    let mut data: MSG = core::mem::zeroed();
    data.message = Msg;
    data.wParam = wParam;
    data.lParam = (lParam) as _;
    let lparam_ptr = core::ptr::addr_of_mut!(data) as crate::shellcom::LPARAM;
    EnumWindows(Some(PostCabinetMessageCallback), lparam_ptr);
}
unsafe extern "C" fn ViewDlg_Apply(hwndDlg: HWND) {
    let pRTO = GetRegTreeOptions(hwndDlg);
    if pRTO.is_null() || ((*pRTO).WalkTree(WALK_TREE_SAVE) < 0) {
        return;
    }

    SHSettingsChanged(core::ptr::null(), core::ptr::null()); // Invalidate restrictions
    SHGetSetSettings(core::ptr::null_mut(), 0, TRUE); // Invalidate SHELLSTATE

    let mut cs: CABINETSTATE = core::mem::zeroed();
    cs.cLength = core::mem::size_of::<CABINETSTATE>() as WORD;
    if ReadCabinetState(
        &mut cs as *mut _,
        (core::mem::size_of::<CABINETSTATE>() as u32) as i32,
    ) != 0
    {
        WriteCabinetState(&mut cs as *mut _); // Write the settings and invalidate global counter
    }

    SHSendMessageBroadcastW(WM_SETTINGCHANGE, 0, 0);
    PostCabinetMessage(WM_COMMAND, (FCIDM_DESKBROWSER_REFRESH) as usize, 0);
}
#[no_mangle]
pub unsafe extern "system" fn FolderOptionsViewDlg(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            return ((ViewDlg_OnInitDialog(hwndDlg, lParam as *mut PROPSHEETPAGE))) as INT_PTR;
        }
        WM_DESTROY => {
            ViewDlg_OnDestroy(hwndDlg);
        }
        WM_COMMAND => {
            match LOWORD((wParam) as u32) {
                id if id == IDC_VIEW_RESTORE_DEFAULTS as u16 => {
                    ViewDlg_RestoreDefaults(hwndDlg);
                }
                id if id == IDC_VIEW_APPLY_TO_ALL as u16
                    || id == IDC_VIEW_RESET_ALL as u16 =>
                {
                    let mut hr: HRESULT = HRESULT_FROM_WIN32((ERROR_NOT_SUPPORTED) as i32);
                    let reset_to_default = LOWORD((wParam) as u32) == (IDC_VIEW_RESET_ALL) as u16;
                    let pfo = GetFolderOptions(hwndDlg);
                    if !pfo.is_null() {
                        hr = CFolderOptions_ApplyDefFolderSettings_RustBridge(pfo, reset_to_default as BOOL);
                    }
                    if reset_to_default && hr == HRESULT_FROM_WIN32((ERROR_NOT_SUPPORTED) as i32) {
                        hr = CFolderOptions_ResetGlobalAndDefViewFolderSettings_RustBridge();
                    }
                    if hr < 0 {
                        SHELL_ErrorBox(hwndDlg, (hr) as u32);
                    }
                }
                _ => {}
            }
        }
        WM_NOTIFY => {
            let hdr = lParam as LPNMHDR;
            match (*hdr).code {
                NM_CLICK => {
                    ViewDlg_OnTreeViewClick(hwndDlg);
                }
                NM_CUSTOMDRAW => {
                    let result = ViewDlg_OnTreeCustomDraw(hwndDlg, lParam as *mut NMTVCUSTOMDRAW);
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, (result) as _);
                    return (result) as INT_PTR;
                }
                TVN_KEYDOWN => {
                    ViewDlg_OnTreeViewKeyDown(hwndDlg, lParam as *mut TV_KEYDOWN);
                }
                PSN_APPLY => {
                    ViewDlg_Apply(hwndDlg);
                }
                _ => {}
            }
        }
        _ => {}
    }
    ((FALSE) as isize) as _
}
