use crate::progman::dialog::DIALOG_GroupAttributes;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_936EE4BC995B: [u16; 24] = [
    0x0025u16, 0x0064, 0x0020, 0x0025, 0x0064, 0x0020, 0x0025, 0x0064, 0x0020, 0x0025,
    0x0064, 0x0020, 0x0025, 0x0064, 0x0020, 0x0025, 0x0064, 0x0020, 0x0025, 0x0064,
    0x0020, 0x0025, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_A86DFBEF8E9A: [u16; 9] = [0x0045u16, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x0000];
pub static RUSTCONV_WSTR_FC7CAE4C93A6: [u16; 12] = [
    0x0075u16, 0x0078, 0x0074, 0x0068, 0x0065, 0x006D, 0x0065, 0x002E, 0x0064, 0x006C,
    0x006C, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const STRING_GROUP_WIN_CLASS_NAME: [::core::ffi::c_ushort; 8] = unsafe {
    ::core::mem::transmute::<[u8; 16], [::core::ffi::c_ushort; 8]>(*b"P\0M\0G\0r\0o\0u\0p\0\0\0")
};

static mut hUxTheme: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

static mut fnEnableThemeDialogTexture: ETDTProc = None;

static mut fnSetWindowTheme: SWTProc = None;

pub type ETDTProc = Option<unsafe extern "system" fn(HWND, DWORD) -> HRESULT>;

pub type SWTProc = Option<unsafe extern "system" fn(HWND, LPCWSTR, LPCWSTR) -> HRESULT>;
unsafe extern "C" fn InitUxTheme() -> BOOL {
    if !hUxTheme.is_null() { return TRUE as BOOL; }

    hUxTheme = LoadLibraryW(RUSTCONV_WSTR_FC7CAE4C93A6.as_ptr()) as HMODULE;
    if hUxTheme.is_null() { return FALSE as BOOL; }

    fnEnableThemeDialogTexture =
        core::mem::transmute::<FARPROC, ETDTProc>(GetProcAddress(hUxTheme as HINSTANCE, b"EnableThemeDialogTexture\0" as *const u8 as LPCSTR));
    fnSetWindowTheme =
        core::mem::transmute::<FARPROC, SWTProc>(GetProcAddress(hUxTheme as HINSTANCE, b"SetWindowTheme\0" as *const u8 as LPCSTR));

    return TRUE as BOOL;
}

unsafe extern "C" fn HRESULT_FROM_WIN32(x: DWORD) -> HRESULT {
    if (x as HRESULT) <= 0 {
        x as HRESULT
    } else {
        ((x & 0x0000FFFF) | (7u32 << 16) | 0x80000000) as HRESULT
    }
}

#[no_mangle]
pub unsafe extern "system" fn EnableThemeDialogTexture(hwnd: HWND, dwFlags: DWORD) -> HRESULT {
    if InitUxTheme() == 0 {
        return HRESULT_FROM_WIN32(GetLastError());
    }

    if fnEnableThemeDialogTexture.is_none() {
        return HRESULT_FROM_WIN32(GetLastError());
    }

    return (fnEnableThemeDialogTexture.unwrap_unchecked())(hwnd, dwFlags);
}

#[no_mangle]
pub unsafe extern "system" fn SetWindowTheme(
    hwnd: HWND,
    pszSubAppName: LPCWSTR,
    pszSubIdList: LPCWSTR,
) -> HRESULT {
    if InitUxTheme() == 0 {
        return HRESULT_FROM_WIN32(GetLastError());
    }

    if fnSetWindowTheme.is_none() {
        return HRESULT_FROM_WIN32(GetLastError());
    }

    return (fnSetWindowTheme.unwrap_unchecked())(hwnd, pszSubAppName, pszSubIdList);
}

unsafe extern "system" fn GROUP_GroupWndProc(
    hWnd: HWND,
    uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut group: *mut PROGGROUP;
    let mut iItem: INT;
    let mut lvItem: LVITEMW = core::mem::zeroed();
    let mut pt: POINT = core::mem::zeroed();

    group = GetWindowLongPtrW(hWnd, 0) as *mut PROGGROUP;

    match uMsg {
        WM_NCCREATE => {
            let mut pcs: LPCREATESTRUCTW = lParam as LPCREATESTRUCTW;
            let mut pMDIcs: LPMDICREATESTRUCTW = (*pcs).lpCreateParams as LPMDICREATESTRUCTW;
            group = (*pMDIcs).lParam as *mut PROGGROUP;
            SetWindowLongPtrW(hWnd, 0, group as LONG_PTR);

            if (*group).bIsCommonGroup != 0 {
                DefMDIChildProcW(
                    hWnd,
                    WM_SETICON,
                    ICON_BIG as WPARAM,
                    CopyImage(
                        Globals.hCommonGroupIcon as HANDLE,
                        IMAGE_ICON,
                        GetSystemMetrics(SM_CXICON as INT),
                        GetSystemMetrics(SM_CYICON as INT),
                        LR_COPYFROMRESOURCE,
                    ) as LPARAM,
                );
                DefMDIChildProcW(
                    hWnd,
                    WM_SETICON,
                    ICON_SMALL as WPARAM,
                    CopyImage(
                        Globals.hCommonGroupIcon as HANDLE,
                        IMAGE_ICON,
                        GetSystemMetrics(SM_CXSMICON as INT),
                        GetSystemMetrics(SM_CYSMICON as INT),
                        LR_COPYFROMRESOURCE,
                    ) as LPARAM,
                );
            } else {
                DefMDIChildProcW(
                    hWnd,
                    WM_SETICON,
                    ICON_BIG as WPARAM,
                    CopyImage(
                        Globals.hPersonalGroupIcon as HANDLE,
                        IMAGE_ICON,
                        GetSystemMetrics(SM_CXICON as INT),
                        GetSystemMetrics(SM_CYICON as INT),
                        LR_COPYFROMRESOURCE,
                    ) as LPARAM,
                );
                DefMDIChildProcW(
                    hWnd,
                    WM_SETICON,
                    ICON_SMALL as WPARAM,
                    CopyImage(
                        Globals.hPersonalGroupIcon as HANDLE,
                        IMAGE_ICON,
                        GetSystemMetrics(SM_CXSMICON as INT),
                        GetSystemMetrics(SM_CYSMICON as INT),
                        LR_COPYFROMRESOURCE,
                    ) as LPARAM,
                );
            }
        }

        WM_NCDESTROY => {
            SetWindowLongPtrW(hWnd, 0, 0);
        }

        WM_CREATE => {
            let mut dwStyle: DWORD;
            let mut rect: RECT = core::mem::zeroed();
            GetClientRect(hWnd, core::ptr::addr_of_mut!(rect));
            (*group).hListView = CreateWindowW!(
                WC_LISTVIEW,
                core::ptr::null_mut(),
                WS_CHILD | WS_VISIBLE | WS_OVERLAPPED,
                0,
                0,
                rect.right - rect.left,
                rect.bottom - rect.top,
                hWnd,
                core::ptr::null_mut(),
                Globals.hInstance,
                core::ptr::null_mut(),
            );
            dwStyle = ((GetWindowLongPtrW((*group).hListView, GWL_STYLE)
                | (LVS_SHOWSELALWAYS as LONG_PTR))
                & !(LVS_AUTOARRANGE as LONG_PTR)) as DWORD;
            SetWindowLongPtrW((*group).hListView, GWL_STYLE, dwStyle as LONG_PTR);
            dwStyle = (SendMessageA(
                (*group).hListView,
                LVM_GETEXTENDEDLISTVIEWSTYLE,
                0,
                0,
            ) | (LVS_EX_BORDERSELECT as LRESULT)) as DWORD;
            SendMessageA(
                (*group).hListView,
                LVM_SETEXTENDEDLISTVIEWSTYLE,
                LVS_EX_SNAPTOGRID as WPARAM,
                dwStyle as LPARAM,
            );
            InitUxTheme();
            SetWindowTheme(
                (*group).hListView,
                RUSTCONV_WSTR_A86DFBEF8E9A.as_ptr(),
                core::ptr::null_mut(),
            );
            (*group).hListLarge = ImageList_Create(
                GetSystemMetrics(SM_CXICON as INT),
                GetSystemMetrics(SM_CYICON as INT),
                ILC_COLOR24 | ILC_MASK,
                1,
                1,
            );
            SendMessageA(
                (*group).hListView,
                LVM_SETIMAGELIST,
                0,
                (*group).hListLarge as LPARAM,
            );
            SendMessageA(
                (*group).hListView,
                LVM_SETICONSPACING,
                0,
                MAKELPARAM!(80, 64),
            );
        }

        WM_DESTROY => {
            SendMessageA(
                (*group).hListView,
                LVM_SETIMAGELIST,
                0,
                0,
            );
            ImageList_Destroy((*group).hListLarge);
            DestroyWindow((*group).hListView);
        }

        WM_SIZE => {
            let mut rect: RECT = core::mem::zeroed();
            rect.left = 0;
            rect.top = 0;
            rect.right = LOWORD!(lParam) as LONG;
            rect.bottom = HIWORD!(lParam) as LONG;
            AdjustWindowRectEx(
                core::ptr::addr_of_mut!(rect),
                GetWindowLongPtrW((*group).hListView, GWL_STYLE) as DWORD,
                FALSE as BOOL,
                GetWindowLongPtrW((*group).hListView, GWL_EXSTYLE) as DWORD,
            );
            MoveWindow(
                (*group).hListView,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                TRUE as BOOL,
            );
        }

        WM_CLOSE => {
            SendMessageW(
                hWnd,
                WM_SYSCOMMAND,
                SC_MINIMIZE as WPARAM,
                0,
            );
        }

        WM_SYSCOMMAND => {
            if wParam == (SC_CLOSE as WPARAM) {
                wParam = SC_MINIMIZE as WPARAM;
            }
        }

        WM_CHILDACTIVATE | WM_NCLBUTTONDOWN => {
            Globals.hActiveGroup =
                GetWindowLongPtrW(hWnd, 0) as *mut PROGGROUP;
            (*Globals.hActiveGroup).hActiveProgram = core::ptr::null_mut();
        }

        WM_NOTIFY => {
            'sw: {
                match (*(lParam as LPNMHDR)).code {
                    NM_CLICK => {
                        iItem = (*(lParam as LPNMITEMACTIVATE)).iItem as INT;
                        if iItem == -1 {
                            (*group).hActiveProgram = core::ptr::null_mut();
                            break 'sw;
                        }

                        lvItem.mask = LVIF_PARAM;
                        lvItem.iItem = iItem;
                        SendMessageW(
                            (*group).hListView,
                            LVM_GETITEMW,
                            0,
                            core::ptr::addr_of_mut!(lvItem) as LPARAM,
                        );
                        (*group).hActiveProgram = lvItem.lParam as *mut PROGRAM;
                        break 'sw;
                    }

                    NM_DBLCLK => {
                        iItem = (*(lParam as LPNMITEMACTIVATE)).iItem as INT;
                        if iItem == -1 {
                            break 'sw;
                        }

                        lvItem.mask = LVIF_PARAM;
                        lvItem.iItem = iItem;
                        SendMessageW(
                            (*group).hListView,
                            LVM_GETITEMW,
                            0,
                            core::ptr::addr_of_mut!(lvItem) as LPARAM,
                        );
                        /* ... or use group->hActiveProgram */
                        PROGRAM_ExecuteProgram(lvItem.lParam as *mut PROGRAM);
                        break 'sw;
                    }

                    _ if ((*(lParam as LPNMHDR)).code as INT) == LVN_BEGINDRAG => {
                        let mut ptMin: POINT = core::mem::zeroed();

                        let mut bFirst: BOOL = TRUE as BOOL;
                        iItem = SendMessageA(
                            (*group).hListView,
                            LVM_GETNEXTITEM,
                            (-1i32) as WPARAM,
                            LVNI_SELECTED as LPARAM,
                        ) as INT;
                        while iItem != -1 {
                            if bFirst != 0 {
                                (*group).hDragImageList = SendMessageA(
                                    (*group).hListView,
                                    LVM_CREATEDRAGIMAGE,
                                    iItem as WPARAM,
                                    core::ptr::addr_of_mut!(pt) as LPARAM,
                                ) as HIMAGELIST;
                                ptMin = pt;
                                bFirst = FALSE as BOOL;
                            } else {
                                let mut hOneImageList: HIMAGELIST;
                                let mut hTempImageList: HIMAGELIST;

                                hOneImageList = SendMessageA(
                                    (*group).hListView,
                                    LVM_CREATEDRAGIMAGE,
                                    iItem as WPARAM,
                                    core::ptr::addr_of_mut!(pt) as LPARAM,
                                ) as HIMAGELIST;
                                hTempImageList = ImageList_Merge(
                                    (*group).hDragImageList,
                                    0,
                                    hOneImageList,
                                    0,
                                    pt.x - ptMin.x,
                                    pt.y - ptMin.y,
                                );
                                ImageList_Destroy((*group).hDragImageList);
                                ImageList_Destroy(hOneImageList);
                                (*group).hDragImageList = hTempImageList;
                                ptMin.x = min(ptMin.x, pt.x);
                                ptMin.y = min(ptMin.y, pt.y);
                            }
                            iItem = SendMessageA(
                                (*group).hListView,
                                LVM_GETNEXTITEM,
                                iItem as WPARAM,
                                LVNI_SELECTED as LPARAM,
                            ) as INT;
                        }
                        // pt = ((LPNMLISTVIEW)lParam)->ptAction;
                        pt.x = (*(lParam as LPNMLISTVIEW)).ptAction.x;
                        pt.y = (*(lParam as LPNMLISTVIEW)).ptAction.y;
                        (*group).ptStart = pt;
                        pt.x -= ptMin.x;
                        pt.y -= ptMin.y;
                        ImageList_BeginDrag(
                            (*group).hDragImageList,
                            0,
                            pt.x,
                            pt.y,
                        );
                        MapWindowPoints(
                            (*group).hListView,
                            Globals.hMDIWnd,
                            core::ptr::addr_of_mut!(pt),
                            1,
                        );
                        ImageList_DragEnter(Globals.hMDIWnd, pt.x, pt.y);
                        (*group).bDragging = TRUE as BOOL;
                        (*group).hOldCursor = GetCursor();
                        SetCapture((*group).hWnd);

                        break 'sw;
                    }

                    _ => {}
                }
            }
        }

        WM_MOUSEMOVE => {
            if (*group).bDragging != 0 {
                pt.x = GET_X_LPARAM!(lParam);
                pt.y = GET_Y_LPARAM!(lParam);
                MapWindowPoints(
                    (*group).hWnd,
                    Globals.hMDIWnd,
                    core::ptr::addr_of_mut!(pt),
                    1,
                );
                ImageList_DragMove(pt.x, pt.y);
            }
        }

        WM_LBUTTONUP => {
            if (*group).bDragging != 0 {
                // LVHITTESTINFO lvhti;
                let mut ptHit: POINT = core::mem::zeroed();

                (*group).bDragging = FALSE as BOOL;
                ImageList_DragLeave(Globals.hMDIWnd);
                ImageList_EndDrag();
                ImageList_Destroy((*group).hDragImageList);
                ReleaseCapture();
                SetCursor((*group).hOldCursor);
                ptHit.x = GET_X_LPARAM!(lParam);
                ptHit.y = GET_Y_LPARAM!(lParam);
                MapWindowPoints(
                    (*group).hWnd,
                    (*group).hListView,
                    core::ptr::addr_of_mut!(ptHit),
                    1,
                );
                iItem = SendMessageA(
                    (*group).hListView,
                    LVM_GETNEXTITEM,
                    (-1i32) as WPARAM,
                    LVNI_SELECTED as LPARAM,
                ) as INT;
                while iItem != -1 {
                    SendMessageA(
                        (*group).hListView,
                        LVM_GETITEMPOSITION,
                        iItem as WPARAM,
                        core::ptr::addr_of_mut!(pt) as LPARAM,
                    );
                    pt.x += ptHit.x - (*group).ptStart.x;
                    pt.y += ptHit.y - (*group).ptStart.y;
                    SendMessageA(
                        (*group).hListView,
                        LVM_SETITEMPOSITION,
                        iItem as WPARAM,
                        MAKELPARAM!(pt.x, pt.y),
                    );
                    iItem = SendMessageA(
                        (*group).hListView,
                        LVM_GETNEXTITEM,
                        iItem as WPARAM,
                        LVNI_SELECTED as LPARAM,
                    ) as INT;
                }
            }
        }

        _ => {}
    }

    return DefMDIChildProcW(hWnd, uMsg, wParam, lParam);
}
#[no_mangle]
pub unsafe extern "C" fn GROUP_RegisterGroupWinClass() -> ATOM {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();
    wndClass.style = (CS_DBLCLKS | CS_HREDRAW | CS_VREDRAW) as UINT;
    wndClass.lpfnWndProc = Some(GROUP_GroupWndProc);
    wndClass.cbClsExtra = 0 as ::core::ffi::c_int;
    wndClass.cbWndExtra = ::core::mem::size_of::<LONG_PTR>() as ::core::ffi::c_int;
    wndClass.hInstance = Globals.hInstance;
    wndClass.hIcon = LoadIconW(
        Globals.hInstance,
        MAKEINTRESOURCEW!(IDI_GROUP_ICON as WORD) as LPCWSTR,
    );
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        (IDC_ARROW as usize) as LPCWSTR,
    );
    wndClass.hbrBackground =
        ((COLOR_WINDOW + 1) as usize) as HBRUSH;
    wndClass.lpszMenuName = core::ptr::null();
    wndClass.lpszClassName = STRING_GROUP_WIN_CLASS_NAME.as_ptr();
    return RegisterClassW(core::ptr::addr_of_mut!(wndClass));
}
#[no_mangle]
pub unsafe extern "C" fn GROUP_NewGroup(mut format: GROUPFORMAT, mut bIsCommonGroup: BOOL) {
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut szGrpFile: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize];
    let mut szTitle_buf: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize];

    // ZeroMemory(szTitle, sizeof(szTitle));
    // ZeroMemory(szGrpFile, sizeof(szGrpFile));

    if DIALOG_GroupAttributes(
        format,
        core::ptr::addr_of_mut!(szTitle_buf) as LPWSTR,
        core::ptr::addr_of_mut!(szGrpFile) as LPWSTR,
        MAX_PATHNAME_LEN as INT,
    ) == 0
    {
        return;
    }

    /*
     * Just check whether the group file does exist. If it does, close the handle, because GRPFILE_ReadGroupFile will
     * reopen the file for loading. If it doesn't exist, we create a new one.
     */
    hFile = CreateFileW(
        core::ptr::addr_of_mut!(szGrpFile) as *mut WCHAR as LPCWSTR,
        GENERIC_READ,
        FILE_SHARE_READ as DWORD,
        core::ptr::null_mut(),
        OPEN_EXISTING as DWORD,
        FILE_ATTRIBUTE_NORMAL as DWORD,
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        /* File doesn't exist */
        let mut hGroup: *mut PROGGROUP = GROUP_AddGroup(
            format,
            bIsCommonGroup,
            core::ptr::addr_of_mut!(szTitle_buf) as *mut WCHAR as LPCWSTR,
            core::ptr::addr_of_mut!(szGrpFile) as *mut WCHAR as LPCWSTR,
            DEF_GROUP_WIN_XPOS as INT,
            DEF_GROUP_WIN_YPOS as INT,
            (DEF_GROUP_WIN_XPOS + DEF_GROUP_WIN_WIDTH) as INT,
            (DEF_GROUP_WIN_YPOS + DEF_GROUP_WIN_HEIGHT) as INT,
            0,
            0,
            SW_SHOWNORMAL as INT,
            0,
            0,
            FALSE as BOOL,
            FALSE as BOOL,
        );
        if !hGroup.is_null() {
            GRPFILE_WriteGroupFile(hGroup);
        }
    } else {
        /* File exist */
        CloseHandle(hFile);
        GRPFILE_ReadGroupFile(
            core::ptr::addr_of_mut!(szGrpFile) as *mut WCHAR as LPCWSTR,
            bIsCommonGroup,
        );
    }

    /* FIXME Update progman.ini */
}
#[no_mangle]
pub unsafe extern "C" fn GROUP_AddGroup(format: GROUPFORMAT, bIsCommonGroup: BOOL, lpszName: LPCWSTR, lpszGrpFile: LPCWSTR, left: INT, top: INT, right: INT, bottom: INT, xMin: INT, yMin: INT, nCmdShow: INT, cxIcon: WORD, cyIcon: WORD, bOverwriteFileOk: BOOL, bSuppressShowWindow: BOOL) -> *mut PROGGROUP {
    let mut hGroup: *mut PROGGROUP = core::ptr::null_mut();
    let mut hPrior: *mut PROGGROUP = core::ptr::null_mut();
    let mut p: *mut *mut PROGGROUP = core::ptr::null_mut();
    let mut hName: LPWSTR = core::ptr::null_mut();
    let mut hGrpFile: LPWSTR = core::ptr::null_mut();
    let mut GroupFileName: LPCWSTR = core::ptr::null();
    let mut skip: INT = 0;
    let mut width: INT = 0;
    let mut height: INT = 0;
    let mut seqnum: INT = 0;
    let mut mcs: MDICREATESTRUCTW = core::mem::zeroed();
    let mut WndPl: WINDOWPLACEMENT = core::mem::zeroed();

    WndPl.length = ::core::mem::size_of::<WINDOWPLACEMENT>() as UINT;

    // FIXME: Use system default position in case we don't place the window at a given (x,y) coordinate.

    if bIsCommonGroup != 0 {
        if swscanf(
            lpszGrpFile,
            RUSTCONV_WSTR_936EE4BC995B.as_ptr(),
            &raw mut WndPl.rcNormalPosition.left,
            &raw mut WndPl.rcNormalPosition.top,
            &raw mut WndPl.rcNormalPosition.right,
            &raw mut WndPl.rcNormalPosition.bottom,
            &raw mut WndPl.ptMinPosition.x,
            &raw mut WndPl.ptMinPosition.y,
            &raw mut WndPl.showCmd,
            &raw mut skip,
        ) == 7 as ::core::ffi::c_int
        {
            WndPl.flags = WPF_SETMINPOSITION as UINT;
            width = (WndPl.rcNormalPosition.right - WndPl.rcNormalPosition.left) as INT;
            height = (WndPl.rcNormalPosition.bottom - WndPl.rcNormalPosition.top) as INT;
            GroupFileName = lpszGrpFile.offset(skip as isize);
        } else {
            WndPl.flags = WPF_SETMINPOSITION as UINT;
            WndPl.ptMinPosition.x = xMin as LONG;
            WndPl.ptMinPosition.y = yMin as LONG;
            WndPl.rcNormalPosition.left = left as LONG;
            WndPl.rcNormalPosition.top = top as LONG;
            WndPl.rcNormalPosition.right = right as LONG;
            WndPl.rcNormalPosition.bottom = bottom as LONG;
            width = right - left;
            height = bottom - top;
            WndPl.showCmd = nCmdShow as UINT;
            GroupFileName = lpszGrpFile;
        }
    } else {
        WndPl.flags = WPF_SETMINPOSITION as UINT;
        WndPl.ptMinPosition.x = xMin as LONG;
        WndPl.ptMinPosition.y = yMin as LONG;
        WndPl.rcNormalPosition.left = left as LONG;
        WndPl.rcNormalPosition.top = top as LONG;
        WndPl.rcNormalPosition.right = right as LONG;
        WndPl.rcNormalPosition.bottom = bottom as LONG;
        width = right - left;
        height = bottom - top;
        WndPl.showCmd = nCmdShow as UINT;
        GroupFileName = lpszGrpFile;
    }

    hGroup = Alloc(
        HEAP_ZERO_MEMORY as DWORD,
        ::core::mem::size_of::<PROGGROUP>() as SIZE_T,
    ) as *mut PROGGROUP;
    hName = Alloc(
        HEAP_ZERO_MEMORY as DWORD,
        (wcslen(lpszName) as SIZE_T + 1 as SIZE_T)
            * ::core::mem::size_of::<WCHAR>() as SIZE_T,
    ) as LPWSTR;
    hGrpFile = Alloc(
        HEAP_ZERO_MEMORY as DWORD,
        (wcslen(GroupFileName) as SIZE_T + 1 as SIZE_T)
            * ::core::mem::size_of::<WCHAR>() as SIZE_T,
    ) as LPWSTR;
    if hGroup.is_null() || hName.is_null() || hGrpFile.is_null() {
        MAIN_MessageBoxIDS(
            IDS_OUT_OF_MEMORY as UINT,
            IDS_ERROR as UINT,
            MB_OK as WORD,
        );
        if !hGroup.is_null() {
            Free(hGroup as PVOID);
        }
        if !hName.is_null() {
            Free(hName as PVOID);
        }
        if !hGrpFile.is_null() {
            Free(hGrpFile as PVOID);
        }
        return core::ptr::null_mut();
    }
    memcpy(
        hName as *mut ::core::ffi::c_void,
        lpszName as *const ::core::ffi::c_void,
        (wcslen(lpszName) as size_t + 1 as size_t)
            * ::core::mem::size_of::<WCHAR>() as size_t,
    );
    memcpy(
        hGrpFile as *mut ::core::ffi::c_void,
        GroupFileName as *const ::core::ffi::c_void,
        (wcslen(GroupFileName) as size_t + 1 as size_t)
            * ::core::mem::size_of::<WCHAR>() as size_t,
    );

    Globals.hActiveGroup = hGroup;

    seqnum = 1 as INT;
    hPrior = core::ptr::null_mut();
    p = &raw mut Globals.hGroups;
    while !(*p).is_null() {
        hPrior = *p;
        if ((*hPrior).seqnum as INT) >= seqnum {
            seqnum = ((*hPrior).seqnum as INT + 1 as INT) as INT;
        }
        p = &raw mut (*hPrior).hNext;
    }
    *p = hGroup;

    (*hGroup).hPrior = hPrior;
    (*hGroup).hNext = core::ptr::null_mut();
    (*hGroup).format = format;
    (*hGroup).bIsCommonGroup = bIsCommonGroup;
    (*hGroup).hName = hName;
    (*hGroup).hGrpFile = hGrpFile;
    (*hGroup).bOverwriteFileOk = bOverwriteFileOk;
    (*hGroup).seqnum = seqnum;
    (*hGroup).nCmdShow = nCmdShow;
    (*hGroup).iconx = cxIcon as INT;
    (*hGroup).icony = cyIcon as INT;
    (*hGroup).hPrograms = core::ptr::null_mut();
    (*hGroup).hActiveProgram = core::ptr::null_mut();
    (*hGroup).TagsSize = 0 as SIZE_T;
    (*hGroup).Tags = core::ptr::null_mut();

    mcs.szClass = STRING_GROUP_WIN_CLASS_NAME.as_ptr() as LPCWSTR;
    mcs.szTitle = lpszName;
    mcs.hOwner = core::ptr::null_mut();
    mcs.x = WndPl.rcNormalPosition.left as INT;
    mcs.y = WndPl.rcNormalPosition.top as INT;
    mcs.cx = width as INT;
    mcs.cy = height as INT;
    mcs.style = 0 as DWORD;
    mcs.lParam = hGroup as LPARAM;

    (*hGroup).hWnd = SendMessageW(
        Globals.hMDIWnd,
        WM_MDICREATE as UINT,
        0 as WPARAM,
        &raw mut mcs as LPARAM,
    ) as HWND;

    SetWindowPlacement((*hGroup).hWnd, &raw mut WndPl);

    if bSuppressShowWindow == 0 {
        UpdateWindow((*hGroup).hWnd);
    }

    return hGroup;
}
#[no_mangle]
pub unsafe extern "C" fn GROUP_ModifyGroup(hGroup: *mut PROGGROUP) {
    let mut Dest: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize];
    let mut szGrpFile: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize];

    wcsncpy(
        core::ptr::addr_of_mut!(Dest) as *mut WCHAR,
        (*hGroup).hName as *const WCHAR,
        ARRAYSIZE!(Dest) as usize,
    );
    wcsncpy(
        core::ptr::addr_of_mut!(szGrpFile) as *mut WCHAR,
        (*hGroup).hGrpFile as *const WCHAR,
        ARRAYSIZE!(szGrpFile) as usize,
    );

    if DIALOG_GroupAttributes(
        (*hGroup).format,
        core::ptr::addr_of_mut!(Dest) as LPWSTR,
        core::ptr::addr_of_mut!(szGrpFile) as LPWSTR,
        MAX_PATHNAME_LEN as INT,
    ) == 0
    {
        return;
    }

    if wcscmp(
        core::ptr::addr_of_mut!(szGrpFile) as *mut WCHAR,
        (*hGroup).hGrpFile as *const WCHAR,
    ) != 0
    {
        (*hGroup).bOverwriteFileOk = FALSE as BOOL;
    }

    MAIN_ReplaceString(
        core::ptr::addr_of_mut!((*hGroup).hName),
        core::ptr::addr_of_mut!(Dest) as LPWSTR,
    );
    MAIN_ReplaceString(
        core::ptr::addr_of_mut!((*hGroup).hGrpFile),
        core::ptr::addr_of_mut!(szGrpFile) as LPWSTR,
    );

    GRPFILE_WriteGroupFile(hGroup);

    /* FIXME Delete old GrpFile if GrpFile changed */

    /* FIXME Update progman.ini */

    SetWindowTextW(
        (*hGroup).hWnd,
        core::ptr::addr_of_mut!(Dest) as *mut WCHAR as LPCWSTR,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GROUP_DeleteGroup(mut hGroup: *mut PROGGROUP) {
    if Globals.hActiveGroup == hGroup {
        Globals.hActiveGroup = ::core::ptr::null_mut::<PROGGROUP>();
    }
    if !(*hGroup).hPrior.is_null() {
        (*(*hGroup).hPrior).hNext = (*hGroup).hNext;
    } else {
        Globals.hGroups = (*hGroup).hNext;
    }
    if !(*hGroup).hNext.is_null() {
        (*(*hGroup).hNext).hPrior = (*hGroup).hPrior;
    }
    while !(*hGroup).hPrograms.is_null() {
        PROGRAM_DeleteProgram((*hGroup).hPrograms, FALSE as BOOL);
    }
    SendMessageW(
        Globals.hMDIWnd,
        WM_MDIDESTROY as UINT,
        (*hGroup).hWnd as WPARAM,
        0 as LPARAM,
    );
    if !(*hGroup).Tags.is_null() {
        Free((*hGroup).Tags);
    }
    Free((*hGroup).hName as PVOID);
    Free((*hGroup).hGrpFile as PVOID);
    Free(hGroup as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn GROUP_ShowGroupWindow(hGroup: *mut PROGGROUP) {
    ShowWindow((*hGroup).hWnd, (*hGroup).nCmdShow);
    UpdateWindow((*hGroup).hWnd);
}
#[no_mangle]
pub unsafe extern "C" fn GROUP_ActiveGroup() -> *mut PROGGROUP {
    return Globals.hActiveGroup;
}
