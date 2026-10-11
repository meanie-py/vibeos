use crate::progman::main::Alloc;
use crate::progman::dialog::DIALOG_CopyMove;
use crate::progman::dialog::DIALOG_ProgramAttributes;
use crate::progman::main::Free;
use crate::progman::grpfile::GRPFILE_WriteGroupFile;
use crate::progman::main::MAIN_MessageBoxIDS;
use crate::progman::main::MAIN_ReplaceString;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn PROGRAM_NewProgram(hGroup: *mut PROGGROUP) {
    let mut hIcon: HICON = core::ptr::null_mut();
    let mut nIconIndex: INT = 0;
    let mut nHotKey: INT = 0;
    let mut nCmdShow: INT = SW_SHOWNORMAL as INT;
    let mut bNewVDM: BOOL = FALSE as BOOL;
    let mut szTitleBuf: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize];
    let mut szCmdLine: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize];
    let mut szIconFile: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize];
    let mut szWorkDir: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize];

    if DIALOG_ProgramAttributes(
        &raw mut szTitleBuf as *mut WCHAR as LPWSTR,
        &raw mut szCmdLine as *mut WCHAR as LPWSTR,
        &raw mut szWorkDir as *mut WCHAR as LPWSTR,
        &raw mut szIconFile as *mut WCHAR as LPWSTR,
        &raw mut hIcon,
        &raw mut nIconIndex,
        &raw mut nHotKey,
        &raw mut nCmdShow,
        &raw mut bNewVDM,
        MAX_PATHNAME_LEN as INT,
    ) == 0
    {
        return;
    }

    if hIcon.is_null() {
        hIcon = LoadIconW(
            core::ptr::null_mut(),
            MAKEINTRESOURCEW!(IDI_WINLOGO) as LPCWSTR,
        );
    }

    if PROGRAM_AddProgram(
        hGroup,
        hIcon,
        &raw mut szTitleBuf as *mut WCHAR as LPCWSTR,
        -1,
        -1,
        &raw mut szCmdLine as *mut WCHAR as LPCWSTR,
        &raw mut szIconFile as *mut WCHAR as LPCWSTR,
        nIconIndex,
        &raw mut szWorkDir as *mut WCHAR as LPCWSTR,
        nHotKey,
        nCmdShow,
        bNewVDM,
    )
    .is_null()
    {
        return;
    }

    GRPFILE_WriteGroupFile(hGroup);
}
#[no_mangle]
pub unsafe extern "C" fn PROGRAM_ModifyProgram(hProgram: *mut PROGRAM) {
    let mut lvItem: LVITEMW = core::mem::zeroed();
    let mut szName: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize];
    let mut szWorkDir: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize];
    let mut szCmdLine: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize];
    let mut szIconFile: [WCHAR; MAX_PATHNAME_LEN as usize] = [0; MAX_PATHNAME_LEN as usize];

    lstrcpynW(
        &raw mut szName as LPWSTR,
        (*hProgram).hName as LPCWSTR,
        ARRAYSIZE!(szName) as ::core::ffi::c_int,
    );
    lstrcpynW(
        &raw mut szCmdLine as LPWSTR,
        (*hProgram).hCmdLine as LPCWSTR,
        ARRAYSIZE!(szCmdLine) as ::core::ffi::c_int,
    );
    lstrcpynW(
        &raw mut szIconFile as LPWSTR,
        (*hProgram).hIconFile as LPCWSTR,
        ARRAYSIZE!(szIconFile) as ::core::ffi::c_int,
    );
    lstrcpynW(
        &raw mut szWorkDir as LPWSTR,
        (*hProgram).hWorkDir as LPCWSTR,
        ARRAYSIZE!(szWorkDir) as ::core::ffi::c_int,
    );

    if DIALOG_ProgramAttributes(
        &raw mut szName as LPWSTR,
        &raw mut szCmdLine as LPWSTR,
        &raw mut szWorkDir as LPWSTR,
        &raw mut szIconFile as LPWSTR,
        &raw mut (*hProgram).hIcon,
        &raw mut (*hProgram).nIconIndex,
        &raw mut (*hProgram).nHotKey,
        &raw mut (*hProgram).nCmdShow,
        &raw mut (*hProgram).bNewVDM,
        MAX_PATHNAME_LEN as INT,
    ) == 0
    {
        return;
    }

    MAIN_ReplaceString(&raw mut (*hProgram).hName, &raw mut szName as LPWSTR);
    MAIN_ReplaceString(&raw mut (*hProgram).hCmdLine, &raw mut szCmdLine as LPWSTR);
    MAIN_ReplaceString(&raw mut (*hProgram).hIconFile, &raw mut szIconFile as LPWSTR);
    MAIN_ReplaceString(&raw mut (*hProgram).hWorkDir, &raw mut szWorkDir as LPWSTR);

    ZeroMemory!(
        &raw mut lvItem as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<LVITEMW>(),
    );
    lvItem.mask = LVIF_TEXT;
    lvItem.iSubItem = 0;
    lvItem.pszText = &raw mut szName as *mut WCHAR as LPWSTR;
    SendMessageW(
        (*(*hProgram).hGroup).hListView,
        LVM_SETITEMTEXTW as UINT,
        (*hProgram).iItem as WPARAM,
        &raw mut lvItem as LPARAM,
    );

    GRPFILE_WriteGroupFile((*hProgram).hGroup);
}
#[no_mangle]
pub unsafe extern "C" fn PROGRAM_AddProgram(
    hGroup: *mut PROGGROUP,
    hIcon: HICON,
    lpszName: LPCWSTR,
    x: INT,
    y: INT,
    lpszCmdLine: LPCWSTR,
    lpszIconFile: LPCWSTR,
    nIconIndex: INT,
    lpszWorkDir: LPCWSTR,
    nHotKey: INT,
    nCmdShow: INT,
    bNewVDM: BOOL,
) -> *mut PROGRAM {
    let mut hProgram: *mut PROGRAM;
    let mut hPrior: *mut PROGRAM;
    let mut p: *mut *mut PROGRAM;
    let mut hCmdLine: LPWSTR;
    let mut hIconFile: LPWSTR;
    let mut hName: LPWSTR;
    let mut hWorkDir: LPWSTR;
    let mut lvItem: LVITEMW = core::mem::zeroed();
    let mut iItem: INT;

    hProgram = Alloc(
        HEAP_ZERO_MEMORY as DWORD,
        core::mem::size_of::<PROGRAM>() as SIZE_T,
    ) as *mut PROGRAM;
    hName = Alloc(
        HEAP_ZERO_MEMORY as DWORD,
        ((wcslen(lpszName as *const wchar_t) as SIZE_T)
            .wrapping_add(1 as SIZE_T))
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    hCmdLine = Alloc(
        HEAP_ZERO_MEMORY as DWORD,
        ((wcslen(lpszCmdLine as *const wchar_t) as SIZE_T)
            .wrapping_add(1 as SIZE_T))
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    hIconFile = Alloc(
        HEAP_ZERO_MEMORY as DWORD,
        ((wcslen(lpszIconFile as *const wchar_t) as SIZE_T)
            .wrapping_add(1 as SIZE_T))
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    hWorkDir = Alloc(
        HEAP_ZERO_MEMORY as DWORD,
        ((wcslen(lpszWorkDir as *const wchar_t) as SIZE_T)
            .wrapping_add(1 as SIZE_T))
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if hProgram.is_null()
        || hName.is_null()
        || hCmdLine.is_null()
        || hIconFile.is_null()
        || hWorkDir.is_null()
    {
        MAIN_MessageBoxIDS(
            IDS_OUT_OF_MEMORY as UINT,
            IDS_ERROR as UINT,
            MB_OK as WORD,
        );
        if !hProgram.is_null() {
            Free(hProgram as PVOID);
        }
        if !hName.is_null() {
            Free(hName as PVOID);
        }
        if !hCmdLine.is_null() {
            Free(hCmdLine as PVOID);
        }
        if !hIconFile.is_null() {
            Free(hIconFile as PVOID);
        }
        if !hWorkDir.is_null() {
            Free(hWorkDir as PVOID);
        }
        return core::ptr::null_mut();
    }
    memcpy(
        hName as *mut core::ffi::c_void,
        lpszName as *const core::ffi::c_void,
        ((wcslen(lpszName as *const wchar_t) as size_t)
            .wrapping_add(1 as size_t))
            .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
    );
    memcpy(
        hCmdLine as *mut core::ffi::c_void,
        lpszCmdLine as *const core::ffi::c_void,
        ((wcslen(lpszCmdLine as *const wchar_t) as size_t)
            .wrapping_add(1 as size_t))
            .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
    );
    memcpy(
        hIconFile as *mut core::ffi::c_void,
        lpszIconFile as *const core::ffi::c_void,
        ((wcslen(lpszIconFile as *const wchar_t) as size_t)
            .wrapping_add(1 as size_t))
            .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
    );
    memcpy(
        hWorkDir as *mut core::ffi::c_void,
        lpszWorkDir as *const core::ffi::c_void,
        ((wcslen(lpszWorkDir as *const wchar_t) as size_t)
            .wrapping_add(1 as size_t))
            .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
    );

    (*hGroup).hActiveProgram = hProgram;

    hPrior = core::ptr::null_mut();
    p = &raw mut (*hGroup).hPrograms;
    while !(*p).is_null() {
        hPrior = *p;
        p = &raw mut (*hPrior).hNext;
    }
    *p = hProgram;

    (*hProgram).hGroup = hGroup;
    (*hProgram).hPrior = hPrior;
    (*hProgram).hNext = core::ptr::null_mut();
    (*hProgram).hName = hName;
    (*hProgram).hCmdLine = hCmdLine;
    (*hProgram).hIconFile = hIconFile;
    (*hProgram).nIconIndex = nIconIndex;
    (*hProgram).hWorkDir = hWorkDir;
    (*hProgram).hIcon = hIcon;
    (*hProgram).nCmdShow = nCmdShow;
    (*hProgram).nHotKey = nHotKey;
    (*hProgram).bNewVDM = bNewVDM;
    (*hProgram).TagsSize = 0 as SIZE_T;
    (*hProgram).Tags = core::ptr::null_mut();

    ZeroMemory!(
        &raw mut lvItem as *mut core::ffi::c_void,
        core::mem::size_of::<LVITEMW>() as size_t,
    );
    lvItem.mask = (LVIF_PARAM | LVIF_IMAGE | LVIF_TEXT) as UINT;
    lvItem.pszText = lpszName as LPWSTR;
    lvItem.lParam = hProgram as LPARAM;
    lvItem.iImage = ImageList_ReplaceIcon(
        (*hGroup).hListLarge,
        -(1 as INT),
        hIcon,
    );
    DestroyIcon(hIcon);

    lvItem.iItem = SendMessageA(
        (*hGroup).hListView,
        LVM_GETITEMCOUNT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as INT;
    iItem = SendMessageW(
        (*hGroup).hListView,
        LVM_INSERTITEMW as UINT,
        0 as WPARAM,
        &raw mut lvItem as LPARAM,
    ) as INT;
    (*hProgram).iItem = iItem;
    if x != -(1 as INT) && y != -(1 as INT) {
        SendMessageA(
            (*hGroup).hListView,
            LVM_SETITEMPOSITION as UINT,
            lvItem.iItem as WPARAM,
            MAKELPARAM!(x as WORD, y as WORD) as LPARAM,
        );
    }

    return hProgram as *mut PROGRAM;
}
#[no_mangle]
pub unsafe extern "C" fn PROGRAM_CopyMoveProgram(hProgram: *mut PROGRAM, bMove: BOOL) {
    let mut hGroup: *mut PROGGROUP = core::ptr::null_mut();

    hGroup = DIALOG_CopyMove(hProgram, bMove);
    if hGroup.is_null() {
        return;
    }

    /* FIXME: shouldn't be necessary */
    OpenIcon((*hGroup).hWnd);

    if PROGRAM_AddProgram(
        hGroup,
        (*hProgram).hIcon,
        (*hProgram).hName as LPCWSTR,
        (*hProgram).x,
        (*hProgram).y,
        (*hProgram).hCmdLine as LPCWSTR,
        (*hProgram).hIconFile as LPCWSTR,
        (*hProgram).nIconIndex,
        (*hProgram).hWorkDir as LPCWSTR,
        (*hProgram).nHotKey,
        (*hProgram).nCmdShow,
        (*hProgram).bNewVDM,
    )
    .is_null()
    {
        return;
    }

    GRPFILE_WriteGroupFile(hGroup);

    if bMove != 0 {
        PROGRAM_DeleteProgram(hProgram, TRUE as BOOL);
    }
}
#[no_mangle]
pub unsafe extern "C" fn PROGRAM_ExecuteProgram(hProgram: *mut PROGRAM) {
    ShellExecuteW(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (*hProgram).hCmdLine,
        core::ptr::null_mut(),
        (*hProgram).hWorkDir,
        (*hProgram).nCmdShow,
    );

    if Globals.bMinOnRun != 0 {
        CloseWindow(Globals.hMainWnd);
    }
}
#[no_mangle]
pub unsafe extern "C" fn PROGRAM_DeleteProgram(hProgram: *mut PROGRAM, bUpdateGrpFile: BOOL) {
    let mut group: *mut PROGGROUP = core::ptr::null_mut();

    group = (*hProgram).hGroup;
    if (*(*hProgram).hGroup).hActiveProgram == hProgram {
        (*group).hActiveProgram = core::ptr::null_mut();
    }

    SendMessageA(
        (*group).hListView,
        LVM_DELETEITEM as UINT,
        (*hProgram).iItem as WPARAM,
        0 as LPARAM,
    );

    if !(*hProgram).hPrior.is_null() {
        (*(*hProgram).hPrior).hNext = (*hProgram).hNext;
    } else {
        (*(*hProgram).hGroup).hPrograms = (*hProgram).hNext;
    }

    if !(*hProgram).hNext.is_null() {
        (*(*hProgram).hNext).hPrior = (*hProgram).hPrior;
    }

    if bUpdateGrpFile != 0 {
        GRPFILE_WriteGroupFile((*hProgram).hGroup);
    }

    if !(*hProgram).Tags.is_null() {
        Free((*hProgram).Tags);
    }
    Free((*hProgram).hName as PVOID);
    Free((*hProgram).hCmdLine as PVOID);
    Free((*hProgram).hIconFile as PVOID);
    Free((*hProgram).hWorkDir as PVOID);
    Free(hProgram as PVOID);
}
#[no_mangle]
pub unsafe extern "C" fn PROGRAM_ActiveProgram(hGroup: *mut PROGGROUP) -> *mut PROGRAM {
    if hGroup.is_null() {
        return core::ptr::null_mut();
    }
    if IsIconic((*hGroup).hWnd) != 0 {
        return core::ptr::null_mut();
    }
    return (*hGroup).hActiveProgram;
}
