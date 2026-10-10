// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_01DF560B8E81: [u16; 30] = [
    0x002Au16, 0x002E, 0x0065, 0x0078, 0x0065, 0x003B, 0x002A, 0x002E, 0x0070, 0x0069,
    0x0066, 0x003B, 0x002A, 0x002E, 0x0063, 0x006F, 0x006D, 0x003B, 0x002A, 0x002E,
    0x0062, 0x0061, 0x0074, 0x003B, 0x002A, 0x002E, 0x0063, 0x006D, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_075DF3C8587C: [u16; 6] = [0x002Au16, 0x002E, 0x0069, 0x0063, 0x006F, 0x0000];
pub static RUSTCONV_WSTR_296BA9290CDB: [u16; 12] = [
    0x0057u16, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x0033, 0x002E,
    0x0031, 0x0000,
];
pub static RUSTCONV_WSTR_2E43B4AF5DB8: [u16; 10] = [0x004Du16, 0x0061, 0x0078, 0x0020, 0x0046, 0x0069, 0x006C, 0x0065, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_41BC824843A5: [u16; 6] = [0x002Au16, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_523F3509AA5C: [u16; 6] = [0x002Au16, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_69A283E639E4: [u16; 3] = [0x0025u16, 0x0075, 0x0000];
pub static RUSTCONV_WSTR_6AAEE4F6F310: [u16; 17] = [
    0x0052u16, 0x0065, 0x0063, 0x0065, 0x006E, 0x0074, 0x0020, 0x0046, 0x0069, 0x006C,
    0x0065, 0x0020, 0x004C, 0x0069, 0x0073, 0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_955300DD21E1: [u16; 4] = [0x002Au16, 0x002E, 0x002A, 0x0000];
pub static RUSTCONV_WSTR_A9AABF000DA3: [u16; 18] = [
    0x002Au16, 0x002E, 0x0069, 0x0063, 0x006F, 0x003B, 0x002A, 0x002E, 0x0065, 0x0078,
    0x0065, 0x003B, 0x002A, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_C1E3A7433EB3: [u16; 11] = [
    0x004Eu16, 0x0054, 0x0020, 0x0055, 0x006E, 0x0069, 0x0063, 0x006F, 0x0064, 0x0065,
    0x0000,
];
pub static RUSTCONV_WSTR_CE548DCD52E0: [u16; 8] = [0x004Eu16, 0x0054, 0x0020, 0x0041, 0x006E, 0x0073, 0x0069, 0x0000];
pub static RUSTCONV_WSTR_D5B988E0271D: [u16; 4] = [0x0065u16, 0x0078, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_F5F7867737F3: [u16; 16] = [
    0x0043u16, 0x0068, 0x0065, 0x0063, 0x006B, 0x0042, 0x0069, 0x006E, 0x0061, 0x0072,
    0x0079, 0x0054, 0x0079, 0x0070, 0x0065, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct NEW_ITEM_CONTEXT {
    pub nDefault: INT,
    pub nResult: INT,
}
pub type PNEW_ITEM_CONTEXT = *mut NEW_ITEM_CONTEXT;

pub type PCOPY_MOVE_CONTEXT = *mut COPY_MOVE_CONTEXT;

#[repr(C)]
pub struct GROUP_ATTRIBUTES_CONTEXT {
    pub format: GROUPFORMAT,
    pub lpszTitle: LPWSTR,
    pub lpszGrpFile: LPWSTR,
    pub nSize: INT,
}
pub type PGROUP_ATTRIBUTES_CONTEXT = *mut GROUP_ATTRIBUTES_CONTEXT;

#[repr(C)]
pub struct PICK_ICON_CONTEXT {
    pub hLibrary: HMODULE,
    pub hDlgCtrl: HWND,
    pub szName: [WCHAR; (MAX_PATH) as _], // LPWSTR lpszIconFile; // INT nSize;
    pub Index: INT,
    pub hIcon: HICON,
}
pub type PPICK_ICON_CONTEXT = *mut PICK_ICON_CONTEXT;

#[repr(C)]
pub struct PROGRAM_ATTRIBUTES_CONTEXT {
    pub lpszTitle: LPWSTR,
    pub lpszCmdLine: LPWSTR,
    pub lpszWorkDir: LPWSTR,
    pub lpszIconFile: LPWSTR,
    pub lpszTmpIconFile: LPWSTR,
    pub nSize: INT,
    pub lpnCmdShow: *mut INT,
    pub lpnHotKey: *mut INT,
    // HWND   hSelGroupWnd;
    pub lpbNewVDM: *mut BOOL, // unused!
    pub lphIcon: *mut HICON,
    pub hTmpIcon: HICON,
    pub lpnIconIndex: *mut INT,
    pub nTmpIconIndex: INT,
    pub bCheckBinaryType: BOOL,
}
pub type PPROGRAM_ATTRIBUTES_CONTEXT = *mut PROGRAM_ATTRIBUTES_CONTEXT;

#[repr(C)]
pub struct EXECUTE_CONTEXT {
    pub hKeyPMRecentFilesList: HKEY,
    pub dwMaxFiles: DWORD,
    pub bCheckBinaryType: BOOL,
}
pub type PEXECUTE_CONTEXT = *mut EXECUTE_CONTEXT;

unsafe extern "C" fn DIALOG_AddFilterItem(p: *mut LPWSTR, ids: UINT, filter: LPCWSTR) {
    LoadStringW(Globals.hInstance, ids, *p, MAX_STRING_LEN as INT);
    *p = (*p).offset((wcslen(*p as *const u16) + 1) as isize);
    lstrcpyW(*p, filter);
    *p = (*p).offset((wcslen(*p as *const u16) + 1) as isize);
    **p = 0 as WCHAR;
}

unsafe extern "C" fn DIALOG_Browse(hWnd: HWND, lpszzFilter: LPCWSTR, lpstrFile: LPWSTR, nMaxFile: INT) -> BOOL {
    let mut openfilename: OPENFILENAMEW = core::mem::zeroed();
    let mut szDir: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    ZeroMemory!(core::ptr::addr_of_mut!(openfilename) as *mut core::ffi::c_void, core::mem::size_of::<OPENFILENAMEW>());

    GetCurrentDirectoryW(ARRAYSIZE!(szDir), szDir.as_mut_ptr());

    openfilename.lStructSize = core::mem::size_of::<OPENFILENAMEW>() as DWORD;
    openfilename.hwndOwner = hWnd;
    openfilename.hInstance = Globals.hInstance;
    openfilename.lpstrFilter = lpszzFilter;
    openfilename.lpstrFile = lpstrFile;
    openfilename.nMaxFile = nMaxFile as DWORD;
    openfilename.lpstrInitialDir = szDir.as_mut_ptr();
    openfilename.Flags = OFN_EXPLORER;
    openfilename.lpstrDefExt = RUSTCONV_WSTR_D5B988E0271D.as_ptr();
    openfilename.lpstrCustomFilter = core::ptr::null_mut();
    openfilename.nMaxCustFilter = 0;
    openfilename.nFilterIndex = 0;
    openfilename.lpstrFileTitle = core::ptr::null_mut();
    openfilename.nMaxFileTitle = 0;
    openfilename.lpstrTitle = core::ptr::null_mut();
    openfilename.nFileOffset = 0;
    openfilename.nFileExtension = 0;
    openfilename.lCustData = 0;
    openfilename.lpfnHook = None;
    openfilename.lpTemplateName = core::ptr::null_mut();

    return GetOpenFileNameW(core::ptr::addr_of_mut!(openfilename));
}

unsafe extern "C" fn DIALOG_BrowsePrograms(hWnd: HWND, lpszFile: LPWSTR, nMaxFile: INT) -> BOOL {
    let mut szzFilter: [WCHAR; (2 * MAX_STRING_LEN + 100) as usize] =
        [0; (2 * MAX_STRING_LEN + 100) as usize];
    let mut p: LPWSTR = &raw mut szzFilter as LPWSTR;

    DIALOG_AddFilterItem(&raw mut p, IDS_PROGRAMS as UINT, RUSTCONV_WSTR_01DF560B8E81.as_ptr());
    DIALOG_AddFilterItem(&raw mut p, IDS_ALL_FILES as UINT, RUSTCONV_WSTR_955300DD21E1.as_ptr());

    return DIALOG_Browse(
        hWnd,
        &raw mut szzFilter as *mut WCHAR as LPCWSTR,
        lpszFile,
        nMaxFile,
    );
}

unsafe extern "C" fn DIALOG_BrowseSymbols(hWnd: HWND, lpszFile: LPWSTR, nMaxFile: INT) -> BOOL {
    let mut szzFilter: [WCHAR; 5 * MAX_STRING_LEN as usize + 100] =
        [0; 5 * MAX_STRING_LEN as usize + 100];
    let mut p: LPWSTR = &raw mut szzFilter as *mut WCHAR as LPWSTR;

    DIALOG_AddFilterItem(&raw mut p, IDS_SYMBOL_FILES as UINT, RUSTCONV_WSTR_A9AABF000DA3.as_ptr());
    DIALOG_AddFilterItem(&raw mut p, IDS_PROGRAMS as UINT, RUSTCONV_WSTR_523F3509AA5C.as_ptr());
    DIALOG_AddFilterItem(&raw mut p, IDS_LIBRARIES_DLL as UINT, RUSTCONV_WSTR_41BC824843A5.as_ptr());
    DIALOG_AddFilterItem(&raw mut p, IDS_SYMBOLS_ICO as UINT, RUSTCONV_WSTR_075DF3C8587C.as_ptr());
    DIALOG_AddFilterItem(&raw mut p, IDS_ALL_FILES as UINT, RUSTCONV_WSTR_955300DD21E1.as_ptr());

    return DIALOG_Browse(
        hWnd,
        &raw mut szzFilter as *mut WCHAR as LPCWSTR,
        lpszFile,
        nMaxFile,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GroupFormatToFormatName(Format: GROUPFORMAT) -> LPCWSTR {
    let FormatNames: [LPCWSTR; 3] = [
        RUSTCONV_WSTR_296BA9290CDB.as_ptr(),
        RUSTCONV_WSTR_CE548DCD52E0.as_ptr(),
        RUSTCONV_WSTR_C1E3A7433EB3.as_ptr(),
    ];

    if Format as ::core::ffi::c_uint > NT_Unicode as ::core::ffi::c_int as ::core::ffi::c_uint {
        return core::ptr::null_mut();
    } else {
        return FormatNames[Format as usize];
    }
}

unsafe extern "system" fn DIALOG_NEW_DlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pNewItem: PNEW_ITEM_CONTEXT = core::ptr::null_mut();
    let mut format: GROUPFORMAT = Win_311;
    let mut iItem: INT = 0;

    pNewItem = GetWindowLongPtrW(hDlg, 8) as PNEW_ITEM_CONTEXT;

    match uMsg {
        WM_INITDIALOG => {
            pNewItem = lParam as PNEW_ITEM_CONTEXT;
            SetWindowLongPtrW(hDlg, 8, lParam as LPARAM);

            format = Win_311;
            while (format as i32) <= (NT_Unicode as i32) {
                iItem = SendDlgItemMessageW(
                    hDlg,
                    PM_FORMAT as INT,
                    CB_INSERTSTRING,
                    0 as WPARAM,
                    GroupFormatToFormatName(format) as LPARAM,
                ) as INT;
                if iItem != CB_ERR && iItem != CB_ERRSPACE {
                    SendDlgItemMessageW(
                        hDlg,
                        PM_FORMAT as INT,
                        CB_SETITEMDATA,
                        iItem as WPARAM,
                        format as LPARAM,
                    );
                }
                format += 1;
            }

            SendDlgItemMessageW(
                hDlg,
                PM_FORMAT as INT,
                CB_SETCURSEL,
                0 as WPARAM,
                0 as LPARAM,
            );
            CheckRadioButton(
                hDlg,
                PM_NEW_GROUP as INT,
                PM_NEW_PROGRAM as INT,
                (*pNewItem).nDefault as INT,
            );
            CheckRadioButton(
                hDlg,
                PM_PERSONAL_GROUP as INT,
                PM_COMMON_GROUP as INT,
                PM_PERSONAL_GROUP as INT,
            );

            EnableDlgItem!(
                hDlg,
                PM_NEW_PROGRAM as INT,
                (GROUP_ActiveGroup() != core::ptr::null_mut()) as BOOL,
            );

            SendMessageW(
                hDlg,
                WM_COMMAND,
                (*pNewItem).nDefault as WPARAM,
                0 as LPARAM,
            );
            // break;
        }

        WM_COMMAND => {
            match LOWORD!(wParam) as UINT {
                PM_NEW_GROUP => {
                    CheckRadioButton(
                        hDlg,
                        PM_NEW_GROUP as INT,
                        PM_NEW_PROGRAM as INT,
                        wParam as INT,
                    );
                    EnableDlgItem!(hDlg, PM_COMMON_GROUP as INT, TRUE as BOOL);
                    EnableDlgItem!(hDlg, PM_PERSONAL_GROUP as INT, TRUE as BOOL);
                    EnableDlgItem!(hDlg, PM_FORMAT_TXT as INT, TRUE as BOOL);
                    EnableDlgItem!(hDlg, PM_FORMAT as INT, TRUE as BOOL);
                    return TRUE as INT_PTR;
                }

                PM_NEW_PROGRAM => {
                    CheckRadioButton(
                        hDlg,
                        PM_NEW_GROUP as INT,
                        PM_NEW_PROGRAM as INT,
                        wParam as INT,
                    );
                    EnableDlgItem!(hDlg, PM_COMMON_GROUP as INT, FALSE as BOOL);
                    EnableDlgItem!(hDlg, PM_PERSONAL_GROUP as INT, FALSE as BOOL);
                    EnableDlgItem!(hDlg, PM_FORMAT_TXT as INT, FALSE as BOOL);
                    EnableDlgItem!(hDlg, PM_FORMAT as INT, FALSE as BOOL);
                    return TRUE as INT_PTR;
                }

                IDHELP => {
                    MAIN_MessageBoxIDS(
                        IDS_NOT_IMPLEMENTED as UINT,
                        IDS_ERROR as UINT,
                        MB_OK as WORD,
                    );
                    return TRUE as INT_PTR;
                }

                IDOK => {
                    iItem = SendDlgItemMessageW(
                        hDlg,
                        PM_FORMAT as INT,
                        CB_GETCURSEL,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) as INT;

                    format = SendDlgItemMessageW(
                        hDlg,
                        PM_FORMAT as INT,
                        CB_GETITEMDATA,
                        iItem as WPARAM,
                        0 as LPARAM,
                    ) as GROUPFORMAT;
                    format = min(max(format as i32, Win_311 as i32), NT_Unicode as i32) as GROUPFORMAT;

                    (*pNewItem).nResult = (IsDlgButtonChecked(hDlg, PM_NEW_GROUP as INT) != 0) as INT;
                    (*pNewItem).nResult |=
                        ((IsDlgButtonChecked(hDlg, PM_COMMON_GROUP as INT) != 0) as INT)
                            << 1;
                    (*pNewItem).nResult |= (format as INT) << 2;

                    EndDialog(hDlg, IDOK as INT_PTR);
                    return TRUE as INT_PTR;
                }

                IDCANCEL => {
                    EndDialog(hDlg, IDCANCEL as INT_PTR);
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
            return FALSE as INT_PTR;
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_New(nDefault: INT, pnResult: PINT) -> BOOL {
    let mut ret: INT_PTR;
    let mut NewItem: NEW_ITEM_CONTEXT = core::mem::zeroed();

    *pnResult = 0;

    NewItem.nDefault = nDefault;
    NewItem.nResult = 0;
    ret = DialogBoxParamW(Globals.hInstance, MAKEINTRESOURCEW!(IDD_NEW), Globals.hMainWnd, Some(DIALOG_NEW_DlgProc), core::ptr::addr_of_mut!(NewItem) as LPARAM);
    if ret == IDOK as INT_PTR {
        *pnResult = NewItem.nResult;
    }

    return (ret == IDOK as INT_PTR) as BOOL;
}

unsafe extern "system" fn DIALOG_COPY_MOVE_DlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut hGrpItem: *mut PROGGROUP = core::ptr::null_mut();
    let mut hGroup: *mut PROGGROUP = core::ptr::null_mut();

    let mut pCopyMove: PCOPY_MOVE_CONTEXT = core::ptr::null_mut();
    let mut iItem: INT = 0;

    let mut text: [WCHAR; MAX_STRING_LEN as usize] = [0; MAX_STRING_LEN as usize];

    pCopyMove = GetWindowLongPtrW(hDlg, 8) as PCOPY_MOVE_CONTEXT;

    match uMsg {
        WM_INITDIALOG => {
            pCopyMove = lParam as PCOPY_MOVE_CONTEXT;
            SetWindowLongPtrW(hDlg, 8, lParam);

            if (*pCopyMove).bMove != 0 {
                LoadStringW(Globals.hInstance, IDS_MOVE_PROGRAM_1, core::ptr::addr_of_mut!(text) as LPWSTR, ARRAYSIZE!(text) as INT);
                SetWindowTextW(hDlg, core::ptr::addr_of_mut!(text) as *mut WCHAR as LPCWSTR);
                LoadStringW(Globals.hInstance, IDS_MOVE_PROGRAM_2, core::ptr::addr_of_mut!(text) as LPWSTR, ARRAYSIZE!(text) as INT);
                SetDlgItemTextW(hDlg, PM_COPY_MOVE_TXT as ::core::ffi::c_int, core::ptr::addr_of_mut!(text) as *mut WCHAR as LPCWSTR);
            }

            /* List all the group names but the source group, in case we are doing a move */
            hGroup = Globals.hGroups;
            while !hGroup.is_null() {
                if (*pCopyMove).bMove == 0 || hGroup != (*(*pCopyMove).Program).hGroup {
                    iItem = SendDlgItemMessageW(hDlg, PM_TO_GROUP as ::core::ffi::c_int, CB_ADDSTRING, 0, (*hGroup).hName as LPARAM) as INT;
                    if iItem != CB_ERR && iItem != CB_ERRSPACE {
                        SendDlgItemMessageW(hDlg, PM_TO_GROUP as ::core::ffi::c_int, CB_SETITEMDATA, iItem as WPARAM, hGroup as LPARAM);
                    }
                }
                hGroup = (*hGroup).hNext;
            }
            SendDlgItemMessageW(hDlg, PM_TO_GROUP as ::core::ffi::c_int, CB_SETCURSEL, 0, 0);
            SetDlgItemTextW(hDlg, PM_PROGRAM as ::core::ffi::c_int, (*(*pCopyMove).Program).hName as LPCWSTR);
            SetDlgItemTextW(hDlg, PM_FROM_GROUP as ::core::ffi::c_int, (*(*(*pCopyMove).Program).hGroup).hName as LPCWSTR);
            return FALSE as INT_PTR;
        }

        WM_COMMAND => {
            match LOWORD!(wParam) as UINT {
                IDHELP => {
                    MAIN_MessageBoxIDS(IDS_NOT_IMPLEMENTED, IDS_ERROR, MB_OK as WORD);
                    return TRUE as INT_PTR;
                }

                IDOK => {
                    /* Get the selected group */
                    iItem = SendDlgItemMessageW(hDlg, PM_TO_GROUP as ::core::ffi::c_int, CB_GETCURSEL, 0, 0) as INT;
                    hGrpItem = SendDlgItemMessageW(hDlg, PM_TO_GROUP as ::core::ffi::c_int, CB_GETITEMDATA, iItem as WPARAM, 0) as *mut PROGGROUP;
                    /* Check that it is indeed in the group list */
                    hGroup = Globals.hGroups;
                    while !hGroup.is_null() && hGroup != hGrpItem {
                        hGroup = (*hGroup).hNext;
                    }
                    if (*pCopyMove).bMove != 0 {
                        if hGrpItem == (*(*pCopyMove).Program).hGroup {
                            hGrpItem = core::ptr::null_mut();
                        }
                    }
                    (*pCopyMove).hToGroup = hGrpItem;
                    EndDialog(hDlg, IDOK as INT_PTR);
                    return TRUE as INT_PTR;
                }

                IDCANCEL => {
                    EndDialog(hDlg, IDCANCEL as INT_PTR);
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
            return FALSE as INT_PTR;
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_CopyMove(hProgram: *mut PROGRAM, bMove: BOOL) -> *mut PROGGROUP {
    let mut CopyMove: COPY_MOVE_CONTEXT = _COPY_MOVE_CONTEXT {
        Program: core::ptr::null_mut::<PROGRAM>(),
        hToGroup: core::ptr::null_mut::<PROGGROUP>(),
        bMove: 0,
    };
    let mut ret: INT_PTR = 0;

    CopyMove.bMove = bMove;
    CopyMove.Program = hProgram;
    CopyMove.hToGroup = core::ptr::null_mut::<PROGGROUP>();
    ret = DialogBoxParamW(
        Globals.hInstance,
        MAKEINTRESOURCEW!(IDD_COPY_MOVE),
        Globals.hMainWnd,
        Some(DIALOG_COPY_MOVE_DlgProc),
        core::ptr::addr_of_mut!(CopyMove) as LPARAM,
    );

    return if ret == IDOK as INT_PTR {
        CopyMove.hToGroup
    } else {
        core::ptr::null_mut::<PROGGROUP>()
    };
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_Delete(mut ids_text_s: UINT, mut lpszName: LPCWSTR) -> BOOL {
    return (MAIN_MessageBoxIDS_s(
        ids_text_s,
        lpszName,
        IDS_DELETE as UINT,
        (MB_YESNO | MB_DEFBUTTON2) as WORD,
    ) == IDYES as i32) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ValidateEditContents(hDlg: HWND, nIDEditItem: INT) -> BOOL {
    let mut Enable: BOOL = FALSE as BOOL;
    let mut psz: LPWSTR = core::ptr::null_mut();
    let mut Length: INT;
    let mut n: INT;
    let mut Edit: HWND;

    Edit = GetDlgItem(hDlg, nIDEditItem);
    Length = GetWindowTextLengthW(Edit);

    if Length <= 0 {
        return FALSE as BOOL;
    }

    psz = Alloc(
        0,
        ((Length + 1) as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if !psz.is_null() {
        GetWindowTextW(Edit, psz, Length + 1);
        n = 0;
        while n < Length && Enable == 0 {
            Enable = (*psz.offset(n as isize) != ' ' as WCHAR) as BOOL;
            n += 1;
        }
        Free(psz as PVOID);
    } else {
        Enable = TRUE as BOOL;
    }

    return Enable;
}

unsafe extern "system" fn DIALOG_GROUP_DlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pGroupAttributes: PGROUP_ATTRIBUTES_CONTEXT = core::ptr::null_mut();

    pGroupAttributes = GetWindowLongPtrW(hDlg, 8) as PGROUP_ATTRIBUTES_CONTEXT;

    match uMsg {
        WM_INITDIALOG => {
            let mut evMask: DWORD = 0;

            pGroupAttributes = lParam as PGROUP_ATTRIBUTES_CONTEXT;
            SetWindowLongPtrW(hDlg, 8 as _, lParam as LONG_PTR);

            /* Configure Richedit control for sending notification changes */
            evMask = (SendDlgItemMessageW(
                hDlg,
                PM_DESCRIPTION as _,
                EM_GETEVENTMASK as UINT,
                0 as WPARAM,
                0 as LPARAM,
            ) | ENM_CHANGE as LRESULT) as DWORD;
            SendDlgItemMessageW(
                hDlg,
                PM_DESCRIPTION as _,
                EM_SETEVENTMASK as UINT,
                0 as WPARAM,
                evMask as LPARAM,
            );

            SetDlgItemTextW(
                hDlg,
                PM_DESCRIPTION as _,
                (*pGroupAttributes).lpszTitle as LPCWSTR,
            );

            if (*pGroupAttributes).format != Win_311 {
                EnableDlgItem!(hDlg, PM_FILE as _, FALSE as BOOL);
            } else {
                EnableDlgItem!(hDlg, PM_FILE as _, TRUE as BOOL);
                SetDlgItemTextW(
                    hDlg,
                    PM_FILE as _,
                    (*pGroupAttributes).lpszGrpFile as LPCWSTR,
                );
            }

            EnableDlgItem!(
                hDlg,
                IDOK as _,
                ValidateEditContents(hDlg, PM_DESCRIPTION as _) as BOOL,
            );
        }

        WM_COMMAND => {
            match (wParam & 0xffff) as WORD as UINT {
                IDHELP => {
                    MAIN_MessageBoxIDS(
                        IDS_NOT_IMPLEMENTED as UINT,
                        IDS_ERROR as UINT,
                        MB_OK as WORD,
                    );
                    return TRUE as INT_PTR;
                }

                PM_DESCRIPTION => {
                    if (wParam >> 16 & 0xffff) as WORD as ::core::ffi::c_int == EN_CHANGE as ::core::ffi::c_int {
                        EnableDlgItem!(
                            hDlg,
                            IDOK as _,
                            ValidateEditContents(hDlg, PM_DESCRIPTION as _) as BOOL,
                        );
                    }
                    return FALSE as INT_PTR;
                }

                IDOK => {
                    GetDlgItemTextW(
                        hDlg,
                        PM_DESCRIPTION as _,
                        (*pGroupAttributes).lpszTitle,
                        (*pGroupAttributes).nSize,
                    );
                    if (*pGroupAttributes).format as u64 != 0 {
                        *(*pGroupAttributes).lpszGrpFile = '\0' as u16;
                    } else {
                        GetDlgItemTextW(
                            hDlg,
                            PM_FILE as _,
                            (*pGroupAttributes).lpszGrpFile,
                            (*pGroupAttributes).nSize,
                        );
                    }
                    EndDialog(hDlg, IDOK as INT_PTR);
                    return TRUE as INT_PTR;
                }

                IDCANCEL => {
                    EndDialog(hDlg, IDCANCEL as INT_PTR);
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
            return FALSE as INT_PTR;
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_GroupAttributes(format: GROUPFORMAT, lpszTitle: LPWSTR, lpszGrpFile: LPWSTR, nSize: INT) -> BOOL {
    let mut ret: INT_PTR;
    let mut GroupAttributes: GROUP_ATTRIBUTES_CONTEXT = core::mem::zeroed();

    GroupAttributes.format = format;
    GroupAttributes.nSize = nSize;
    GroupAttributes.lpszTitle = lpszTitle;
    GroupAttributes.lpszGrpFile = lpszGrpFile;

    ret = DialogBoxParamW(Globals.hInstance, MAKEINTRESOURCEW!(IDD_GROUP), Globals.hMainWnd, Some(DIALOG_GROUP_DlgProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR), core::ptr::addr_of_mut!(GroupAttributes) as LPARAM);

    return (ret == IDOK as INT_PTR) as BOOL;
}
unsafe extern "system" fn EnumPickIconResourceProc(hModule: HMODULE, lpszType: LPCWSTR, lpszName: LPWSTR, lParam: LONG_PTR) -> BOOL {
    let mut hIcon: HICON = core::ptr::null_mut();
    let hDlgCtrl: HWND = lParam as HWND;
    let mut szName: [WCHAR; 100] = [0; 100];

    if IS_INTRESOURCE!(lpszName) {
        StringCbPrintfW!(core::ptr::addr_of_mut!(szName) as *mut WCHAR, core::mem::size_of::<[WCHAR; 100]>(), RUSTCONV_WSTR_69A283E639E4.as_ptr(), lpszName as UINT_PTR as u32);
    } else {
        StringCbCopyW(core::ptr::addr_of_mut!(szName) as *mut WCHAR, core::mem::size_of::<[WCHAR; 100]>(), lpszName);
    }

    hIcon = LoadImageW(hModule as HINSTANCE, lpszName, IMAGE_ICON, 32, 32, LR_DEFAULTCOLOR) as HICON;
    if hIcon.is_null() {
        return TRUE as BOOL;
    }

    SendMessageW(hDlgCtrl, LB_ADDSTRING, 0, hIcon as LPARAM);
    return TRUE as BOOL;
}

unsafe extern "C" fn DestroyIconList(mut hDlgCtrl: HWND) {
    let mut hIcon: HICON = core::ptr::null_mut();
    let mut count: UINT = 0;
    count = SendMessageA(hDlgCtrl, LB_GETCOUNT as UINT, 0 as WPARAM, 0 as LPARAM) as UINT;
    if count == LB_ERR as UINT || count == 0 as UINT {
        return;
    }
    loop {
        let fresh31 = count;
        count = count.wrapping_sub(1);
        if !(fresh31 > 0 as UINT) {
            break;
        }
        hIcon = SendMessageA(hDlgCtrl, LB_GETITEMDATA as UINT, 0 as WPARAM, 0 as LPARAM) as HICON;
        DestroyIcon(hIcon);
        SendMessageA(hDlgCtrl, LB_DELETESTRING as UINT, 0 as WPARAM, 0 as LPARAM);
    }
}

unsafe extern "system" fn DIALOG_SYMBOL_DlgProc(
    mut hDlg: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    let mut filename: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize];
    let mut pIconContext: PPICK_ICON_CONTEXT = core::ptr::null_mut();

    pIconContext = GetWindowLongPtrW(hDlg, 8 as ::core::ffi::c_int) as PPICK_ICON_CONTEXT;

    match uMsg {
        WM_INITDIALOG => {
            pIconContext = lParam as PPICK_ICON_CONTEXT;
            SetWindowLongPtrW(hDlg, 8 as ::core::ffi::c_int, lParam as LONG_PTR);

            (*pIconContext).hDlgCtrl = GetDlgItem(hDlg, PM_SYMBOL_LIST as ::core::ffi::c_int);
            SetDlgItemTextW(
                hDlg,
                PM_ICON_FILE as ::core::ffi::c_int,
                &raw mut (*pIconContext).szName as *mut WCHAR as LPCWSTR,
            );
            SendMessageA(
                (*pIconContext).hDlgCtrl,
                LB_SETITEMHEIGHT as UINT,
                0 as WPARAM,
                32 as LPARAM,
            );

            (*pIconContext).hLibrary = LoadLibraryExW(
                &raw mut (*pIconContext).szName as *mut WCHAR as LPCWSTR,
                core::ptr::null_mut(),
                LOAD_LIBRARY_AS_DATAFILE as DWORD,
            ) as HMODULE;
            if !(*pIconContext).hLibrary.is_null() {
                EnumResourceNamesW(
                    (*pIconContext).hLibrary,
                    (RT_GROUP_ICON as usize) as LPCWSTR,
                    Some(
                        EnumPickIconResourceProc
                            as unsafe extern "system" fn(HMODULE, LPCWSTR, LPWSTR, LONG_PTR) -> BOOL,
                    ),
                    (*pIconContext).hDlgCtrl as LONG_PTR,
                );
                FreeLibrary((*pIconContext).hLibrary);
                (*pIconContext).hLibrary = core::ptr::null_mut();
            }
            SendMessageA(
                (*pIconContext).hDlgCtrl,
                LB_SETCURSEL as UINT,
                (*pIconContext).Index as WPARAM,
                0 as LPARAM,
            );
            return TRUE as INT_PTR;
        }

        WM_MEASUREITEM => {
            let mut measure: PMEASUREITEMSTRUCT = lParam as PMEASUREITEMSTRUCT;
            (*measure).itemWidth = 32 as UINT;
            (*measure).itemHeight = 32 as UINT;
            return TRUE as INT_PTR;
        }

        WM_DRAWITEM => {
            let mut dis: PDRAWITEMSTRUCT = lParam as PDRAWITEMSTRUCT;

            if ((*dis).itemState & (ODS_SELECTED as UINT)) != 0 {
                FillRect(
                    (*dis).hDC,
                    &raw mut (*dis).rcItem as LPCRECT,
                    (COLOR_HIGHLIGHT + 1) as HBRUSH,
                );
            } else {
                FillRect(
                    (*dis).hDC,
                    &raw mut (*dis).rcItem as LPCRECT,
                    (COLOR_WINDOW + 1) as HBRUSH,
                );
            }

            DrawIcon(
                (*dis).hDC,
                (*dis).rcItem.left as ::core::ffi::c_int,
                (*dis).rcItem.top as ::core::ffi::c_int,
                (*dis).itemData as HICON,
            );
            return TRUE as INT_PTR;
        }

        WM_COMMAND => {
            match LOWORD!(wParam) as UINT {
                PM_BROWSE => {
                    filename[0 as usize] = 0 as WCHAR;
                    if DIALOG_BrowseSymbols(
                        hDlg,
                        &raw mut filename as LPWSTR,
                        ARRAYSIZE!(filename) as INT,
                    ) == 0
                    {
                        return TRUE as INT_PTR;
                    }

                    if _wcsnicmp(
                        &raw mut (*pIconContext).szName as *mut WCHAR,
                        &raw mut filename as *mut WCHAR,
                        ARRAYSIZE!((*pIconContext).szName) as size_t,
                    ) == 0 as ::core::ffi::c_int
                    {
                        return TRUE as INT_PTR;
                    }

                    SetDlgItemTextW(
                        hDlg,
                        PM_ICON_FILE as ::core::ffi::c_int,
                        &raw mut filename as *mut WCHAR as LPCWSTR,
                    );
                    DestroyIconList((*pIconContext).hDlgCtrl);
                    (*pIconContext).hLibrary = LoadLibraryExW(
                        &raw mut filename as *mut WCHAR as LPCWSTR,
                        core::ptr::null_mut(),
                        LOAD_LIBRARY_AS_DATAFILE as DWORD,
                    ) as HMODULE;
                    if !(*pIconContext).hLibrary.is_null() {
                        EnumResourceNamesW(
                            (*pIconContext).hLibrary,
                            (RT_GROUP_ICON as usize) as LPCWSTR,
                            Some(
                                EnumPickIconResourceProc
                                    as unsafe extern "system" fn(HMODULE, LPCWSTR, LPWSTR, LONG_PTR) -> BOOL,
                            ),
                            (*pIconContext).hDlgCtrl as LONG_PTR,
                        );
                        FreeLibrary((*pIconContext).hLibrary);
                        (*pIconContext).hLibrary = core::ptr::null_mut();
                    }
                    SendMessageA(
                        (*pIconContext).hDlgCtrl,
                        LB_SETCURSEL as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    );
                    return TRUE as INT_PTR;
                }

                IDHELP => {
                    MAIN_MessageBoxIDS(
                        IDS_NOT_IMPLEMENTED as UINT,
                        IDS_ERROR as UINT,
                        MB_OK as WORD,
                    );
                    return TRUE as INT_PTR;
                }

                IDOK => {
                    let mut nCurSel: INT = SendMessageW(
                        (*pIconContext).hDlgCtrl,
                        LB_GETCURSEL as UINT,
                        0 as WPARAM,
                        0 as LPARAM,
                    ) as INT;
                    GetDlgItemTextW(
                        hDlg,
                        PM_ICON_FILE as ::core::ffi::c_int,
                        &raw mut (*pIconContext).szName as LPWSTR,
                        ARRAYSIZE!((*pIconContext).szName) as ::core::ffi::c_int,
                    );
                    (*pIconContext).hIcon = SendMessageA(
                        (*pIconContext).hDlgCtrl,
                        LB_GETITEMDATA as UINT,
                        nCurSel as WPARAM,
                        0 as LPARAM,
                    ) as HICON;
                    (*pIconContext).hIcon = CopyIcon((*pIconContext).hIcon);
                    DestroyIconList((*pIconContext).hDlgCtrl);
                    EndDialog(hDlg, IDOK as INT_PTR);
                    return TRUE as INT_PTR;
                }

                IDCANCEL => {
                    DestroyIconList((*pIconContext).hDlgCtrl);
                    EndDialog(hDlg, IDCANCEL as INT_PTR);
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
            return FALSE as INT_PTR;
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}

unsafe extern "C" fn DIALOG_Symbol(hWnd: HWND, lphIcon: *mut HICON, lpszIconFile: LPWSTR, lpnIconIndex: *mut INT, nSize: INT) {
    let mut IconContext: PICK_ICON_CONTEXT = core::mem::zeroed();
    IconContext.Index = *lpnIconIndex;
    StringCchCopyNW(core::ptr::addr_of_mut!(IconContext.szName) as STRSAFE_LPWSTR, ARRAYSIZE!(IconContext.szName) as size_t, lpszIconFile as STRSAFE_LPCWSTR, nSize as size_t);

    if DialogBoxParamW(Globals.hInstance, MAKEINTRESOURCEW!(IDD_SYMBOL), hWnd, Some(DIALOG_SYMBOL_DlgProc), core::ptr::addr_of_mut!(IconContext) as LPARAM) != IDOK as INT_PTR {
        return;
    }

    StringCchCopyNW(lpszIconFile as STRSAFE_LPWSTR, nSize as size_t, core::ptr::addr_of_mut!(IconContext.szName) as STRSAFE_LPCWSTR, ARRAYSIZE!(IconContext.szName) as size_t);
    *lpnIconIndex = IconContext.Index;
    *lphIcon = IconContext.hIcon;
}

unsafe extern "system" fn DIALOG_PROGRAM_DlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut bEnable: BOOL = 0;
    let mut filename: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize];
    let mut evMask: DWORD = 0;
    let mut dwBinaryType: DWORD = 0;
    let mut pProgramAttributes: PPROGRAM_ATTRIBUTES_CONTEXT = core::ptr::null_mut();

    pProgramAttributes = GetWindowLongPtrW(hDlg, 8) as PPROGRAM_ATTRIBUTES_CONTEXT;

    match uMsg {
        WM_INITDIALOG => {
            pProgramAttributes = lParam as PPROGRAM_ATTRIBUTES_CONTEXT;
            SetWindowLongPtrW(hDlg, 8, lParam);

            evMask = (SendDlgItemMessageW(hDlg, PM_COMMAND_LINE as INT, EM_GETEVENTMASK, 0, 0) as DWORD | ENM_CHANGE) as DWORD;
            SendDlgItemMessageW(hDlg, PM_COMMAND_LINE as INT, EM_SETEVENTMASK, 0, evMask as LPARAM);

            SetDlgItemTextW(hDlg, PM_DESCRIPTION as INT, (*pProgramAttributes).lpszTitle as LPCWSTR);
            SetDlgItemTextW(hDlg, PM_COMMAND_LINE as INT, (*pProgramAttributes).lpszCmdLine as LPCWSTR);
            SetDlgItemTextW(hDlg, PM_DIRECTORY as INT, (*pProgramAttributes).lpszWorkDir as LPCWSTR);

            /*                                                  0x0F                                          0x06 */
            SendDlgItemMessageW(hDlg, PM_HOT_KEY as INT, HKM_SETRULES, (HKCOMB_C | HKCOMB_A | HKCOMB_S | HKCOMB_NONE) as WPARAM, (HOTKEYF_CONTROL | HOTKEYF_ALT) as LPARAM);
            SendDlgItemMessageW(hDlg, PM_HOT_KEY as INT, HKM_SETHOTKEY, *(*pProgramAttributes).lpnHotKey as WPARAM, 0);

            bEnable = ValidateEditContents(hDlg, PM_COMMAND_LINE as INT);
            EnableWindow(GetDlgItem(hDlg, IDOK as INT), bEnable);
            EnableWindow(GetDlgItem(hDlg, PM_OTHER_SYMBOL as INT), bEnable);

            CheckDlgButton(hDlg, PM_SYMBOL as INT, (*(*pProgramAttributes).lpnCmdShow == SW_SHOWMINNOACTIVE as INT) as UINT);

            if (*pProgramAttributes).bCheckBinaryType != 0
                && (GetBinaryTypeW((*pProgramAttributes).lpszCmdLine as LPCWSTR, core::ptr::addr_of_mut!(dwBinaryType)) == 0
                    || dwBinaryType != SCS_WOW_BINARY as DWORD)
            {
                EnableWindow(GetDlgItem(hDlg, PM_NEW_VDM as INT), FALSE as BOOL);
                CheckDlgButton(hDlg, PM_NEW_VDM as INT, BST_CHECKED);
            } else {
                EnableWindow(GetDlgItem(hDlg, PM_NEW_VDM as INT), TRUE as BOOL);
                CheckDlgButton(hDlg, PM_NEW_VDM as INT, *(*pProgramAttributes).lpbNewVDM as UINT);
            }
            SendDlgItemMessageW(hDlg, PM_ICON as INT, STM_SETICON, (*pProgramAttributes).hTmpIcon as WPARAM, 0);
        }

        WM_COMMAND => {
            match LOWORD!(wParam) as u32 {
                PM_NEW_VDM => {
                    CheckDlgButton(hDlg, PM_NEW_VDM as INT, (IsDlgButtonChecked(hDlg, PM_NEW_VDM as INT) == 0) as UINT);
                    return TRUE as INT_PTR;
                }

                PM_BROWSE => {
                    filename[0 as usize] = 0;
                    if DIALOG_BrowsePrograms(hDlg, core::ptr::addr_of_mut!(filename) as LPWSTR, ARRAYSIZE!(filename) as INT) != 0 {
                        SetDlgItemTextW(hDlg, PM_COMMAND_LINE as INT, core::ptr::addr_of_mut!(filename) as LPCWSTR);
                        if (*pProgramAttributes).bCheckBinaryType != 0
                            && (GetBinaryTypeW(core::ptr::addr_of_mut!(filename) as LPCWSTR, core::ptr::addr_of_mut!(dwBinaryType)) == 0
                                || dwBinaryType != SCS_WOW_BINARY as DWORD)
                        {
                            EnableWindow(GetDlgItem(hDlg, PM_NEW_VDM as INT), FALSE as BOOL);
                            CheckDlgButton(hDlg, PM_NEW_VDM as INT, BST_CHECKED);
                        } else {
                            EnableWindow(GetDlgItem(hDlg, PM_NEW_VDM as INT), TRUE as BOOL);
                            CheckDlgButton(hDlg, PM_NEW_VDM as INT, BST_UNCHECKED);
                        }
                    }
                    return TRUE as INT_PTR;
                }

                IDHELP => {
                    MAIN_MessageBoxIDS(IDS_NOT_IMPLEMENTED, IDS_ERROR, MB_OK as WORD);
                    return TRUE as INT_PTR;
                }

                PM_SYMBOL => {
                    CheckDlgButton(hDlg, PM_SYMBOL as INT, (IsDlgButtonChecked(hDlg, PM_SYMBOL as INT) == 0) as UINT);
                    return TRUE as INT_PTR;
                }

                PM_OTHER_SYMBOL => {
                    DIALOG_Symbol(
                        hDlg,
                        core::ptr::addr_of_mut!((*pProgramAttributes).hTmpIcon),
                        (*pProgramAttributes).lpszTmpIconFile,
                        core::ptr::addr_of_mut!((*pProgramAttributes).nTmpIconIndex),
                        MAX_PATHNAME_LEN as INT,
                    );

                    SendDlgItemMessageW(hDlg, PM_ICON as INT, STM_SETICON, (*pProgramAttributes).hTmpIcon as WPARAM, 0);
                    return TRUE as INT_PTR;
                }

                PM_COMMAND_LINE => {
                    if HIWORD!(wParam) as u32 == EN_CHANGE {
                        bEnable = ValidateEditContents(hDlg, PM_COMMAND_LINE as INT);
                        EnableWindow(GetDlgItem(hDlg, IDOK as INT), bEnable);
                        EnableWindow(GetDlgItem(hDlg, PM_OTHER_SYMBOL as INT), bEnable);
                    }
                    return FALSE as INT_PTR;
                }

                IDOK => {
                    GetDlgItemTextW(hDlg, PM_DESCRIPTION as INT, (*pProgramAttributes).lpszTitle, (*pProgramAttributes).nSize);
                    GetDlgItemTextW(hDlg, PM_COMMAND_LINE as INT, (*pProgramAttributes).lpszCmdLine, (*pProgramAttributes).nSize);
                    GetDlgItemTextW(hDlg, PM_DIRECTORY as INT, (*pProgramAttributes).lpszWorkDir, (*pProgramAttributes).nSize);
                    if !(*pProgramAttributes).hTmpIcon.is_null() {
                        *(*pProgramAttributes).lphIcon = (*pProgramAttributes).hTmpIcon;
                        *(*pProgramAttributes).lpnIconIndex = (*pProgramAttributes).nTmpIconIndex;
                        lstrcpynW((*pProgramAttributes).lpszIconFile, (*pProgramAttributes).lpszTmpIconFile, (*pProgramAttributes).nSize);
                    }
                    *(*pProgramAttributes).lpnHotKey = SendDlgItemMessageW(hDlg, PM_HOT_KEY as INT, HKM_GETHOTKEY, 0, 0) as INT;
                    *(*pProgramAttributes).lpnCmdShow = if IsDlgButtonChecked(hDlg, PM_SYMBOL as INT) != 0 {
                        SW_SHOWMINNOACTIVE as INT
                    } else {
                        SW_SHOWNORMAL as INT
                    };
                    *(*pProgramAttributes).lpbNewVDM = IsDlgButtonChecked(hDlg, PM_NEW_VDM as INT) as BOOL;
                    EndDialog(hDlg, IDOK as INT_PTR);
                    return TRUE as INT_PTR;
                }

                IDCANCEL => {
                    EndDialog(hDlg, IDCANCEL as INT_PTR);
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
            return FALSE as INT_PTR;
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_ProgramAttributes(lpszTitle: LPWSTR, lpszCmdLine: LPWSTR, lpszWorkDir: LPWSTR, lpszIconFile: LPWSTR, lphIcon: *mut HICON, lpnIconIndex: *mut INT, lpnHotKey: *mut INT, lpnCmdShow: *mut INT, lpbNewVDM: *mut BOOL, nSize: INT) -> BOOL {
    let mut ret: INT_PTR;
    let mut szTmpIconFile: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize];
    let mut ProgramAttributes: PROGRAM_ATTRIBUTES_CONTEXT = core::mem::zeroed();
    let mut dwSize: DWORD;
    let mut dwType: DWORD = 0;

    ProgramAttributes.nSize = nSize;
    ProgramAttributes.lpszTitle = lpszTitle;
    ProgramAttributes.lpszCmdLine = lpszCmdLine;
    ProgramAttributes.lpszWorkDir = lpszWorkDir;
    ProgramAttributes.lpszIconFile = lpszIconFile;
    ProgramAttributes.lpnCmdShow = lpnCmdShow;
    ProgramAttributes.lpnHotKey = lpnHotKey;
    ProgramAttributes.lpbNewVDM = lpbNewVDM;
    ProgramAttributes.lphIcon = lphIcon;
    ProgramAttributes.lpnIconIndex = lpnIconIndex;

    dwSize = core::mem::size_of::<BOOL>() as DWORD;
    if RegQueryValueExW(
        Globals.hKeyPMSettings,
        RUSTCONV_WSTR_F5F7867737F3.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwType),
        core::ptr::addr_of_mut!(ProgramAttributes.bCheckBinaryType) as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    ) != ERROR_SUCCESS as i32
        || dwType != REG_DWORD
    {
        ProgramAttributes.bCheckBinaryType = TRUE as BOOL;
    }

    ProgramAttributes.hTmpIcon = *lphIcon;
    ProgramAttributes.nTmpIconIndex = *lpnIconIndex;
    ProgramAttributes.lpszTmpIconFile = core::ptr::addr_of_mut!(szTmpIconFile) as *mut WCHAR as LPWSTR;
    wcsncpy(
        core::ptr::addr_of_mut!(szTmpIconFile) as *mut WCHAR,
        lpszIconFile,
        ARRAYSIZE!(szTmpIconFile) as usize,
    );

    ret = DialogBoxParamW(
        Globals.hInstance,
        MAKEINTRESOURCEW!(IDD_PROGRAM),
        Globals.hMainWnd,
        Some(DIALOG_PROGRAM_DlgProc),
        core::ptr::addr_of_mut!(ProgramAttributes) as LPARAM,
    );
    return (ret == IDOK as INT_PTR) as BOOL;
}

unsafe extern "system" fn DIALOG_EXECUTE_DlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut szFile: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize]; // filename
    let mut BinaryType: DWORD = 0;
    let mut pExecuteContext: PEXECUTE_CONTEXT = core::ptr::null_mut();

    pExecuteContext = GetWindowLongPtrW(hDlg, 8) as PEXECUTE_CONTEXT;

    match uMsg {
        WM_INITDIALOG => {
            pExecuteContext = lParam as PEXECUTE_CONTEXT;
            SetWindowLongPtrW(hDlg, 8, lParam as LONG_PTR);

            EnableDlgItem!(hDlg, IDOK as ::core::ffi::c_int, ValidateEditContents(hDlg, PM_COMMAND as INT));

            if (*pExecuteContext).bCheckBinaryType != 0 {
                EnableDlgItem!(hDlg, PM_NEW_VDM as ::core::ffi::c_int, FALSE as BOOL);
                CheckDlgButton(hDlg, PM_NEW_VDM as ::core::ffi::c_int, BST_CHECKED);
            } else {
                EnableDlgItem!(hDlg, PM_NEW_VDM as ::core::ffi::c_int, TRUE as BOOL);
            }

            ()
        }

        WM_COMMAND => {
            match LOWORD!(wParam) as UINT {
                PM_SYMBOL => {
                    CheckDlgButton(hDlg, PM_SYMBOL as ::core::ffi::c_int, (IsDlgButtonChecked(hDlg, PM_SYMBOL as ::core::ffi::c_int) == 0) as UINT);
                    return TRUE as INT_PTR;
                }

                PM_NEW_VDM => {
                    CheckDlgButton(hDlg, PM_NEW_VDM as ::core::ffi::c_int, (IsDlgButtonChecked(hDlg, PM_NEW_VDM as ::core::ffi::c_int) == 0) as UINT);
                    return TRUE as INT_PTR;
                }

                PM_BROWSE => {
                    szFile[0 as usize] = 0 as WCHAR; // '\0'
                    if DIALOG_BrowsePrograms(
                        hDlg,
                        core::ptr::addr_of_mut!(szFile) as *mut WCHAR as LPWSTR,
                        ARRAYSIZE!(szFile) as INT,
                    ) != 0
                    {
                        SetWindowTextW(
                            GetDlgItem(hDlg, PM_COMMAND as ::core::ffi::c_int),
                            core::ptr::addr_of_mut!(szFile) as *mut WCHAR as LPCWSTR,
                        );
                        SetFocus(GetDlgItem(hDlg, IDOK as ::core::ffi::c_int));
                        if (*pExecuteContext).bCheckBinaryType != 0
                            && (GetBinaryTypeW(
                                core::ptr::addr_of_mut!(szFile) as *mut WCHAR as LPCWSTR,
                                core::ptr::addr_of_mut!(BinaryType),
                            ) == 0
                                || BinaryType != SCS_WOW_BINARY as DWORD)
                        {
                            EnableDlgItem!(hDlg, PM_NEW_VDM as ::core::ffi::c_int, FALSE as BOOL);
                            CheckDlgButton(hDlg, PM_NEW_VDM as ::core::ffi::c_int, BST_CHECKED);
                        } else {
                            EnableDlgItem!(hDlg, PM_NEW_VDM as ::core::ffi::c_int, TRUE as BOOL);
                            CheckDlgButton(hDlg, PM_NEW_VDM as ::core::ffi::c_int, BST_UNCHECKED);
                        }
                    }
                    return TRUE as INT_PTR;
                }

                PM_COMMAND => {
                    if HIWORD!(wParam) as UINT == CBN_EDITCHANGE {
                        EnableDlgItem!(hDlg, IDOK as ::core::ffi::c_int, ValidateEditContents(hDlg, PM_COMMAND as INT));
                    }
                    return FALSE as INT_PTR;
                }

                IDHELP => {
                    MAIN_MessageBoxIDS(IDS_NOT_IMPLEMENTED, IDS_ERROR, MB_OK as WORD);
                    return TRUE as INT_PTR;
                }

                IDOK => {
                    GetDlgItemTextW(
                        hDlg,
                        PM_COMMAND as ::core::ffi::c_int,
                        core::ptr::addr_of_mut!(szFile) as *mut WCHAR as LPWSTR,
                        ARRAYSIZE!(szFile) as INT,
                    );
                    ShellExecuteW(
                        core::ptr::null_mut(),
                        core::ptr::null(),
                        core::ptr::addr_of_mut!(szFile) as *mut WCHAR as LPCWSTR,
                        core::ptr::null(),
                        core::ptr::null(),
                        if IsDlgButtonChecked(hDlg, PM_SYMBOL as ::core::ffi::c_int) != 0 {
                            SW_SHOWMINNOACTIVE as INT
                        } else {
                            SW_SHOWNORMAL as INT
                        },
                    );
                    if Globals.bMinOnRun != 0 {
                        CloseWindow(Globals.hMainWnd);
                    }
                    EndDialog(hDlg, IDOK as INT_PTR);
                    return TRUE as INT_PTR;
                }

                IDCANCEL => {
                    EndDialog(hDlg, IDCANCEL as INT_PTR);
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
            return FALSE as INT_PTR;
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn DIALOG_Execute() {
    let mut ExecuteContext: EXECUTE_CONTEXT = EXECUTE_CONTEXT {
        hKeyPMRecentFilesList: core::ptr::null_mut(),
        dwMaxFiles: 0,
        bCheckBinaryType: 0,
    };
    let mut lRet: LONG;
    let mut dwSize: DWORD;
    let mut dwType: DWORD = 0;

    ExecuteContext.hKeyPMRecentFilesList = core::ptr::null_mut();
    ExecuteContext.bCheckBinaryType = TRUE as BOOL;

    lRet = RegCreateKeyExW(
        Globals.hKeyProgMan,
        RUSTCONV_WSTR_6AAEE4F6F310.as_ptr() as LPCWSTR,
        0 as DWORD,
        core::ptr::null_mut(),
        0 as DWORD,
        (KEY_READ | KEY_WRITE) as REGSAM,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(ExecuteContext.hKeyPMRecentFilesList),
        core::ptr::null_mut(),
    ) as LONG;
    if lRet == ERROR_SUCCESS as LONG {
        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        lRet = RegQueryValueExW(
            ExecuteContext.hKeyPMRecentFilesList,
            RUSTCONV_WSTR_2E43B4AF5DB8.as_ptr() as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            core::ptr::addr_of_mut!(ExecuteContext.dwMaxFiles) as LPBYTE,
            core::ptr::addr_of_mut!(dwSize),
        ) as LONG;
        if lRet != ERROR_SUCCESS as LONG || dwType != REG_DWORD as DWORD {
            ExecuteContext.dwMaxFiles = 4 as DWORD;
            dwSize = core::mem::size_of::<DWORD>() as DWORD;
            lRet = RegSetValueExW(
                ExecuteContext.hKeyPMRecentFilesList,
                RUSTCONV_WSTR_2E43B4AF5DB8.as_ptr() as LPCWSTR,
                0 as DWORD,
                REG_DWORD as DWORD,
                core::ptr::addr_of_mut!(ExecuteContext.dwMaxFiles) as LPBYTE as *const BYTE,
                core::mem::size_of::<DWORD>() as DWORD,
            ) as LONG;
        }

        dwSize = core::mem::size_of::<BOOL>() as DWORD;
        lRet = RegQueryValueExW(
            Globals.hKeyPMSettings,
            RUSTCONV_WSTR_F5F7867737F3.as_ptr() as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            core::ptr::addr_of_mut!(ExecuteContext.bCheckBinaryType) as LPBYTE,
            core::ptr::addr_of_mut!(dwSize),
        ) as LONG;
        if lRet != ERROR_SUCCESS as LONG || dwType != REG_DWORD as DWORD {
            ExecuteContext.bCheckBinaryType = TRUE as BOOL;
        }
    }

    DialogBoxParamW(
        Globals.hInstance,
        MAKEINTRESOURCEW!(IDD_EXECUTE),
        Globals.hMainWnd,
        Some(DIALOG_EXECUTE_DlgProc),
        core::ptr::addr_of_mut!(ExecuteContext) as LPARAM,
    );

    if !ExecuteContext.hKeyPMRecentFilesList.is_null() {
        RegCloseKey(ExecuteContext.hKeyPMRecentFilesList);
    }
}
