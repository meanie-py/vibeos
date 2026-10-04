// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_BACKGROUNDS: usize = 100;

pub const PLACEMENT_CENTER: u32 = 0;
pub const PLACEMENT_STRETCH: u32 = 1;
pub const PLACEMENT_TILE: u32 = 2;
pub const PLACEMENT_FIT: u32 = 3;
pub const PLACEMENT_FILL: u32 = 4;

pub const PLACEMENT_VALUE_CENTER: u32 = 0;
pub const PLACEMENT_VALUE_STRETCH: u32 = 2;
pub const PLACEMENT_VALUE_TILE: u32 = 0;
pub const PLACEMENT_VALUE_FIT: u32 = 6;
pub const PLACEMENT_VALUE_FILL: u32 = 10;

#[repr(C)]
pub struct BackgroundItem {
    pub bWallpaper: BOOL,
    pub szFilename: [TCHAR; (MAX_PATH) as usize],
    pub szDisplayName: [TCHAR; 256],
}

#[repr(C)]
pub struct _BACKGROUND_DATA {
    pub bWallpaperChanged: BOOL,
    pub bClrBackgroundChanged: BOOL,
    pub backgroundItems: [BackgroundItem; MAX_BACKGROUNDS],
    pub pWallpaperBitmap: PDIBITMAP,
    pub placementSelection: i32,
    pub backgroundSelection: i32,
    pub custom_colors: [COLORREF; 16],
    pub listViewItemCount: i32,
    pub gdipToken: ULONG_PTR,
    pub desktopData: DESKTOP_DATA,
}

pub type PBACKGROUND_DATA = *mut _BACKGROUND_DATA;
pub type BACKGROUND_DATA = _BACKGROUND_DATA;

#[no_mangle]
pub static mut g_GlobalData: GLOBAL_DATA = unsafe { core::mem::zeroed() };
#[no_mangle]
pub unsafe extern "C" fn GdipGetEncoderClsid(MimeType: PCWSTR, pClsid: *mut CLSID) -> HRESULT {
    let mut num: u32 = 0;
    let mut size: u32 = 0;
    let mut i: u32;
    let mut codecInfo: *mut ImageCodecInfo;

    if GdipGetImageEncodersSize(&mut num, &mut size) != Ok ||
        size == 0 {
        return E_FAIL;
    }

    codecInfo = HeapAlloc(GetProcessHeap(), 0, (size) as SIZE_T) as *mut ImageCodecInfo;
    if codecInfo.is_null() {
        return E_OUTOFMEMORY;
    }

    if GdipGetImageEncoders(num, size, codecInfo) != Ok {
        HeapFree(GetProcessHeap(), 0, codecInfo as *mut _);
        return E_FAIL;
    }

    i = 0;
    while i < num {
        if _wcsicmp((*codecInfo.offset(i as isize)).MimeType, MimeType) == 0 {
            *pClsid = (*codecInfo.offset(i as isize)).Clsid;
            HeapFree(GetProcessHeap(), 0, codecInfo as *mut _);
            return S_OK;
        }
        i += 1;
    }

    HeapFree(GetProcessHeap(), 0, codecInfo as *mut _);
    E_FAIL
}
#[no_mangle]
pub unsafe extern "C" fn GdipGetSupportedFileExtensions() -> LPWSTR {
    let mut codecInfo: *mut ImageCodecInfo;
    let mut num: u32 = 0;
    let mut size: u32 = 0;
    let mut i: u32;
    let mut lpBuffer: LPWSTR = core::ptr::null_mut();

    if GdipGetImageDecodersSize(&mut num, &mut size) != Ok ||
        size == 0 {
        return core::ptr::null_mut();
    }

    codecInfo = HeapAlloc(GetProcessHeap(), 0, (size) as SIZE_T) as *mut ImageCodecInfo;
    if codecInfo.is_null() {
        return core::ptr::null_mut();
    }

    if GdipGetImageDecoders(num, size, codecInfo) != Ok {
        HeapFree(GetProcessHeap(), 0, codecInfo as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    size = 0;
    i = 0;
    while i < num {
        size = size + (wcslen((*codecInfo.offset(i as isize)).FilenameExtension) as u32) + 1;
        i += 1;
    }

    size = (size + 1) * core::mem::size_of::<u16>() as u32;

    lpBuffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (size) as SIZE_T) as LPWSTR;
    if lpBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, codecInfo as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    i = 0;
    while i < num {
        if lstrcmpiW((*codecInfo.offset(i as isize)).FilenameExtension, w!("*.ico") as LPWSTR) == 0 {
            i += 1;
            continue;
        }

        StringCbCatW(lpBuffer, size as usize, (*codecInfo.offset(i as isize)).FilenameExtension);
        if i < (num - 1) {
            StringCbCatW(lpBuffer, size as usize, w!(";") as LPWSTR);
        }
        i += 1;
    }

    HeapFree(GetProcessHeap(), 0, codecInfo as *mut core::ffi::c_void);

    lpBuffer
}
unsafe extern "C" fn AddWallpapersFromDirectory(
    mut uCounter: u32,
    hwndBackgroundList: HWND,
    mut backgroundItem: *mut BackgroundItem,
    pData: PBACKGROUND_DATA,
    wallpaperFilename: LPCTSTR,
    wallpaperDirectory: LPCTSTR,
) -> u32 {
    let mut fd: WIN32_FIND_DATA = core::mem::zeroed();
    let mut hFind: HANDLE;
    let mut szSearchPath: [TCHAR; (MAX_PATH) as usize] = core::mem::zeroed();
    let mut szFileTypes: LPTSTR = core::ptr::null_mut();
    let mut separators: [TCHAR; 2] = wide!(";");
    let mut token: *mut TCHAR;
    let mut hr: HRESULT;
    let mut sfi: SHFILEINFO = core::mem::zeroed();
    let mut i: u32 = uCounter;
    let mut listItem: LV_ITEM = core::mem::zeroed();
    let himl: HIMAGELIST;

    szFileTypes = GdipGetSupportedFileExtensions();
    if szFileTypes.is_null() {
        return i;
    }

    himl = ListView_GetImageList!(hwndBackgroundList, LVSIL_SMALL);

    token = _tcstok(szFileTypes, separators.as_ptr() as *const TCHAR);
    while !token.is_null() {
        if PathCombine(szSearchPath.as_mut_ptr(), wallpaperDirectory, token) == (0) as *mut u16 {
            HeapFree(GetProcessHeap(), 0, szFileTypes as LPVOID);
            return i;
        }

        hFind = FindFirstFile(szSearchPath.as_ptr(), &mut fd);
        while hFind != INVALID_HANDLE_VALUE {
            let mut filename: [TCHAR; (MAX_PATH) as usize] = core::mem::zeroed();

            if PathCombine(filename.as_mut_ptr(), wallpaperDirectory, (fd.cFileName.as_ptr()).cast()) == (0) as *mut u16 {
                FindClose(hFind);
                HeapFree(GetProcessHeap(), 0, szFileTypes as LPVOID);
                return i;
            }

            if ((fd.dwFileAttributes & FILE_ATTRIBUTE_HIDDEN) == 0)
                && (_tcsicmp(wallpaperFilename, filename.as_ptr()) != 0)
            {
                SHGetFileInfo(
                    filename.as_ptr(),
                    0,
                    &mut sfi,
                    core::mem::size_of::<SHFILEINFO>() as u32,
                    SHGFI_ICON | SHGFI_SMALLICON | SHGFI_DISPLAYNAME,
                );
                sfi.iIcon = ImageList_AddIcon!(himl, sfi.hIcon);
                i += 1;

                backgroundItem = &mut (*pData).backgroundItems[(*pData).listViewItemCount as usize];

                (*backgroundItem).bWallpaper = (TRUE) as i32;

                hr = StringCbCopy(
                    (*backgroundItem).szDisplayName.as_mut_ptr(),
                    core::mem::size_of::<[TCHAR; 256]>(),
                    core::ptr::addr_of!(sfi.szDisplayName).cast(),
                );
                if FAILED!(hr) {
                    FindClose(hFind);
                    HeapFree(GetProcessHeap(), 0, szFileTypes as LPVOID);
                    return i;
                }

                PathRemoveExtension((*backgroundItem).szDisplayName.as_mut_ptr());

                hr = StringCbCopy(
                    ((*backgroundItem).szFilename.as_mut_ptr() as *mut core::ffi::c_void).cast(),
                    core::mem::size_of::<[TCHAR; MAX_PATH as usize]>(),
                    (filename.as_ptr() as *const core::ffi::c_void).cast(),
                );
                if FAILED!(hr) {
                    FindClose(hFind);
                    HeapFree(GetProcessHeap(), 0, szFileTypes as LPVOID);
                    return i;
                }

                ZeroMemory!(
                    &mut listItem as *mut _ as *mut core::ffi::c_void,
                    core::mem::size_of::<LV_ITEM>(),
                );
                listItem.mask = LVIF_TEXT | LVIF_PARAM | LVIF_STATE | LVIF_IMAGE;
                listItem.pszText = (*backgroundItem).szDisplayName.as_mut_ptr();
                listItem.state = 0;
                listItem.iImage = sfi.iIcon;
                listItem.iItem = (*pData).listViewItemCount as i32;
                listItem.lParam = (*pData).listViewItemCount as LPARAM;

                ListView_InsertItem!(hwndBackgroundList, &listItem);

                (*pData).listViewItemCount += 1;
            }

            if FindNextFile(hFind, &mut fd) == 0 {
                break;
            }
        }

        token = _tcstok(core::ptr::null_mut(), separators.as_ptr() as *const TCHAR);
        FindClose(hFind);
    }

    HeapFree(GetProcessHeap(), 0, szFileTypes as LPVOID);

    return i;
}
unsafe extern "C" fn AddListViewItems(hwndDlg: HWND, pData: PBACKGROUND_DATA) {
    let mut szSearchPath: [TCHAR; (MAX_PATH) as usize] = core::mem::zeroed();
    let mut listItem: LV_ITEM = core::mem::zeroed();
    let mut dummy: LV_COLUMN = core::mem::zeroed();
    let mut clientRect: RECT = core::mem::zeroed();
    let mut regKey: HKEY = core::mem::zeroed();
    let mut sfi: SHFILEINFO = core::mem::zeroed();
    let mut himl: HIMAGELIST;
    let mut wallpaperFilename: [TCHAR; (MAX_PATH) as usize] = core::mem::zeroed();
    let mut originalWallpaper: [TCHAR; (MAX_PATH) as usize] = core::mem::zeroed();
    let mut bufferSize: u32 = core::mem::size_of_val(&wallpaperFilename) as u32;
    let mut buffer: [TCHAR; (MAX_PATH) as usize] = core::mem::zeroed();
    let mut varType: u32 = REG_SZ;
    let mut result: i32;
    let mut i: u32 = 0;
    let mut backgroundItem: *mut BackgroundItem = core::ptr::null_mut();
    let hwndBackgroundList: HWND;
    let mut hr: HRESULT;
    let mut hIcon: HICON;
    let cx: i32;
    let cy: i32;
    let hShell32: HINSTANCE;

    hwndBackgroundList = GetDlgItem(hwndDlg, (IDC_BACKGROUND_LIST) as i32);

    GetClientRect(hwndBackgroundList, &mut clientRect);

    cx = GetSystemMetrics((SM_CXSMICON) as i32);
    cy = GetSystemMetrics((SM_CYSMICON) as i32);
    himl = ImageList_Create(cx, cy, ILC_COLOR32 | ILC_MASK, 0, 0);

    hShell32 = GetModuleHandleW(w!("shell32.dll") as PCWSTR);
    hIcon = LoadImageW(hShell32, MAKEINTRESOURCEW!(IDI_SHELL_NO), IMAGE_ICON, cx, cy, 0) as HICON;

    ListView_SetImageList!(hwndBackgroundList, himl, LVSIL_SMALL);

    ZeroMemory!(&mut dummy as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<LV_COLUMN>());
    dummy.mask = LVCF_SUBITEM | LVCF_WIDTH;
    dummy.iSubItem = 0;
    dummy.cx = (clientRect.right - clientRect.left) - GetSystemMetrics((SM_CXVSCROLL) as i32);
    ListView_InsertColumn!(hwndBackgroundList, 0, &mut dummy);

    backgroundItem = &mut (*pData).backgroundItems[(*pData).listViewItemCount as usize];
    (*backgroundItem).bWallpaper = (FALSE) as i32;
    LoadString(
        hApplet,
        IDS_NONE,
        (*backgroundItem).szDisplayName.as_mut_ptr(),
        (core::mem::size_of::<[TCHAR; 256]>() / core::mem::size_of::<TCHAR>()) as i32,
    );

    ZeroMemory!(&mut listItem as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<LV_ITEM>());
    listItem.mask = LVIF_TEXT | LVIF_PARAM | LVIF_STATE | LVIF_IMAGE;
    listItem.state = 0;
    listItem.pszText = (*backgroundItem).szDisplayName.as_mut_ptr();
    listItem.iImage = ImageList_AddIcon!(himl, hIcon);
    listItem.iItem = (*pData).listViewItemCount as i32;
    listItem.lParam = (((*pData).listViewItemCount as isize) as i32) as LPARAM;
    hIcon = core::ptr::null_mut();

    ListView_InsertItem!(hwndBackgroundList, &mut listItem);
    ListView_SetItemState!(
        hwndBackgroundList,
        (*pData).listViewItemCount as i32,
        LVIS_SELECTED,
        LVIS_SELECTED,
    );

    (*pData).listViewItemCount += 1;

    result = RegOpenKeyEx(HKEY_CURRENT_USER, w!("Control Panel\\Desktop"), 0, KEY_QUERY_VALUE, &mut regKey);
    if result == (ERROR_SUCCESS) as i32 {
        result = RegQueryValueEx(regKey, w!("Wallpaper"), (0) as *mut u32, &mut varType, wallpaperFilename.as_mut_ptr() as LPBYTE, &mut bufferSize);
        if (result == (ERROR_SUCCESS) as i32) && (_tcslen(wallpaperFilename.as_ptr()) > 0) {
            bufferSize = core::mem::size_of_val(&originalWallpaper) as u32;
            result = RegQueryValueEx(regKey, w!("OriginalWallpaper"), (0) as *mut u32, &mut varType, originalWallpaper.as_mut_ptr() as LPBYTE, &mut bufferSize);

            if (result == (ERROR_SUCCESS) as i32) && (_tcslen(originalWallpaper.as_ptr()) > 0) && (_tcsicmp(wallpaperFilename.as_ptr(), originalWallpaper.as_ptr()) == 0) {
                bufferSize = core::mem::size_of_val(&originalWallpaper) as u32;
                result = RegQueryValueEx(regKey, w!("ConvertedWallpaper"), (0) as *mut u32, &mut varType, originalWallpaper.as_mut_ptr() as LPBYTE, &mut bufferSize);

                if (result == (ERROR_SUCCESS) as i32) && (_tcslen(originalWallpaper.as_ptr()) > 0) {
                    hr = StringCbCopy(wallpaperFilename.as_mut_ptr(), core::mem::size_of_val(&wallpaperFilename), originalWallpaper.as_ptr());
                    if hr < 0 {
                        RegCloseKey(regKey);
                        return;
                    }
                }
            }

            if ExpandEnvironmentStrings(wallpaperFilename.as_ptr(), buffer.as_mut_ptr(), MAX_PATH) != 0 {
                hr = StringCbCopy(wallpaperFilename.as_mut_ptr(), core::mem::size_of_val(&wallpaperFilename), buffer.as_ptr());
                if hr < 0 {
                    RegCloseKey(regKey);
                    return;
                }
            }

            // Upstream 3016c40dc72 (CORE-19896): a wallpaper file that has since been deleted
            // must not be listed, and the icon SHGetFileInfoW hands back is ours to destroy
            // once the image list has copied it (it leaked one HICON per page open before).
            if SHGetFileInfoW(
                wallpaperFilename.as_ptr(),
                0,
                (&mut sfi as *mut _ as *mut core::ffi::c_void).cast(),
                core::mem::size_of_val(&sfi) as u32,
                SHGFI_ICON | SHGFI_SMALLICON | SHGFI_DISPLAYNAME,
            ) == 0
            {
                RegCloseKey(regKey);
                return;
            }
            sfi.iIcon = ImageList_AddIcon!(himl, sfi.hIcon);
            DestroyIcon(sfi.hIcon);

            i += 1;

            backgroundItem = &mut (*pData).backgroundItems[(*pData).listViewItemCount as usize];

            (*backgroundItem).bWallpaper = (TRUE) as i32;

            hr = StringCbCopy((*backgroundItem).szDisplayName.as_mut_ptr(), core::mem::size_of::<[TCHAR; 256]>(), core::ptr::addr_of!(sfi.szDisplayName).cast());
            if hr < 0 {
                RegCloseKey(regKey);
                return;
            }

            PathRemoveExtension((*backgroundItem).szDisplayName.as_mut_ptr());

            hr = StringCbCopy((*backgroundItem).szFilename.as_mut_ptr(), core::mem::size_of::<[TCHAR; MAX_PATH as usize]>(), wallpaperFilename.as_ptr());
            if hr < 0 {
                RegCloseKey(regKey);
                return;
            }

            ZeroMemory!(&mut listItem as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<LV_ITEM>());
            listItem.mask = LVIF_TEXT | LVIF_PARAM | LVIF_STATE | LVIF_IMAGE;
            listItem.state = 0;
            listItem.pszText = (*backgroundItem).szDisplayName.as_mut_ptr();
            listItem.iImage = sfi.iIcon;
            listItem.iItem = (*pData).listViewItemCount as i32;
            listItem.lParam = (((*pData).listViewItemCount as isize) as i32) as LPARAM;

            ListView_InsertItem!(hwndBackgroundList, &mut listItem);
            ListView_SetItemState!(
                hwndBackgroundList,
                (*pData).listViewItemCount as i32,
                LVIS_SELECTED,
                LVIS_SELECTED,
            );

            (*pData).listViewItemCount += 1;
        }

        RegCloseKey(regKey);
    }

    if GetWindowsDirectory(szSearchPath.as_mut_ptr(), MAX_PATH) != 0 {
        i = AddWallpapersFromDirectory(i, hwndBackgroundList, backgroundItem, pData, wallpaperFilename.as_ptr(), szSearchPath.as_ptr());
    }

    if SHRegGetPath(HKEY_LOCAL_MACHINE, w!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion"), w!("WallPaperDir"), szSearchPath.as_mut_ptr(), 0) == ERROR_SUCCESS {
        i = AddWallpapersFromDirectory(i, hwndBackgroundList, backgroundItem, pData, wallpaperFilename.as_ptr(), szSearchPath.as_ptr());
    }
}
unsafe extern "C" fn InitBackgroundDialog(hwndDlg: HWND, pData: PBACKGROUND_DATA) {
    let mut szString: [TCHAR; 256] = [0; 256];
    let mut regKey: HKEY = core::ptr::null_mut();
    let mut szBuffer: [TCHAR; 3] = [0; 3];
    let mut bufferSize: u32 = core::mem::size_of_val(&szBuffer) as u32;

    AddListViewItems(hwndDlg, pData);

    LoadString(hApplet, IDS_CENTER, szString.as_mut_ptr(), (core::mem::size_of_val(&szString) / core::mem::size_of::<TCHAR>()) as i32);
    SendDlgItemMessage(hwndDlg, (IDC_PLACEMENT_COMBO) as i32, CB_INSERTSTRING, (PLACEMENT_CENTER) as WPARAM, szString.as_ptr() as LPARAM);

    LoadString(hApplet, IDS_STRETCH, szString.as_mut_ptr(), (core::mem::size_of_val(&szString) / core::mem::size_of::<TCHAR>()) as i32);
    SendDlgItemMessage(hwndDlg, (IDC_PLACEMENT_COMBO) as i32, CB_INSERTSTRING, (PLACEMENT_STRETCH) as WPARAM, szString.as_ptr() as LPARAM);

    LoadString(hApplet, IDS_TILE, szString.as_mut_ptr(), (core::mem::size_of_val(&szString) / core::mem::size_of::<TCHAR>()) as i32);
    SendDlgItemMessage(hwndDlg, (IDC_PLACEMENT_COMBO) as i32, CB_INSERTSTRING, (PLACEMENT_TILE) as WPARAM, szString.as_ptr() as LPARAM);

    LoadString(hApplet, IDS_FIT, szString.as_mut_ptr(), (core::mem::size_of_val(&szString) / core::mem::size_of::<TCHAR>()) as i32);
    SendDlgItemMessage(hwndDlg, (IDC_PLACEMENT_COMBO) as i32, CB_INSERTSTRING, (PLACEMENT_FIT) as WPARAM, szString.as_ptr() as LPARAM);

    LoadString(hApplet, IDS_FILL, szString.as_mut_ptr(), (core::mem::size_of_val(&szString) / core::mem::size_of::<TCHAR>()) as i32);
    SendDlgItemMessage(hwndDlg, (IDC_PLACEMENT_COMBO) as i32, CB_INSERTSTRING, (PLACEMENT_FILL) as WPARAM, szString.as_ptr() as LPARAM);

    SendDlgItemMessage(hwndDlg, (IDC_PLACEMENT_COMBO) as i32, CB_SETCURSEL, (PLACEMENT_CENTER) as WPARAM, 0);
    (*pData).placementSelection = (PLACEMENT_CENTER) as i32;

    if RegOpenKeyEx(HKEY_CURRENT_USER, w!("Control Panel\\Desktop"), 0, KEY_QUERY_VALUE, core::ptr::addr_of_mut!(regKey)) != (ERROR_SUCCESS) as i32 {
        return;
    }

    if RegQueryValueEx(regKey, w!("WallpaperStyle"), (0) as *mut u32, core::ptr::null_mut(), szBuffer.as_mut_ptr() as LPBYTE, core::ptr::addr_of_mut!(bufferSize)) == (ERROR_SUCCESS) as i32 {
        if _ttoi(szBuffer.as_ptr()) == (PLACEMENT_VALUE_CENTER) as i32 {
            SendDlgItemMessage(hwndDlg, (IDC_PLACEMENT_COMBO) as i32, CB_SETCURSEL, (PLACEMENT_CENTER) as WPARAM, 0);
            (*pData).placementSelection = (PLACEMENT_CENTER) as i32;
        }

        if _ttoi(szBuffer.as_ptr()) == (PLACEMENT_VALUE_STRETCH) as i32 {
            SendDlgItemMessage(hwndDlg, (IDC_PLACEMENT_COMBO) as i32, CB_SETCURSEL, (PLACEMENT_STRETCH) as WPARAM, 0);
            (*pData).placementSelection = (PLACEMENT_STRETCH) as i32;
        }

        if _ttoi(szBuffer.as_ptr()) == (PLACEMENT_VALUE_FIT) as i32 {
            SendDlgItemMessage(hwndDlg, (IDC_PLACEMENT_COMBO) as i32, CB_SETCURSEL, (PLACEMENT_FIT) as WPARAM, 0);
            (*pData).placementSelection = (PLACEMENT_FIT) as i32;
        }

        if _ttoi(szBuffer.as_ptr()) == (PLACEMENT_VALUE_FILL) as i32 {
            SendDlgItemMessage(hwndDlg, (IDC_PLACEMENT_COMBO) as i32, CB_SETCURSEL, (PLACEMENT_FILL) as WPARAM, 0);
            (*pData).placementSelection = (PLACEMENT_FILL) as i32;
        }
    }

    if RegQueryValueEx(regKey, w!("TileWallpaper"), (0) as *mut u32, core::ptr::null_mut(), szBuffer.as_mut_ptr() as LPBYTE, core::ptr::addr_of_mut!(bufferSize)) == (ERROR_SUCCESS) as i32 {
        if _ttoi(szBuffer.as_ptr()) == 1 {
            SendDlgItemMessage(hwndDlg, (IDC_PLACEMENT_COMBO) as i32, CB_SETCURSEL, (PLACEMENT_TILE) as WPARAM, 0);
            (*pData).placementSelection = (PLACEMENT_TILE) as i32;
        }
    }

    RegCloseKey(regKey);
}
unsafe extern "C" fn OnColorButton(hwndDlg: HWND, pData: *mut BACKGROUND_DATA) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut res: i32 = (ERROR_SUCCESS) as i32;
    let mut cc: CHOOSECOLOR = core::mem::zeroed();

    res = RegCreateKeyEx(
        HKEY_CURRENT_USER,
        w!("Control Panel\\Appearance"),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_QUERY_VALUE,
        core::ptr::null_mut(),
        &mut hKey,
        core::ptr::null_mut()
    );

    if res == (ERROR_SUCCESS) as i32 {
        let mut dwType: u32 = REG_BINARY;
        let mut cbData: u32 = core::mem::size_of_val(&(*pData).custom_colors) as u32;
        res = RegQueryValueEx(
            hKey,
            w!("CustomColors"),
            core::ptr::null_mut(),
            &mut dwType,
            (*pData).custom_colors.as_mut_ptr() as LPBYTE,
            &mut cbData
        );
        RegCloseKey(hKey);
        hKey = core::ptr::null_mut();
    }

    cc.lStructSize = (core::mem::size_of::<CHOOSECOLOR>()) as u32;
    cc.hwndOwner = hwndDlg;
    cc.hInstance = core::ptr::null_mut();
    cc.rgbResult = g_GlobalData.desktop_color;
    cc.lpCustColors = (*pData).custom_colors.as_mut_ptr();
    cc.Flags = CC_ANYCOLOR | CC_FULLOPEN | CC_RGBINIT;
    cc.lCustData = 0;
    cc.lpfnHook = None;
    cc.lpTemplateName = core::ptr::null_mut();

    if ChooseColor(&mut cc) != 0 {
        g_GlobalData.desktop_color = cc.rgbResult;
        (*pData).bClrBackgroundChanged = (TRUE) as i32;

        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);

        InvalidateRect(
            GetDlgItem(hwndDlg, (IDC_BACKGROUND_PREVIEW) as i32),
            core::ptr::null_mut(),
            (TRUE) as BOOL
        );
        InvalidateRect(
            GetDlgItem(hwndDlg, (IDC_COLOR_BUTTON) as i32),
            core::ptr::null_mut(),
            (TRUE) as BOOL
        );

        res = RegOpenKeyEx(
            HKEY_CURRENT_USER,
            w!("Control Panel\\Appearance"),
            0,
            KEY_SET_VALUE,
            &mut hKey
        );

        if res == (ERROR_SUCCESS) as i32 {
            RegSetValueEx(
                hKey,
                w!("CustomColors"),
                0,
                REG_BINARY,
                (*pData).custom_colors.as_mut_ptr() as LPBYTE,
                core::mem::size_of_val(&(*pData).custom_colors) as u32
            );
            RegCloseKey(hKey);
            hKey = core::ptr::null_mut();
        }
    }
}
unsafe extern "C" fn CheckListViewFilenameExists(hwndList: HWND, tszFileName: LPCTSTR) -> BOOL {
    let mut lvfi: LVFINDINFO = core::mem::zeroed();
    let retVal: i32;

    lvfi.flags = LVFI_STRING;
    lvfi.psz = tszFileName;

    retVal = ListView_FindItem!(hwndList, -1, &lvfi);
    if retVal != -1 {
        return (TRUE) as i32;
    }

    return (FALSE) as i32;
}
unsafe extern "C" fn OnBrowseButton(hwndDlg: HWND, pData: PBACKGROUND_DATA) {
    let mut ofn: OPENFILENAME = core::mem::zeroed();
    let mut filename: [TCHAR; (MAX_PATH) as usize] = core::mem::zeroed();
    let mut fileTitle: [TCHAR; 256] = core::mem::zeroed();
    let mut initialDir: [TCHAR; (MAX_PATH) as usize] = core::mem::zeroed();
    let mut filter: LPTSTR;
    let mut extensions: LPTSTR;
    let mut backgroundItem: *mut BackgroundItem = core::ptr::null_mut();
    let mut sfi: SHFILEINFO = core::mem::zeroed();
    let mut listItem: LV_ITEM = core::mem::zeroed();
    let hwndBackgroundList: HWND;
    let mut p: *mut TCHAR;
    let hr: HRESULT;
    let mut filterdesc: [TCHAR; (MAX_PATH) as usize] = core::mem::zeroed();
    let mut c: *mut TCHAR;
    let mut sizeRemain: usize;
    let buffersize: SIZE_T;
    let success: BOOL;
    let himl: HIMAGELIST;

    hwndBackgroundList = GetDlgItem(hwndDlg, (IDC_BACKGROUND_LIST) as i32);
    himl = ListView_GetImageList!(hwndBackgroundList, LVSIL_SMALL);
    SHGetFolderPathW(core::ptr::null_mut(), (CSIDL_MYPICTURES) as i32, core::ptr::null_mut(), 0, initialDir.as_mut_ptr());

    ZeroMemory!(core::ptr::addr_of_mut!(ofn) as *mut core::ffi::c_void, core::mem::size_of::<OPENFILENAME>());

    ofn.lStructSize = core::mem::size_of::<OPENFILENAME>() as u32;
    ofn.hwndOwner = hwndDlg;
    ofn.lpstrFile = filename.as_mut_ptr();

    LoadString(hApplet, IDS_BACKGROUND_COMDLG_FILTER, filterdesc.as_mut_ptr(), (core::mem::size_of_val(&filterdesc) / core::mem::size_of::<TCHAR>()) as i32);

    extensions = GdipGetSupportedFileExtensions();
    if extensions.is_null() {
        return;
    }

    buffersize = (((_tcslen(extensions) * 2 + 6) * core::mem::size_of::<TCHAR>() + core::mem::size_of_val(&filterdesc)) as u32) as WPARAM;

    filter = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, buffersize) as LPTSTR;
    if filter.is_null() {
        HeapFree(GetProcessHeap(), 0, extensions as *mut core::ffi::c_void);
        return;
    }

    sizeRemain = (buffersize) as usize;
    c = filter;

    if StringCbPrintfEx!(c, sizeRemain, core::ptr::addr_of_mut!(c), core::ptr::addr_of_mut!(sizeRemain), 0, w!("%ls (%ls)"), filterdesc.as_ptr(), extensions) < 0 {
        HeapFree(GetProcessHeap(), 0, extensions as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, filter as *mut core::ffi::c_void);
        return;
    }

    c = c.offset(1);
    sizeRemain -= core::mem::size_of::<TCHAR>();

    if StringCbPrintfEx!(c, sizeRemain, core::ptr::addr_of_mut!(c), core::ptr::addr_of_mut!(sizeRemain), 0, w!("%ls"), extensions) < 0 {
        HeapFree(GetProcessHeap(), 0, extensions as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, filter as *mut core::ffi::c_void);
        return;
    }

    HeapFree(GetProcessHeap(), 0, extensions as *mut core::ffi::c_void);

    ofn.lpstrFile.offset(0 as isize).write(('\0' as WCHAR));
    ofn.nMaxFile = MAX_PATH;
    ofn.lpstrFilter = filter;
    ofn.nFilterIndex = 0;
    ofn.lpstrFileTitle = fileTitle.as_mut_ptr();
    ofn.nMaxFileTitle = 256;
    ofn.lpstrInitialDir = initialDir.as_mut_ptr();
    ofn.Flags = OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_HIDEREADONLY | OFN_EXPLORER;

    success = GetOpenFileName(core::ptr::addr_of_mut!(ofn));
    HeapFree(GetProcessHeap(), 0, filter as *mut core::ffi::c_void);

    if success != 0 {
        if CheckListViewFilenameExists(hwndBackgroundList, ofn.lpstrFileTitle) != (FALSE) as i32 {
            return;
        }

        if (*pData).listViewItemCount > ((MAX_BACKGROUNDS - 1)) as i32 {
            return;
        }

        SHGetFileInfo(
            filename.as_mut_ptr(),
            0,
            (core::ptr::addr_of_mut!(sfi) as *mut core::ffi::c_void).cast(),
            (core::mem::size_of_val(&sfi)) as u32,
            SHGFI_ICON | SHGFI_SMALLICON | SHGFI_DISPLAYNAME
        );
        sfi.iIcon = ImageList_AddIcon!(himl, sfi.hIcon);

        backgroundItem = core::ptr::addr_of_mut!((*pData).backgroundItems[(*pData).listViewItemCount as usize]);

        (*backgroundItem).bWallpaper = (TRUE) as i32;

        let hr = StringCbCopy(
            (*backgroundItem).szDisplayName.as_mut_ptr(),
            core::mem::size_of::<[TCHAR; 256]>(),
            core::ptr::addr_of!(sfi.szDisplayName).cast()
        );
        if hr < 0 {
            return;
        }
        p = _tcsrchr((*backgroundItem).szDisplayName.as_mut_ptr(), ('.' as WCHAR));
        if !p.is_null() {
            *p = 0 as TCHAR;
        }
        let hr = StringCbCopy(
            (*backgroundItem).szFilename.as_mut_ptr(),
            core::mem::size_of::<[TCHAR; MAX_PATH as usize]>(),
            filename.as_ptr()
        );
        if hr < 0 {
            return;
        }

        ZeroMemory!(core::ptr::addr_of_mut!(listItem) as *mut core::ffi::c_void, core::mem::size_of::<LV_ITEM>());
        listItem.mask = LVIF_TEXT | LVIF_PARAM | LVIF_STATE | LVIF_IMAGE;
        listItem.state = 0;
        listItem.pszText = (*backgroundItem).szDisplayName.as_mut_ptr();
        listItem.iImage = sfi.iIcon;
        listItem.iItem = (*pData).listViewItemCount;
        listItem.lParam = (*pData).listViewItemCount as LPARAM;

        ListView_InsertItem!(hwndBackgroundList, core::ptr::addr_of_mut!(listItem));
        ListView_SetItemState!(
            hwndBackgroundList,
            (*pData).listViewItemCount,
            LVIS_SELECTED,
            LVIS_SELECTED
        );
        SendMessage(hwndBackgroundList, WM_VSCROLL, (SB_BOTTOM) as WPARAM, 0);

        (*pData).listViewItemCount += 1;
    }
}
unsafe extern "C" fn ListViewItemChanged(hwndDlg: HWND, pData: *mut BACKGROUND_DATA, itemIndex: i32) {
    let mut backgroundItem: *mut BackgroundItem = core::ptr::null_mut();

    (*pData).backgroundSelection = itemIndex;
    backgroundItem = &mut (*pData).backgroundItems[(*pData).backgroundSelection as usize];

    if (*pData).pWallpaperBitmap != core::ptr::null_mut() {
        DibFreeImage((*pData).pWallpaperBitmap);
        (*pData).pWallpaperBitmap = core::ptr::null_mut();
    }

    if (*backgroundItem).bWallpaper != (FALSE) as i32 {
        (*pData).pWallpaperBitmap = DibLoadImage(((*backgroundItem).szFilename.as_ptr()) as *mut u16);

        if (*pData).pWallpaperBitmap == core::ptr::null_mut() {
            return;
        }
    }

    (*pData).bWallpaperChanged = (TRUE) as i32;

    InvalidateRect(GetDlgItem(hwndDlg, (IDC_BACKGROUND_PREVIEW) as i32),
                   core::ptr::null_mut(), (TRUE) as BOOL);

    EnableWindow(GetDlgItem(hwndDlg, (IDC_PLACEMENT_COMBO) as i32),
                 (*backgroundItem).bWallpaper);

    PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
}
unsafe extern "C" fn DrawBackgroundPreview(draw: LPDRAWITEMSTRUCT, pData: *mut BACKGROUND_DATA) {
    let mut scaleX: f32;
    let mut scaleY: f32;
    let mut scaledWidth: i32;
    let mut scaledHeight: i32;
    let mut posX: i32;
    let mut desX: i32;
    let mut posY: i32;
    let mut desY: i32;
    let mut fitFillScaleNum: i32;
    let mut fitFillScaleDen: i32;
    let mut fitFillWidth: i32;
    let mut fitFillHeight: i32;
    let mut hBrush: HBRUSH;
    let mut x: i32;
    let mut y: i32;
    let hDC: HDC;
    let hOldObj: HGDIOBJ;
    let mut rcItem: RECT = RECT {
        left: (MONITOR_LEFT) as i32,
        top: (MONITOR_TOP) as i32,
        right: (MONITOR_RIGHT) as i32,
        bottom: (MONITOR_BOTTOM) as i32,
    };

    hDC = CreateCompatibleDC((*draw).hDC);
    hOldObj = SelectObject(hDC, (g_GlobalData.hMonitorBitmap).cast());

    if (*pData).backgroundItems[(*pData).backgroundSelection as usize].bWallpaper == (FALSE) as i32 {
        hBrush = CreateSolidBrush(g_GlobalData.desktop_color);
        FillRect(hDC, &mut rcItem, hBrush);
        DeleteObject((hBrush).cast());
    } else if (*pData).pWallpaperBitmap != core::ptr::null_mut() {
        scaleX = (GetSystemMetrics((SM_CXSCREEN) as i32) as f32 - 1.0) / MONITOR_WIDTH as f32;
        scaleY = (GetSystemMetrics((SM_CYSCREEN) as i32) as f32 - 1.0) / MONITOR_HEIGHT as f32;

        scaledWidth = ((*(*pData).pWallpaperBitmap).width as f32 / scaleX) as i32;
        scaledHeight = ((*(*pData).pWallpaperBitmap).height as f32 / scaleY) as i32;

        FillRect(hDC, &mut rcItem, GetSysColorBrush((COLOR_BACKGROUND) as i32));

        SetStretchBltMode(hDC, (COLORONCOLOR) as i32);

        match ((*pData).placementSelection) as u32 {
            PLACEMENT_CENTER => {
                posX = ((MONITOR_WIDTH - (scaledWidth) as u32 + 1) / 2) as i32;
                posY = ((MONITOR_HEIGHT - (scaledHeight) as u32 + 1) / 2) as i32;
                desX = 0;
                desY = 0;

                if posX < 0 {
                    desX = -posX / 2;
                    posX = 0;
                }
                if posY < 0 {
                    desY = -posY / 2;
                    posY = 0;
                }

                if scaledWidth > (MONITOR_WIDTH) as i32 {
                    scaledWidth = (MONITOR_WIDTH) as i32;
                }

                if scaledHeight > (MONITOR_HEIGHT) as i32 {
                    scaledHeight = (MONITOR_HEIGHT) as i32;
                }

                StretchDIBits(
                    hDC,
                    (MONITOR_LEFT + (posX) as u32) as i32,
                    (MONITOR_TOP + (posY) as u32) as i32,
                    scaledWidth,
                    scaledHeight,
                    desX,
                    desY,
                    ((*(*pData).pWallpaperBitmap).width - ((desX as f32 * scaleX) as i32) as u32) as i32,
                    ((*(*pData).pWallpaperBitmap).height - ((desY as f32 * scaleY) as i32) as u32) as i32,
                    ((*(*pData).pWallpaperBitmap).bits) as *const c_void,
                    (*(*pData).pWallpaperBitmap).info,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
            }
            PLACEMENT_STRETCH => {
                StretchDIBits(
                    hDC,
                    (MONITOR_LEFT) as i32,
                    (MONITOR_TOP) as i32,
                    (MONITOR_WIDTH) as i32,
                    (MONITOR_HEIGHT) as i32,
                    0,
                    0,
                    ((*(*pData).pWallpaperBitmap).width) as i32,
                    ((*(*pData).pWallpaperBitmap).height) as i32,
                    ((*(*pData).pWallpaperBitmap).bits) as *const c_void,
                    (*(*pData).pWallpaperBitmap).info,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
            }
            PLACEMENT_TILE => {
                y = 0;
                while y < (MONITOR_HEIGHT) as i32 {
                    x = 0;
                    while x < (MONITOR_WIDTH) as i32 {
                        if (MONITOR_WIDTH - (x) as u32) >= (scaledWidth) as u32 {
                            posX = scaledWidth;
                        } else {
                            posX = (MONITOR_WIDTH - (x) as u32) as i32;
                        }

                        if (MONITOR_HEIGHT - (y) as u32) >= (scaledHeight) as u32 {
                            posY = scaledHeight;
                        } else {
                            posY = (MONITOR_HEIGHT - (y) as u32) as i32;
                        }

                        StretchDIBits(
                            hDC,
                            (MONITOR_LEFT + (x) as u32) as i32,
                            (MONITOR_TOP + (y) as u32) as i32,
                            posX,
                            posY,
                            0,
                            0,
                            ((*(*pData).pWallpaperBitmap).width * (posX) as u32 / (scaledWidth) as u32) as i32,
                            ((*(*pData).pWallpaperBitmap).height * (posY) as u32 / (scaledHeight) as u32) as i32,
                            ((*(*pData).pWallpaperBitmap).bits) as *const c_void,
                            (*(*pData).pWallpaperBitmap).info,
                            DIB_RGB_COLORS,
                            SRCCOPY,
                        );

                        x += scaledWidth;
                    }
                    y += scaledHeight;
                }
            }
            PLACEMENT_FIT => {
                if (MONITOR_WIDTH * (scaledHeight) as u32) <= (MONITOR_HEIGHT * (scaledWidth) as u32) {
                    fitFillScaleNum = (MONITOR_WIDTH) as i32;
                    fitFillScaleDen = scaledWidth;
                } else {
                    fitFillScaleNum = (MONITOR_HEIGHT) as i32;
                    fitFillScaleDen = scaledHeight;
                }

                fitFillWidth = MulDiv(scaledWidth, fitFillScaleNum, fitFillScaleDen);
                fitFillHeight = MulDiv(scaledHeight, fitFillScaleNum, fitFillScaleDen);

                posX = ((MONITOR_WIDTH - (fitFillWidth) as u32) / 2) as i32;
                posY = ((MONITOR_HEIGHT - (fitFillHeight) as u32) / 2) as i32;

                StretchDIBits(
                    hDC,
                    (MONITOR_LEFT + (posX) as u32) as i32,
                    (MONITOR_TOP + (posY) as u32) as i32,
                    fitFillWidth,
                    fitFillHeight,
                    0,
                    0,
                    ((*(*pData).pWallpaperBitmap).width) as i32,
                    ((*(*pData).pWallpaperBitmap).height) as i32,
                    ((*(*pData).pWallpaperBitmap).bits) as *const c_void,
                    (*(*pData).pWallpaperBitmap).info,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
            }
            PLACEMENT_FILL => {
                if (MONITOR_WIDTH * (scaledHeight) as u32) > (MONITOR_HEIGHT * (scaledWidth) as u32) {
                    fitFillScaleNum = (MONITOR_WIDTH) as i32;
                    fitFillScaleDen = scaledWidth;
                } else {
                    fitFillScaleNum = (MONITOR_HEIGHT) as i32;
                    fitFillScaleDen = scaledHeight;
                }

                fitFillWidth = MulDiv(scaledWidth, fitFillScaleNum, fitFillScaleDen);
                fitFillHeight = MulDiv(scaledHeight, fitFillScaleNum, fitFillScaleDen);

                desX = (((fitFillWidth - (MONITOR_WIDTH) as i32) * ((*(*pData).pWallpaperBitmap).width) as i32) / (2 * fitFillWidth));
                desY = (((fitFillHeight - (MONITOR_HEIGHT) as i32) * ((*(*pData).pWallpaperBitmap).height) as i32) / (2 * fitFillHeight));

                StretchDIBits(
                    hDC,
                    (MONITOR_LEFT) as i32,
                    (MONITOR_TOP) as i32,
                    (MONITOR_WIDTH) as i32,
                    (MONITOR_HEIGHT) as i32,
                    desX,
                    desY,
                    ((MONITOR_WIDTH * (*(*pData).pWallpaperBitmap).width) / (fitFillWidth) as u32) as i32,
                    ((MONITOR_HEIGHT * (*(*pData).pWallpaperBitmap).height) / (fitFillHeight) as u32) as i32,
                    ((*(*pData).pWallpaperBitmap).bits) as *const c_void,
                    (*(*pData).pWallpaperBitmap).info,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
            }
            _ => {}
        }
    }

    GdiTransparentBlt(
        (*draw).hDC,
        (*draw).rcItem.left,
        (*draw).rcItem.top,
        (*draw).rcItem.right - (*draw).rcItem.left + 1,
        (*draw).rcItem.bottom - (*draw).rcItem.top + 1,
        hDC,
        0,
        0,
        g_GlobalData.bmMonWidth,
        g_GlobalData.bmMonHeight,
        MONITOR_ALPHA,
    );

    SelectObject(hDC, hOldObj);
    DeleteDC(hDC);
}
unsafe extern "C" fn SetWallpaper(pData: *mut BACKGROUND_DATA) {
    let mut regKey: HKEY = core::ptr::null_mut();
    let mut szWallpaper: [TCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut image: *mut GpImage = core::ptr::null_mut();
    let mut encoderClsid: CLSID = core::mem::zeroed();
    let mut guidFormat: GUID = core::mem::zeroed();
    let mut length: usize = 0;
    let mut status: GpStatus;

    if SHGetFolderPath(
        core::ptr::null_mut(),
        (CSIDL_LOCAL_APPDATA | CSIDL_FLAG_CREATE) as i32,
        core::ptr::null_mut(),
        0,
        szWallpaper.as_mut_ptr(),
    ) < 0
    {
        return;
    }

    if StringCbCat(
        szWallpaper.as_mut_ptr(),
        core::mem::size_of_val(&szWallpaper),
        w!("\\Wallpaper1.bmp"),
    ) < 0
    {
        return;
    }

    if RegCreateKeyEx(
        HKEY_CURRENT_USER,
        w!("Control Panel\\Desktop"),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_SET_VALUE,
        core::ptr::null_mut(),
        &mut regKey,
        core::ptr::null_mut(),
    ) != (ERROR_SUCCESS) as i32
    {
        return;
    }

    if (*pData).placementSelection == (PLACEMENT_TILE) as i32 {
        RegSetValueEx(
            regKey,
            w!("TileWallpaper"),
            0,
            REG_SZ,
            w!("1") as LPBYTE,
            (core::mem::size_of::<TCHAR>() * 2) as u32,
        );
        RegSetValueEx(
            regKey,
            w!("WallpaperStyle"),
            0,
            REG_SZ,
            w!("0") as LPBYTE,
            (core::mem::size_of::<TCHAR>() * 2) as u32,
        );
    }

    if (*pData).placementSelection == (PLACEMENT_CENTER) as i32 {
        RegSetValueEx(
            regKey,
            w!("TileWallpaper"),
            0,
            REG_SZ,
            w!("0") as LPBYTE,
            (core::mem::size_of::<TCHAR>() * 2) as u32,
        );
        RegSetValueEx(
            regKey,
            w!("WallpaperStyle"),
            0,
            REG_SZ,
            w!("0") as LPBYTE,
            (core::mem::size_of::<TCHAR>() * 2) as u32,
        );
    }

    if (*pData).placementSelection == (PLACEMENT_STRETCH) as i32 {
        RegSetValueEx(
            regKey,
            w!("TileWallpaper"),
            0,
            REG_SZ,
            w!("0") as LPBYTE,
            (core::mem::size_of::<TCHAR>() * 2) as u32,
        );
        RegSetValueEx(
            regKey,
            w!("WallpaperStyle"),
            0,
            REG_SZ,
            w!("2") as LPBYTE,
            (core::mem::size_of::<TCHAR>() * 2) as u32,
        );
    }

    if (*pData).placementSelection == (PLACEMENT_FIT) as i32 {
        RegSetValueEx(
            regKey,
            w!("TileWallpaper"),
            0,
            REG_SZ,
            w!("0") as LPBYTE,
            (core::mem::size_of::<TCHAR>() * 2) as u32,
        );
        RegSetValueEx(
            regKey,
            w!("WallpaperStyle"),
            0,
            REG_SZ,
            w!("6") as LPBYTE,
            (core::mem::size_of::<TCHAR>() * 2) as u32,
        );
    }

    if (*pData).placementSelection == (PLACEMENT_FILL) as i32 {
        RegSetValueEx(
            regKey,
            w!("TileWallpaper"),
            0,
            REG_SZ,
            w!("0") as LPBYTE,
            (core::mem::size_of::<TCHAR>() * 2) as u32,
        );
        RegSetValueEx(
            regKey,
            w!("WallpaperStyle"),
            0,
            REG_SZ,
            w!("10") as LPBYTE,
            (core::mem::size_of::<TCHAR>() * 3) as u32,
        );
    }

    if (*pData).backgroundItems[((*pData).backgroundSelection) as usize].bWallpaper != FALSE {
        GdipLoadImageFromFile(
            (*pData).backgroundItems[((*pData).backgroundSelection) as usize].szFilename.as_ptr(),
            &mut image,
        );
        if image.is_null() {
            RegCloseKey(regKey);
            return;
        }

        GdipGetImageRawFormat(image, &mut guidFormat);
        if IsEqualGUID!(&guidFormat, &ImageFormatBMP) != 0 {
            GdipDisposeImage(image);
            RegCloseKey(regKey);
            SystemParametersInfo(
                SPI_SETDESKWALLPAPER,
                0,
                (*pData).backgroundItems[((*pData).backgroundSelection) as usize].szFilename.as_ptr() as *mut _,
                SPIF_UPDATEINIFILE,
            );
            return;
        }

        if GdipGetEncoderClsid(w!("image/bmp") as PCWSTR, &mut encoderClsid) < 0 {
            GdipDisposeImage(image);
            RegCloseKey(regKey);
            return;
        }

        status = GdipSaveImageToFile(image, szWallpaper.as_ptr(), &encoderClsid, core::ptr::null());

        GdipDisposeImage(image);

        if status != Ok {
            RegCloseKey(regKey);
            return;
        }

        if StringCchLength(
            (*pData).backgroundItems[((*pData).backgroundSelection) as usize].szFilename.as_ptr(),
            (MAX_PATH) as usize,
            &mut length,
        ) >= 0
        {
            RegSetValueEx(
                regKey,
                w!("ConvertedWallpaper"),
                0,
                REG_SZ,
                (*pData).backgroundItems[((*pData).backgroundSelection) as usize].szFilename.as_ptr() as LPBYTE,
                ((length + 1) * core::mem::size_of::<TCHAR>()) as u32,
            );
        }

        if StringCchLength(szWallpaper.as_ptr(), (MAX_PATH) as usize, &mut length) >= 0 {
            RegSetValueEx(
                regKey,
                w!("OriginalWallpaper"),
                0,
                REG_SZ,
                szWallpaper.as_ptr() as LPBYTE,
                ((length + 1) * core::mem::size_of::<TCHAR>()) as u32,
            );
        }

        SystemParametersInfo(
            SPI_SETDESKWALLPAPER,
            0,
            szWallpaper.as_ptr() as *mut _,
            SPIF_UPDATEINIFILE,
        );
    } else {
        SystemParametersInfo(
            SPI_SETDESKWALLPAPER,
            0,
            w!("") as *mut core::ffi::c_void,
            SPIF_UPDATEINIFILE,
        );
    }

    RegCloseKey(regKey);
}
unsafe extern "C" fn SetDesktopBackColor(hwndDlg: HWND, pData: *mut BACKGROUND_DATA) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut iElement: i32 = (COLOR_BACKGROUND) as i32;
    let mut clText: [TCHAR; 16] = [0; 16];
    let mut red: u8;
    let mut green: u8;
    let mut blue: u8;

    if SetSysColors(1, core::ptr::addr_of_mut!(iElement), core::ptr::addr_of_mut!(g_GlobalData.desktop_color)) == 0 {
        MessageBox(hwndDlg, w!("SetSysColor() failed!"),
            w!("Error!"), MB_ICONSTOP);
    }

    if RegCreateKeyEx(HKEY_CURRENT_USER, w!("Control Panel\\Colors"), 0, core::ptr::null_mut(),
                       REG_OPTION_NON_VOLATILE, KEY_SET_VALUE, core::ptr::null_mut(), core::ptr::addr_of_mut!(hKey), core::ptr::null_mut()) != (ERROR_SUCCESS) as i32 {
        return;
    }

    red = GetRValue!(g_GlobalData.desktop_color);
    green = GetGValue!(g_GlobalData.desktop_color);
    blue = GetBValue!(g_GlobalData.desktop_color);

    StringCbPrintf!(clText.as_mut_ptr(), core::mem::size_of_val(&clText), w!("%d %d %d"), red as c_int, green as c_int, blue as c_int);
    RegSetValueEx(hKey, w!("Background"), 0, REG_SZ, clText.as_ptr() as LPBYTE,
                  ((wcslen(clText.as_ptr()) + 1) * core::mem::size_of::<TCHAR>()) as u32);

    RegCloseKey(hKey);
}
unsafe extern "C" fn OnCustomButton(hwndDlg: HWND, pData: *mut BACKGROUND_DATA) {
    let mut hpsp: [HPROPSHEETPAGE; 1] = [core::ptr::null_mut(); 1];
    // background.c:1121 `PROPSHEETHEADER psh = {sizeof(psh)};` — a C aggregate initialiser that
    // zeroes everything it does not name, then eleven separate field stores. Rebuilt as a zeroed
    // value plus those stores rather than a Rust struct literal, because the C spells five of the
    // fields (pszIcon, nStartPage, phpage, hbmWatermark, hbmHeader) through prsht.h's ANONYMOUS
    // unions, and a struct literal has no syntax for a path like `__bindgen_anon_1.pszIcon`.
    let mut psh: PROPSHEETHEADER = core::mem::zeroed();
    psh.dwSize = core::mem::size_of::<PROPSHEETHEADER>() as u32;
    psh.dwFlags = PSH_NOAPPLYNOW;
    psh.hwndParent = GetParent(hwndDlg);
    psh.hInstance = hApplet;
    psh.pszCaption = MAKEINTRESOURCE!(IDS_DESKTOP_ITEMS);
    psh.__bindgen_anon_3.phpage = hpsp.as_mut_ptr();

    let mut psp: PROPSHEETPAGE = core::mem::zeroed();
    psp.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as u32;
    psp.dwFlags = PSP_DEFAULT;
    psp.hInstance = hApplet;
    psp.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(IDD_DESKTOP_GENERAL);
    psp.pfnDlgProc = Some(DesktopPageProc);
    psp.lParam = (&mut (*pData).desktopData) as *mut _ as LPARAM;

    hpsp[0] = CreatePropertySheetPage(&mut psp);
    if hpsp[0].is_null() {
        return;
    }

    psh.nPages += 1;

    if PropertySheet(&mut psh) > 0 {
        if SaveDesktopSettings(&mut (*pData).desktopData) != 0 {
            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn BackgroundPageProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pData: *mut BACKGROUND_DATA;
    let mut gdipStartup: GdiplusStartupInput = core::mem::zeroed();

    pData = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut BACKGROUND_DATA;

    match uMsg {
        WM_INITDIALOG => {
            pData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<BACKGROUND_DATA>()) as SIZE_T,
            ) as *mut BACKGROUND_DATA;
            SetWindowLongPtr(hwndDlg, DWLP_USER, ((pData as isize) as i32) as LPARAM);
            gdipStartup.GdiplusVersion = 1;
            gdipStartup.DebugEventCallback = None;
            gdipStartup.SuppressBackgroundThread = (FALSE) as i32;
            gdipStartup.SuppressExternalCodecs = (FALSE) as i32;
            GdiplusStartup(
                core::ptr::addr_of_mut!((*pData).gdipToken),
                core::ptr::addr_of_mut!(gdipStartup),
                core::ptr::null_mut(),
            );
            InitBackgroundDialog(hwndDlg, pData);
            InitDesktopSettings(core::ptr::addr_of_mut!((*pData).desktopData));
        }

        WM_COMMAND => {
            let controlId = LOWORD!(wParam);
            let command = HIWORD!(wParam);

            match (controlId) as u32 {
                IDC_COLOR_BUTTON => {
                    if command == (BN_CLICKED) as u16 {
                        OnColorButton(hwndDlg, pData);
                    }
                }

                IDC_BROWSE_BUTTON => {
                    if command == (BN_CLICKED) as u16 {
                        OnBrowseButton(hwndDlg, pData);
                    }
                }

                IDC_PLACEMENT_COMBO => {
                    if command == (CBN_SELCHANGE) as u16 {
                        (*pData).placementSelection = SendDlgItemMessage(
                            hwndDlg,
                            (IDC_PLACEMENT_COMBO) as i32,
                            CB_GETCURSEL,
                            0,
                            0,
                        ) as i32;

                        InvalidateRect(
                            GetDlgItem(hwndDlg, (IDC_BACKGROUND_PREVIEW) as i32),
                            core::ptr::null_mut(),
                            (TRUE) as BOOL,
                        );

                        PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                    }
                }

                IDC_DESKTOP_CUSTOM => {
                    if command == (BN_CLICKED) as u16 {
                        OnCustomButton(hwndDlg, pData);
                    }
                }

                _ => {}
            }
        }

        WM_DRAWITEM => {
            let drawItem = lParam as LPDRAWITEMSTRUCT;

            if (*drawItem).CtlID == IDC_BACKGROUND_PREVIEW {
                DrawBackgroundPreview(drawItem, pData);
            }
        }

        WM_NOTIFY => {
            let lpnm = lParam as LPNMHDR;

            match (*lpnm).code {
                PSN_APPLY => {
                    if (*pData).bWallpaperChanged != 0 {
                        SetWallpaper(pData);
                    }
                    if (*pData).bClrBackgroundChanged != 0 {
                        SetDesktopBackColor(hwndDlg, pData);
                    }
                    if (*pData).desktopData.bSettingsChanged != 0 {
                        SetDesktopSettings(core::ptr::addr_of_mut!((*pData).desktopData));
                    }
                    SendMessage(
                        HWND_BROADCAST,
                        WM_SETTINGCHANGE,
                        0,
                        w!("") as LPARAM,
                    );
                    return (TRUE) as LPARAM;
                }

                LVN_ITEMCHANGED => {
                    let nm = lParam as *mut NMLISTVIEW;

                    if ((*nm).uNewState & LVIS_SELECTED) == 0 {
                        return (FALSE as INT_PTR) as LPARAM;
                    }

                    ListViewItemChanged(hwndDlg, pData, (*nm).iItem);
                }

                NM_CUSTOMDRAW => {
                    if (*lpnm).idFrom == (IDC_COLOR_BUTTON) as WPARAM {
                        return SetDlgMsgResult!(
                            hwndDlg,
                            0,
                            ClrBtn_CustomDraw(
                                lParam as *mut NMCUSTOMDRAW,
                                g_GlobalData.desktop_color,
                            ),
                        );
                    }
                }

                _ => {}
            }
        }

        WM_DESTROY => {
            if (*pData).pWallpaperBitmap != core::ptr::null_mut() {
                DibFreeImage((*pData).pWallpaperBitmap);
            }

            GdiplusShutdown((*pData).gdipToken);
            HeapFree(GetProcessHeap(), 0, (pData).cast());
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Display Control Panel
//  * FILE:            dll/cpl/desk/background.c
//  * PURPOSE:         Background property page
//  *
//  * PROGRAMMERS:     Trevor McCort (lycan359@gmail.com)
//  *                  Alexey Minnekhanov (minlexx@rambler.ru)
//  */
//
// #include "desk.h"
//
// #include <shellapi.h>
// #include <shlwapi.h>
// #include <windowsx.h>
//
// #define MAX_BACKGROUNDS     100
//
// typedef enum
// {
//     PLACEMENT_CENTER = 0,
//     PLACEMENT_STRETCH,
//     PLACEMENT_TILE,
//     PLACEMENT_FIT,
//     PLACEMENT_FILL
// } PLACEMENT;
//
// /* The tile placement is stored in different registry
//  * key, but due to a condition in win32k it needs to be
//  * zero when stored in the same key as others.
//  */
// typedef enum
// {
//     PLACEMENT_VALUE_CENTER    = 0,
//     PLACEMENT_VALUE_STRETCH   = 2,
//     PLACEMENT_VALUE_TILE      = 0,
//     PLACEMENT_VALUE_FIT       = 6,
//     PLACEMENT_VALUE_FILL      = 10
// } PLACEMENT_VALUE;
//
// typedef struct
// {
//     BOOL bWallpaper; /* Is this background a wallpaper */
//
//     TCHAR szFilename[MAX_PATH];
//     TCHAR szDisplayName[256];
//
// } BackgroundItem;
//
// typedef struct _BACKGROUND_DATA
// {
//     BOOL bWallpaperChanged;
//     BOOL bClrBackgroundChanged;
//
//     BackgroundItem backgroundItems[MAX_BACKGROUNDS];
//
//     PDIBITMAP pWallpaperBitmap;
//
//     int placementSelection;
//     int backgroundSelection;
//
//     COLORREF custom_colors[16];
//
//     int listViewItemCount;
//
//     ULONG_PTR gdipToken;
//
//     DESKTOP_DATA desktopData;
// } BACKGROUND_DATA, *PBACKGROUND_DATA;
//
// GLOBAL_DATA g_GlobalData;
//
//
// HRESULT
// GdipGetEncoderClsid(PCWSTR MimeType, CLSID *pClsid)
// {
//     UINT num;
//     UINT size;
//     UINT i;
//     ImageCodecInfo *codecInfo;
//
//     if (GdipGetImageEncodersSize(&num, &size) != Ok ||
//         size == 0)
//     {
//         return E_FAIL;
//     }
//
//     codecInfo = HeapAlloc(GetProcessHeap(), 0, size);
//     if (!codecInfo)
//     {
//         return E_OUTOFMEMORY;
//     }
//
//     if (GdipGetImageEncoders(num, size, codecInfo) != Ok)
//     {
//         HeapFree(GetProcessHeap(), 0, codecInfo);
//         return E_FAIL;
//     }
//
//     for (i = 0; i < num; i++)
//     {
//         if (!_wcsicmp(codecInfo[i].MimeType, MimeType))
//         {
//             *pClsid = codecInfo[i].Clsid;
//             HeapFree(GetProcessHeap(), 0, codecInfo);
//             return S_OK;
//         }
//     }
//
//     HeapFree(GetProcessHeap(), 0, codecInfo);
//     return E_FAIL;
// }
//
//
// LPWSTR
// GdipGetSupportedFileExtensions(VOID)
// {
//     ImageCodecInfo *codecInfo;
//     UINT num;
//     UINT size;
//     UINT i;
//     LPWSTR lpBuffer = NULL;
//
//     if (GdipGetImageDecodersSize(&num, &size) != Ok ||
//         size == 0)
//     {
//         return NULL;
//     }
//
//     codecInfo = HeapAlloc(GetProcessHeap(), 0, size);
//     if (!codecInfo)
//     {
//         return NULL;
//     }
//
//     if (GdipGetImageDecoders(num, size, codecInfo) != Ok)
//     {
//         HeapFree(GetProcessHeap(), 0, codecInfo);
//         return NULL;
//     }
//
//     size = 0;
//     for (i = 0; i < num; ++i)
//     {
//         size = size + (UINT)wcslen(codecInfo[i].FilenameExtension) + 1;
//     }
//
//     size = (size + 1) * sizeof(WCHAR);
//
//     lpBuffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, size);
//     if (!lpBuffer)
//     {
//         HeapFree(GetProcessHeap(), 0, codecInfo);
//         return NULL;
//     }
//
//     for (i = 0; i < num; ++i)
//     {
//         if (!lstrcmpiW(codecInfo[i].FilenameExtension, L"*.ico"))
//             continue;
//
//         StringCbCatW(lpBuffer, size, codecInfo[i].FilenameExtension);
//         if (i < (num - 1))
//         {
//             StringCbCatW(lpBuffer, size, L";");
//         }
//     }
//
//     HeapFree(GetProcessHeap(), 0, codecInfo);
//
//     return lpBuffer;
// }
//
//
// static UINT
// AddWallpapersFromDirectory(UINT uCounter, HWND hwndBackgroundList, BackgroundItem *backgroundItem, PBACKGROUND_DATA pData, LPCTSTR wallpaperFilename, LPCTSTR wallpaperDirectory)
// {
//     WIN32_FIND_DATA fd;
//     HANDLE hFind;
//     TCHAR szSearchPath[MAX_PATH];
//     LPTSTR szFileTypes = NULL;
//     TCHAR separators[] = TEXT(";");
//     TCHAR *token;
//     HRESULT hr;
//     SHFILEINFO sfi;
//     UINT i = uCounter;
//     LV_ITEM listItem;
//     HIMAGELIST himl;
//
//     szFileTypes = GdipGetSupportedFileExtensions();
//     if (!szFileTypes)
//     {
//         return i;
//     }
//
//     himl = ListView_GetImageList(hwndBackgroundList, LVSIL_SMALL);
//
//     token = _tcstok(szFileTypes, separators);
//     while (token != NULL)
//     {
//         if (!PathCombine(szSearchPath, wallpaperDirectory, token))
//         {
//             HeapFree(GetProcessHeap(), 0, szFileTypes);
//             return i;
//         }
//
//         hFind = FindFirstFile(szSearchPath, &fd);
//         while (hFind != INVALID_HANDLE_VALUE)
//         {
//             TCHAR filename[MAX_PATH];
//
//             if (!PathCombine(filename, wallpaperDirectory, fd.cFileName))
//             {
//                 FindClose(hFind);
//                 HeapFree(GetProcessHeap(), 0, szFileTypes);
//                 return i;
//             }
//
//             /* Don't add any hidden bitmaps. Also don't add current wallpaper once more. */
//             if (((fd.dwFileAttributes & FILE_ATTRIBUTE_HIDDEN) == 0) && (_tcsicmp(wallpaperFilename, filename) != 0))
//             {
//                 SHGetFileInfo(filename,
//                               0,
//                               &sfi,
//                               sizeof(sfi),
//                               SHGFI_ICON | SHGFI_SMALLICON | SHGFI_DISPLAYNAME);
//                 sfi.iIcon = ImageList_AddIcon(himl, sfi.hIcon);
//                 i++;
//
//                 backgroundItem = &pData->backgroundItems[pData->listViewItemCount];
//
//                 backgroundItem->bWallpaper = TRUE;
//
//                 hr = StringCbCopy(backgroundItem->szDisplayName, sizeof(backgroundItem->szDisplayName), sfi.szDisplayName);
//                 if (FAILED(hr))
//                 {
//                     FindClose(hFind);
//                     HeapFree(GetProcessHeap(), 0, szFileTypes);
//                     return i;
//                 }
//
//                 PathRemoveExtension(backgroundItem->szDisplayName);
//
//                 hr = StringCbCopy(backgroundItem->szFilename, sizeof(backgroundItem->szFilename), filename);
//                 if (FAILED(hr))
//                 {
//                     FindClose(hFind);
//                     HeapFree(GetProcessHeap(), 0, szFileTypes);
//                     return i;
//                 }
//
//                 ZeroMemory(&listItem, sizeof(LV_ITEM));
//                 listItem.mask       = LVIF_TEXT | LVIF_PARAM | LVIF_STATE | LVIF_IMAGE;
//                 listItem.pszText    = backgroundItem->szDisplayName;
//                 listItem.state      = 0;
//                 listItem.iImage     = sfi.iIcon;
//                 listItem.iItem      = pData->listViewItemCount;
//                 listItem.lParam     = pData->listViewItemCount;
//
//                 (void)ListView_InsertItem(hwndBackgroundList, &listItem);
//
//                 pData->listViewItemCount++;
//             }
//
//             if (!FindNextFile(hFind, &fd))
//                 break;
//         }
//
//         token = _tcstok(NULL, separators);
//         FindClose(hFind);
//     }
//
//     HeapFree(GetProcessHeap(), 0, szFileTypes);
//
//     return i;
// }
//
//
// /* Add the images in the C:\ReactOS, the wallpaper directory and the current wallpaper if any */
// static VOID
// AddListViewItems(HWND hwndDlg, PBACKGROUND_DATA pData)
// {
//     TCHAR szSearchPath[MAX_PATH];
//     LV_ITEM listItem;
//     LV_COLUMN dummy;
//     RECT clientRect;
//     HKEY regKey;
//     SHFILEINFO sfi;
//     HIMAGELIST himl;
//     TCHAR wallpaperFilename[MAX_PATH];
//     TCHAR originalWallpaper[MAX_PATH];
//     DWORD bufferSize = sizeof(wallpaperFilename);
//     TCHAR buffer[MAX_PATH];
//     DWORD varType = REG_SZ;
//     LONG result;
//     UINT i = 0;
//     BackgroundItem *backgroundItem = NULL;
//     HWND hwndBackgroundList;
//     HRESULT hr;
//     HICON hIcon;
//     INT cx, cy;
//     HINSTANCE hShell32;
//
//     hwndBackgroundList = GetDlgItem(hwndDlg, IDC_BACKGROUND_LIST);
//
//     GetClientRect(hwndBackgroundList, &clientRect);
//
//     cx = GetSystemMetrics(SM_CXSMICON);
//     cy = GetSystemMetrics(SM_CYSMICON);
//     himl = ImageList_Create(cx, cy, ILC_COLOR32 | ILC_MASK, 0, 0);
//
//     /* Load (None) icon */
// #define IDI_SHELL_NO 200
//     hShell32 = GetModuleHandleW(L"shell32.dll");
//     hIcon = (HICON)LoadImageW(hShell32, MAKEINTRESOURCEW(IDI_SHELL_NO), IMAGE_ICON, cx, cy, 0);
// #undef IDI_SHELL_NO
//
//     ListView_SetImageList(hwndBackgroundList, himl, LVSIL_SMALL);
//
//     /* Add a new column to the list */
//     ZeroMemory(&dummy, sizeof(LV_COLUMN));
//     dummy.mask      = LVCF_SUBITEM | LVCF_WIDTH;
//     dummy.iSubItem  = 0;
//     dummy.cx        = (clientRect.right - clientRect.left) - GetSystemMetrics(SM_CXVSCROLL);
//     (void)ListView_InsertColumn(hwndBackgroundList, 0, &dummy);
//
//     /* Add the "None" item */
//     backgroundItem = &pData->backgroundItems[pData->listViewItemCount];
//     backgroundItem->bWallpaper = FALSE;
//     LoadString(hApplet,
//                IDS_NONE,
//                backgroundItem->szDisplayName,
//                sizeof(backgroundItem->szDisplayName) / sizeof(TCHAR));
//
//     ZeroMemory(&listItem, sizeof(LV_ITEM));
//     listItem.mask       = LVIF_TEXT | LVIF_PARAM | LVIF_STATE | LVIF_IMAGE;
//     listItem.state      = 0;
//     listItem.pszText    = backgroundItem->szDisplayName;
//     listItem.iImage     = ImageList_AddIcon(himl, hIcon);
//     listItem.iItem      = pData->listViewItemCount;
//     listItem.lParam     = pData->listViewItemCount;
//     hIcon = NULL;
//
//     (void)ListView_InsertItem(hwndBackgroundList, &listItem);
//     ListView_SetItemState(hwndBackgroundList,
//                           pData->listViewItemCount,
//                           LVIS_SELECTED,
//                           LVIS_SELECTED);
//
//     pData->listViewItemCount++;
//
//     /* Add current wallpaper if any */
//     result = RegOpenKeyEx(HKEY_CURRENT_USER, TEXT("Control Panel\\Desktop"), 0, KEY_QUERY_VALUE, &regKey);
//     if (result == ERROR_SUCCESS)
//     {
//         result = RegQueryValueEx(regKey, TEXT("Wallpaper"), 0, &varType, (LPBYTE)wallpaperFilename, &bufferSize);
//         if ((result == ERROR_SUCCESS) && (_tcslen(wallpaperFilename) > 0))
//         {
//             bufferSize = sizeof(originalWallpaper);
//             result = RegQueryValueEx(regKey, TEXT("OriginalWallpaper"), 0, &varType, (LPBYTE)originalWallpaper, &bufferSize);
//
//             /* If Wallpaper and OriginalWallpaper are the same, try to retrieve ConvertedWallpaper and use it instead of Wallpaper */
//             if ((result == ERROR_SUCCESS) && (_tcslen(originalWallpaper) > 0) && (_tcsicmp(wallpaperFilename, originalWallpaper) == 0))
//             {
//                 bufferSize = sizeof(originalWallpaper);
//                 result = RegQueryValueEx(regKey, TEXT("ConvertedWallpaper"), 0, &varType, (LPBYTE)originalWallpaper, &bufferSize);
//
//                 if ((result == ERROR_SUCCESS) && (_tcslen(originalWallpaper) > 0))
//                 {
//                     hr = StringCbCopy(wallpaperFilename, sizeof(wallpaperFilename), originalWallpaper);
//                     if (FAILED(hr))
//                     {
//                         RegCloseKey(regKey);
//                         return;
//                     }
//                 }
//             }
//
//             /* Allow environment variables in file name */
//             if (ExpandEnvironmentStrings(wallpaperFilename, buffer, MAX_PATH))
//             {
//                 hr = StringCbCopy(wallpaperFilename, sizeof(wallpaperFilename), buffer);
//                 if (FAILED(hr))
//                 {
//                     RegCloseKey(regKey);
//                     return;
//                 }
//             }
//
//             SHGetFileInfoW(wallpaperFilename,
//                            0,
//                            &sfi,
//                            sizeof(sfi),
//                            SHGFI_ICON | SHGFI_SMALLICON |
//                            SHGFI_DISPLAYNAME);
//             sfi.iIcon = ImageList_AddIcon(himl, sfi.hIcon);
//
//             i++;
//
//             backgroundItem = &pData->backgroundItems[pData->listViewItemCount];
//
//             backgroundItem->bWallpaper = TRUE;
//
//             hr = StringCbCopy(backgroundItem->szDisplayName, sizeof(backgroundItem->szDisplayName), sfi.szDisplayName);
//             if (FAILED(hr))
//             {
//                 RegCloseKey(regKey);
//                 return;
//             }
//
//             PathRemoveExtension(backgroundItem->szDisplayName);
//
//             hr = StringCbCopy(backgroundItem->szFilename, sizeof(backgroundItem->szFilename), wallpaperFilename);
//             if (FAILED(hr))
//             {
//                 RegCloseKey(regKey);
//                 return;
//             }
//
//             ZeroMemory(&listItem, sizeof(LV_ITEM));
//             listItem.mask       = LVIF_TEXT | LVIF_PARAM | LVIF_STATE | LVIF_IMAGE;
//             listItem.state      = 0;
//             listItem.pszText    = backgroundItem->szDisplayName;
//             listItem.iImage     = sfi.iIcon;
//             listItem.iItem      = pData->listViewItemCount;
//             listItem.lParam     = pData->listViewItemCount;
//
//             (void)ListView_InsertItem(hwndBackgroundList, &listItem);
//             ListView_SetItemState(hwndBackgroundList,
//                                   pData->listViewItemCount,
//                                   LVIS_SELECTED,
//                                   LVIS_SELECTED);
//
//             pData->listViewItemCount++;
//         }
//
//         RegCloseKey(regKey);
//     }
//
//     /* Add all the images in the C:\ReactOS directory. */
//     if (GetWindowsDirectory(szSearchPath, MAX_PATH))
//     {
//         i = AddWallpapersFromDirectory(i, hwndBackgroundList, backgroundItem, pData, wallpaperFilename, szSearchPath);
//     }
//
//     /* Add all the images in the wallpaper directory. */
//     if (SHRegGetPath(HKEY_LOCAL_MACHINE, TEXT("SOFTWARE\\Microsoft\\Windows\\CurrentVersion"), TEXT("WallPaperDir"), szSearchPath, 0) == ERROR_SUCCESS)
//     {
//         i = AddWallpapersFromDirectory(i, hwndBackgroundList, backgroundItem, pData, wallpaperFilename, szSearchPath);
//     }
// }
//
//
// static VOID
// InitBackgroundDialog(HWND hwndDlg, PBACKGROUND_DATA pData)
// {
//     TCHAR szString[256];
//     HKEY regKey;
//     TCHAR szBuffer[3];
//     DWORD bufferSize = sizeof(szBuffer);
//
//     AddListViewItems(hwndDlg, pData);
//
//     LoadString(hApplet, IDS_CENTER, szString, sizeof(szString) / sizeof(TCHAR));
//     SendDlgItemMessage(hwndDlg, IDC_PLACEMENT_COMBO, CB_INSERTSTRING, PLACEMENT_CENTER, (LPARAM)szString);
//
//     LoadString(hApplet, IDS_STRETCH, szString, sizeof(szString) / sizeof(TCHAR));
//     SendDlgItemMessage(hwndDlg, IDC_PLACEMENT_COMBO, CB_INSERTSTRING, PLACEMENT_STRETCH, (LPARAM)szString);
//
//     LoadString(hApplet, IDS_TILE, szString, sizeof(szString) / sizeof(TCHAR));
//     SendDlgItemMessage(hwndDlg, IDC_PLACEMENT_COMBO, CB_INSERTSTRING, PLACEMENT_TILE, (LPARAM)szString);
//
//     LoadString(hApplet, IDS_FIT, szString, sizeof(szString) / sizeof(TCHAR));
//     SendDlgItemMessage(hwndDlg, IDC_PLACEMENT_COMBO, CB_INSERTSTRING, PLACEMENT_FIT, (LPARAM)szString);
//
//     LoadString(hApplet, IDS_FILL, szString, sizeof(szString) / sizeof(TCHAR));
//     SendDlgItemMessage(hwndDlg, IDC_PLACEMENT_COMBO, CB_INSERTSTRING, PLACEMENT_FILL, (LPARAM)szString);
//
//     SendDlgItemMessage(hwndDlg, IDC_PLACEMENT_COMBO, CB_SETCURSEL, PLACEMENT_CENTER, 0);
//     pData->placementSelection = PLACEMENT_CENTER;
//
//     /* Load the default settings from the registry */
//     if (RegOpenKeyEx(HKEY_CURRENT_USER, TEXT("Control Panel\\Desktop"), 0, KEY_QUERY_VALUE, &regKey) != ERROR_SUCCESS)
//     {
//         return;
//     }
//
//     if (RegQueryValueEx(regKey, TEXT("WallpaperStyle"), 0, NULL, (LPBYTE)szBuffer, &bufferSize) == ERROR_SUCCESS)
//     {
//         if (_ttoi(szBuffer) == PLACEMENT_VALUE_CENTER)
//         {
//             SendDlgItemMessage(hwndDlg, IDC_PLACEMENT_COMBO, CB_SETCURSEL, PLACEMENT_CENTER, 0);
//             pData->placementSelection = PLACEMENT_CENTER;
//         }
//
//         if (_ttoi(szBuffer) == PLACEMENT_VALUE_STRETCH)
//         {
//             SendDlgItemMessage(hwndDlg, IDC_PLACEMENT_COMBO, CB_SETCURSEL, PLACEMENT_STRETCH, 0);
//             pData->placementSelection = PLACEMENT_STRETCH;
//         }
//
//         if (_ttoi(szBuffer) == PLACEMENT_VALUE_FIT)
//         {
//             SendDlgItemMessage(hwndDlg, IDC_PLACEMENT_COMBO, CB_SETCURSEL, PLACEMENT_FIT, 0);
//             pData->placementSelection = PLACEMENT_FIT;
//         }
//
//         if (_ttoi(szBuffer) == PLACEMENT_VALUE_FILL)
//         {
//             SendDlgItemMessage(hwndDlg, IDC_PLACEMENT_COMBO, CB_SETCURSEL, PLACEMENT_FILL, 0);
//             pData->placementSelection = PLACEMENT_FILL;
//         }
//     }
//
//     if (RegQueryValueEx(regKey, TEXT("TileWallpaper"), 0, NULL, (LPBYTE)szBuffer, &bufferSize) == ERROR_SUCCESS)
//     {
//         if (_ttoi(szBuffer) == 1)
//         {
//             SendDlgItemMessage(hwndDlg, IDC_PLACEMENT_COMBO, CB_SETCURSEL, PLACEMENT_TILE, 0);
//             pData->placementSelection = PLACEMENT_TILE;
//         }
//     }
//
//     RegCloseKey(regKey);
// }
//
//
// static VOID
// OnColorButton(HWND hwndDlg, PBACKGROUND_DATA pData)
// {
//     /* Load custom colors from Registry */
//     HKEY hKey = NULL;
//     LONG res = ERROR_SUCCESS;
//     CHOOSECOLOR cc;
//
//     res = RegCreateKeyEx(HKEY_CURRENT_USER, TEXT("Control Panel\\Appearance"), 0, NULL,
//                          REG_OPTION_NON_VOLATILE, KEY_QUERY_VALUE, NULL, &hKey, NULL);
//     /* Now the key is either created or opened existing, if res == ERROR_SUCCESS */
//     if (res == ERROR_SUCCESS)
//     {
//         /* Key opened */
//         DWORD dwType = REG_BINARY;
//         DWORD cbData = sizeof(pData->custom_colors);
//         res = RegQueryValueEx(hKey, TEXT("CustomColors"), NULL, &dwType,
//                               (LPBYTE)pData->custom_colors, &cbData);
//         RegCloseKey(hKey);
//         hKey = NULL;
//     }
//
//     /* Launch ChooseColor() dialog */
//
//     cc.lStructSize = sizeof(CHOOSECOLOR);
//     cc.hwndOwner = hwndDlg;
//     cc.hInstance = NULL;
//     cc.rgbResult = g_GlobalData.desktop_color;
//     cc.lpCustColors = pData->custom_colors;
//     cc.Flags = CC_ANYCOLOR | /* Causes the dialog box to display all available colors in the set of basic colors.  */
//                CC_FULLOPEN | /* opens dialog in full size */
//                CC_RGBINIT ;  /* init chosen color by rgbResult value */
//     cc.lCustData = 0;
//     cc.lpfnHook = NULL;
//     cc.lpTemplateName = NULL;
//     if (ChooseColor(&cc))
//     {
//         /* Save selected color to var */
//         g_GlobalData.desktop_color = cc.rgbResult;
//         pData->bClrBackgroundChanged = TRUE;
//
//         /* Apply button will be activated */
//         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//
//         /* Window will be updated :) */
//         InvalidateRect(GetDlgItem(hwndDlg, IDC_BACKGROUND_PREVIEW), NULL, TRUE);
//         InvalidateRect(GetDlgItem(hwndDlg, IDC_COLOR_BUTTON), NULL, TRUE);
//
//         /* Save custom colors to reg. To this moment key must be created already. See above */
//         res = RegOpenKeyEx(HKEY_CURRENT_USER, TEXT("Control Panel\\Appearance"), 0,
//                            KEY_SET_VALUE, &hKey);
//         if (res == ERROR_SUCCESS)
//         {
//             /* Key opened */
//             RegSetValueEx(hKey, TEXT("CustomColors"), 0, REG_BINARY,
//                           (LPBYTE)pData->custom_colors, sizeof(pData->custom_colors));
//             RegCloseKey(hKey);
//             hKey = NULL;
//         }
//     }
// }
//
//
// /*
//  * ListView_FindItem() Macro: Searches for a list-view item with the specified
//  * characteristics. Returns the index of the item if successful, or -1 otherwise
//  */
// static BOOL
// CheckListViewFilenameExists(HWND hwndList, LPCTSTR tszFileName)
// {
//     LVFINDINFO lvfi;
//     int retVal;
//
//     lvfi.flags = LVFI_STRING; /* Search item by EXACT string */
//     lvfi.psz   = tszFileName; /* String to search */
//
//     /* Other items of this structure are not valid, besacuse flags are not set. */
//     retVal = ListView_FindItem(hwndList, -1, &lvfi);
//     if (retVal != -1)
//         return TRUE; /* item found! */
//
//     return FALSE; /* item not found. */
// }
//
//
// static VOID
// OnBrowseButton(HWND hwndDlg, PBACKGROUND_DATA pData)
// {
//     OPENFILENAME ofn;
//     TCHAR filename[MAX_PATH];
//     TCHAR fileTitle[256];
//     TCHAR initialDir[MAX_PATH];
//     LPTSTR filter;
//     LPTSTR extensions;
//     BackgroundItem *backgroundItem = NULL;
//     SHFILEINFO sfi;
//     LV_ITEM listItem;
//     HWND hwndBackgroundList;
//     TCHAR *p;
//     HRESULT hr;
//     TCHAR filterdesc[MAX_PATH];
//     TCHAR *c;
//     size_t sizeRemain;
//     SIZE_T buffersize;
//     BOOL success;
//     HIMAGELIST himl;
//
//     hwndBackgroundList = GetDlgItem(hwndDlg, IDC_BACKGROUND_LIST);
//     himl = ListView_GetImageList(hwndBackgroundList, LVSIL_SMALL);
//     SHGetFolderPathW(NULL, CSIDL_MYPICTURES, NULL, 0, initialDir);
//
//     ZeroMemory(&ofn, sizeof(OPENFILENAME));
//
//     ofn.lStructSize = sizeof(OPENFILENAME);
//     ofn.hwndOwner = hwndDlg;
//     ofn.lpstrFile = filename;
//
//     LoadString(hApplet, IDS_BACKGROUND_COMDLG_FILTER, filterdesc, sizeof(filterdesc) / sizeof(TCHAR));
//
//     extensions = GdipGetSupportedFileExtensions();
//     if (!extensions)
//     {
//         return;
//     }
//
//     buffersize = (_tcslen(extensions) * 2 + 6) * sizeof(TCHAR) + sizeof(filterdesc);
//
//     filter = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, buffersize);
//     if (!filter)
//     {
//         HeapFree(GetProcessHeap(), 0, extensions);
//         return;
//     }
//
//     sizeRemain = buffersize;
//     c = filter;
//
//     if (FAILED(StringCbPrintfEx(c, sizeRemain, &c, &sizeRemain, 0, L"%ls (%ls)", filterdesc, extensions)))
//     {
//         HeapFree(GetProcessHeap(), 0, extensions);
//         HeapFree(GetProcessHeap(), 0, filter);
//         return;
//     }
//
//     c++;
//     sizeRemain -= sizeof(*c);
//
//     if (FAILED(StringCbPrintfEx(c, sizeRemain, &c, &sizeRemain, 0, L"%ls", extensions)))
//     {
//         HeapFree(GetProcessHeap(), 0, extensions);
//         HeapFree(GetProcessHeap(), 0, filter);
//         return;
//     }
//
//     HeapFree(GetProcessHeap(), 0, extensions);
//
//     /* Set lpstrFile[0] to '\0' so that GetOpenFileName does not
//      * use the contents of szFile to initialize itself */
//     ofn.lpstrFile[0] = TEXT('\0');
//     ofn.nMaxFile = MAX_PATH;
//     ofn.lpstrFilter = filter;
//     ofn.nFilterIndex = 0;
//     ofn.lpstrFileTitle = fileTitle;
//     ofn.nMaxFileTitle = 256;
//     ofn.lpstrInitialDir = initialDir;
//     ofn.Flags = OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_HIDEREADONLY | OFN_EXPLORER;
//
//     success = GetOpenFileName(&ofn);
//     HeapFree(GetProcessHeap(), 0, filter);
//
//     if (success)
//     {
//         /* Check if there is already a entry that holds this filename */
//         if (CheckListViewFilenameExists(hwndBackgroundList, ofn.lpstrFileTitle) != FALSE)
//             return;
//
//         if (pData->listViewItemCount > (MAX_BACKGROUNDS - 1))
//             return;
//
//         SHGetFileInfo(filename,
//                       0,
//                       &sfi,
//                       sizeof(sfi),
//                       SHGFI_ICON | SHGFI_SMALLICON | SHGFI_DISPLAYNAME);
//         sfi.iIcon = ImageList_AddIcon(himl, sfi.hIcon);
//
//         backgroundItem = &pData->backgroundItems[pData->listViewItemCount];
//
//         backgroundItem->bWallpaper = TRUE;
//
//         hr = StringCbCopy(backgroundItem->szDisplayName, sizeof(backgroundItem->szDisplayName), sfi.szDisplayName);
//         if (FAILED(hr))
//             return;
//         p = _tcsrchr(backgroundItem->szDisplayName, _T('.'));
//         if (p)
//             *p = (TCHAR)0;
//         hr = StringCbCopy(backgroundItem->szFilename, sizeof(backgroundItem->szFilename), filename);
//         if (FAILED(hr))
//             return;
//
//         ZeroMemory(&listItem, sizeof(LV_ITEM));
//         listItem.mask       = LVIF_TEXT | LVIF_PARAM | LVIF_STATE | LVIF_IMAGE;
//         listItem.state      = 0;
//         listItem.pszText    = backgroundItem->szDisplayName;
//         listItem.iImage     = sfi.iIcon;
//         listItem.iItem      = pData->listViewItemCount;
//         listItem.lParam     = pData->listViewItemCount;
//
//         (void)ListView_InsertItem(hwndBackgroundList, &listItem);
//         ListView_SetItemState(hwndBackgroundList,
//                               pData->listViewItemCount,
//                               LVIS_SELECTED,
//                               LVIS_SELECTED);
//         SendMessage(hwndBackgroundList, WM_VSCROLL, SB_BOTTOM, 0);
//
//         pData->listViewItemCount++;
//     }
// }
//
//
// static VOID
// ListViewItemChanged(HWND hwndDlg, PBACKGROUND_DATA pData, int itemIndex)
// {
//     BackgroundItem *backgroundItem = NULL;
//
//     pData->backgroundSelection = itemIndex;
//     backgroundItem = &pData->backgroundItems[pData->backgroundSelection];
//
//     if (pData->pWallpaperBitmap != NULL)
//     {
//         DibFreeImage(pData->pWallpaperBitmap);
//         pData->pWallpaperBitmap = NULL;
//     }
//
//     if (backgroundItem->bWallpaper != FALSE)
//     {
//         pData->pWallpaperBitmap = DibLoadImage(backgroundItem->szFilename);
//
//         if (pData->pWallpaperBitmap == NULL)
//             return;
//     }
//
//     pData->bWallpaperChanged = TRUE;
//
//     InvalidateRect(GetDlgItem(hwndDlg, IDC_BACKGROUND_PREVIEW),
//                    NULL, TRUE);
//
//     EnableWindow(GetDlgItem(hwndDlg, IDC_PLACEMENT_COMBO),
//                  backgroundItem->bWallpaper);
//
//     PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
// }
//
//
// static VOID
// DrawBackgroundPreview(LPDRAWITEMSTRUCT draw, PBACKGROUND_DATA pData)
// {
//     float scaleX;
//     float scaleY;
//     int scaledWidth;
//     int scaledHeight;
//     int posX, desX;
//     int posY, desY;
//     int fitFillScaleNum, fitFillScaleDen;
//     int fitFillWidth, fitFillHeight;
//     HBRUSH hBrush;
//     int x;
//     int y;
//     HDC hDC;
//     HGDIOBJ hOldObj;
//     RECT rcItem = {
//         MONITOR_LEFT,
//         MONITOR_TOP,
//         MONITOR_RIGHT,
//         MONITOR_BOTTOM
//     };
//
//     hDC = CreateCompatibleDC(draw->hDC);
//     hOldObj = SelectObject(hDC, g_GlobalData.hMonitorBitmap);
//
//     if (pData->backgroundItems[pData->backgroundSelection].bWallpaper == FALSE)
//     {
//         /* Update desktop background color image */
//         hBrush = CreateSolidBrush(g_GlobalData.desktop_color);
//         FillRect(hDC, &rcItem, hBrush);
//         DeleteObject(hBrush);
//     }
//     else
//     if (pData->pWallpaperBitmap != NULL)
//     {
//         scaleX = ((float)GetSystemMetrics(SM_CXSCREEN) - 1) / (float)MONITOR_WIDTH;
//         scaleY = ((float)GetSystemMetrics(SM_CYSCREEN) - 1) / (float)MONITOR_HEIGHT;
//
//         scaledWidth = (int)(pData->pWallpaperBitmap->width / scaleX);
//         scaledHeight = (int)(pData->pWallpaperBitmap->height / scaleY);
//
//         FillRect(hDC, &rcItem, GetSysColorBrush(COLOR_BACKGROUND));
//
//         SetStretchBltMode(hDC, COLORONCOLOR);
//
//         switch (pData->placementSelection)
//         {
//             case PLACEMENT_CENTER:
//                 posX = (MONITOR_WIDTH - scaledWidth + 1) / 2;
//                 posY = (MONITOR_HEIGHT - scaledHeight + 1) / 2;
//                 desX = 0;
//                 desY = 0;
//
//                 if (posX < 0) { desX = -posX / 2; posX = 0; }
//                 if (posY < 0) { desY = -posY / 2; posY = 0; }
//
//                 if (scaledWidth > MONITOR_WIDTH)
//                     scaledWidth = MONITOR_WIDTH;
//
//                 if (scaledHeight > MONITOR_HEIGHT)
//                     scaledHeight = MONITOR_HEIGHT;
//
//                 StretchDIBits(hDC,
//                               MONITOR_LEFT+posX,
//                               MONITOR_TOP+posY,
//                               scaledWidth,
//                               scaledHeight,
//                               desX,
//                               desY,
//                               pData->pWallpaperBitmap->width - (int)(desX * scaleX),
//                               pData->pWallpaperBitmap->height - (int)(desY * scaleY),
//                               pData->pWallpaperBitmap->bits,
//                               pData->pWallpaperBitmap->info,
//                               DIB_RGB_COLORS,
//                               SRCCOPY);
//                 break;
//
//             case PLACEMENT_STRETCH:
//                 StretchDIBits(hDC,
//                               MONITOR_LEFT,
//                               MONITOR_TOP,
//                               MONITOR_WIDTH,
//                               MONITOR_HEIGHT,
//                               0,
//                               0,
//                               pData->pWallpaperBitmap->width,
//                               pData->pWallpaperBitmap->height,
//                               pData->pWallpaperBitmap->bits,
//                               pData->pWallpaperBitmap->info,
//                               DIB_RGB_COLORS,
//                               SRCCOPY);
//                 break;
//
//             case PLACEMENT_TILE:
//                 for (y = 0; y < MONITOR_HEIGHT; y += scaledHeight)
//                 {
//                     for (x = 0; x < MONITOR_WIDTH; x += scaledWidth)
//                     {
//                         if ((MONITOR_WIDTH-x) >= scaledWidth)
//                             posX = scaledWidth;
//                         else
//                             posX = MONITOR_WIDTH-x;
//
//
//                         if ((MONITOR_HEIGHT-y) >= scaledHeight)
//                             posY = scaledHeight;
//                         else
//                             posY = MONITOR_HEIGHT-y;
//
//                         StretchDIBits(hDC,
//                                       MONITOR_LEFT + x,
//                                       MONITOR_TOP + y,
//                                       posX,
//                                       posY,
//                                       0,
//                                       0,
//                                       pData->pWallpaperBitmap->width * posX / scaledWidth,
//                                       pData->pWallpaperBitmap->height * posY / scaledHeight,
//                                       pData->pWallpaperBitmap->bits,
//                                       pData->pWallpaperBitmap->info,
//                                       DIB_RGB_COLORS,
//                                       SRCCOPY);
//                     }
//
//                 }
//
//                 break;
//
//             case PLACEMENT_FIT:
//                 if ((MONITOR_WIDTH * scaledHeight) <= (MONITOR_HEIGHT * scaledWidth))
//                 {
//                     fitFillScaleNum = MONITOR_WIDTH;
//                     fitFillScaleDen = scaledWidth;
//                 }
//                 else
//                 {
//                     fitFillScaleNum = MONITOR_HEIGHT;
//                     fitFillScaleDen = scaledHeight;
//                 }
//
//                 fitFillWidth = MulDiv(scaledWidth, fitFillScaleNum, fitFillScaleDen);
//                 fitFillHeight = MulDiv(scaledHeight, fitFillScaleNum, fitFillScaleDen);
//
//                 posX = (MONITOR_WIDTH - fitFillWidth) / 2;
//                 posY = (MONITOR_HEIGHT - fitFillHeight) / 2;
//
//                 StretchDIBits(hDC,
//                               MONITOR_LEFT + posX,
//                               MONITOR_TOP + posY,
//                               fitFillWidth,
//                               fitFillHeight,
//                               0,
//                               0,
//                               pData->pWallpaperBitmap->width,
//                               pData->pWallpaperBitmap->height,
//                               pData->pWallpaperBitmap->bits,
//                               pData->pWallpaperBitmap->info,
//                               DIB_RGB_COLORS,
//                               SRCCOPY);
//                 break;
//
//             case PLACEMENT_FILL:
//                 if ((MONITOR_WIDTH * scaledHeight) > (MONITOR_HEIGHT * scaledWidth))
//                 {
//                     fitFillScaleNum = MONITOR_WIDTH;
//                     fitFillScaleDen = scaledWidth;
//                 }
//                 else
//                 {
//                     fitFillScaleNum = MONITOR_HEIGHT;
//                     fitFillScaleDen = scaledHeight;
//                 }
//
//                 fitFillWidth = MulDiv(scaledWidth, fitFillScaleNum, fitFillScaleDen);
//                 fitFillHeight = MulDiv(scaledHeight, fitFillScaleNum, fitFillScaleDen);
//
//                 desX = (((fitFillWidth - MONITOR_WIDTH) * pData->pWallpaperBitmap->width) / (2 * fitFillWidth));
//                 desY = (((fitFillHeight - MONITOR_HEIGHT) * pData->pWallpaperBitmap->height) / (2 * fitFillHeight));
//
//                 StretchDIBits(hDC,
//                               MONITOR_LEFT,
//                               MONITOR_TOP,
//                               MONITOR_WIDTH,
//                               MONITOR_HEIGHT,
//                               desX,
//                               desY,
//                               (MONITOR_WIDTH * pData->pWallpaperBitmap->width) / fitFillWidth,
//                               (MONITOR_HEIGHT * pData->pWallpaperBitmap->height) / fitFillHeight,
//                               pData->pWallpaperBitmap->bits,
//                               pData->pWallpaperBitmap->info,
//                               DIB_RGB_COLORS,
//                               SRCCOPY);
//                 break;
//         }
//     }
//
//     GdiTransparentBlt(draw->hDC,
//                       draw->rcItem.left, draw->rcItem.top,
//                       draw->rcItem.right - draw->rcItem.left + 1,
//                       draw->rcItem.bottom - draw->rcItem.top + 1,
//                       hDC,
//                       0, 0,
//                       g_GlobalData.bmMonWidth, g_GlobalData.bmMonHeight,
//                       MONITOR_ALPHA);
//
//     SelectObject(hDC, hOldObj);
//     DeleteDC(hDC);
// }
//
//
// static VOID
// SetWallpaper(PBACKGROUND_DATA pData)
// {
//     HKEY regKey;
//     TCHAR szWallpaper[MAX_PATH];
//     GpImage *image;
//     CLSID  encoderClsid;
//     GUID guidFormat;
//     size_t length = 0;
//     GpStatus status;
//
//     if (FAILED(SHGetFolderPath(NULL, CSIDL_LOCAL_APPDATA | CSIDL_FLAG_CREATE, NULL, 0, szWallpaper)))
//     {
//         return;
//     }
//
//     if (FAILED(StringCbCat(szWallpaper, sizeof(szWallpaper), TEXT("\\Wallpaper1.bmp"))))
//     {
//         return;
//     }
//
//     if (RegCreateKeyEx(HKEY_CURRENT_USER, TEXT("Control Panel\\Desktop"), 0, NULL,
//                        REG_OPTION_NON_VOLATILE, KEY_SET_VALUE, NULL, &regKey, NULL) != ERROR_SUCCESS)
//     {
//         return;
//     }
//
//     if (pData->placementSelection == PLACEMENT_TILE)
//     {
//         RegSetValueEx(regKey, TEXT("TileWallpaper"), 0, REG_SZ, (LPBYTE)TEXT("1"), sizeof(TCHAR) * 2);
//         RegSetValueEx(regKey, TEXT("WallpaperStyle"), 0, REG_SZ, (LPBYTE)TEXT("0"), sizeof(TCHAR) * 2);
//     }
//
//     if (pData->placementSelection == PLACEMENT_CENTER)
//     {
//         RegSetValueEx(regKey, TEXT("TileWallpaper"), 0, REG_SZ, (LPBYTE)TEXT("0"), sizeof(TCHAR) * 2);
//         RegSetValueEx(regKey, TEXT("WallpaperStyle"), 0, REG_SZ, (LPBYTE)TEXT("0"), sizeof(TCHAR) * 2);
//     }
//
//     if (pData->placementSelection == PLACEMENT_STRETCH)
//     {
//         RegSetValueEx(regKey, TEXT("TileWallpaper"), 0, REG_SZ, (LPBYTE)TEXT("0"), sizeof(TCHAR) * 2);
//         RegSetValueEx(regKey, TEXT("WallpaperStyle"), 0, REG_SZ, (LPBYTE)TEXT("2"), sizeof(TCHAR) * 2);
//     }
//
//     if (pData->placementSelection == PLACEMENT_FIT)
//     {
//         RegSetValueEx(regKey, TEXT("TileWallpaper"), 0, REG_SZ, (LPBYTE)TEXT("0"), sizeof(TCHAR) * 2);
//         RegSetValueEx(regKey, TEXT("WallpaperStyle"), 0, REG_SZ, (LPBYTE)TEXT("6"), sizeof(TCHAR) * 2);
//     }
//
//     if (pData->placementSelection == PLACEMENT_FILL)
//     {
//         RegSetValueEx(regKey, TEXT("TileWallpaper"), 0, REG_SZ, (LPBYTE)TEXT("0"), sizeof(TCHAR) * 2);
//         RegSetValueEx(regKey, TEXT("WallpaperStyle"), 0, REG_SZ, (LPBYTE)TEXT("10"), sizeof(TCHAR) * 3);
//     }
//
//     if (pData->backgroundItems[pData->backgroundSelection].bWallpaper != FALSE)
//     {
//         GdipLoadImageFromFile(pData->backgroundItems[pData->backgroundSelection].szFilename, &image);
//         if (!image)
//         {
//             RegCloseKey(regKey);
//             return;
//         }
//
//         GdipGetImageRawFormat(image, &guidFormat);
//         if (IsEqualGUID(&guidFormat, &ImageFormatBMP))
//         {
//             GdipDisposeImage(image);
//             RegCloseKey(regKey);
//             SystemParametersInfo(SPI_SETDESKWALLPAPER, 0, pData->backgroundItems[pData->backgroundSelection].szFilename, SPIF_UPDATEINIFILE);
//             return;
//         }
//
//         if (FAILED(GdipGetEncoderClsid(L"image/bmp", &encoderClsid)))
//         {
//             GdipDisposeImage(image);
//             RegCloseKey(regKey);
//             return;
//         }
//
//         status = GdipSaveImageToFile(image, szWallpaper, &encoderClsid, NULL);
//
//         GdipDisposeImage(image);
//
//         if (status != Ok)
//         {
//             RegCloseKey(regKey);
//             return;
//         }
//
//         if (SUCCEEDED(StringCchLength(pData->backgroundItems[pData->backgroundSelection].szFilename, MAX_PATH, &length)))
//         {
//             RegSetValueEx(regKey,
//                           TEXT("ConvertedWallpaper"),
//                           0,
//                           REG_SZ,
//                           (LPBYTE)pData->backgroundItems[pData->backgroundSelection].szFilename,
//                           (DWORD)((length + 1) * sizeof(TCHAR)));
//         }
//
//         if (SUCCEEDED(StringCchLength(szWallpaper, MAX_PATH, &length)))
//         {
//             RegSetValueEx(regKey,
//                           TEXT("OriginalWallpaper"),
//                           0,
//                           REG_SZ,
//                           (LPBYTE)szWallpaper,
//                           (DWORD)((length + 1) * sizeof(TCHAR)));
//         }
//
//         SystemParametersInfo(SPI_SETDESKWALLPAPER, 0, szWallpaper, SPIF_UPDATEINIFILE);
//     }
//     else
//     {
//         SystemParametersInfo(SPI_SETDESKWALLPAPER, 0, (void*) TEXT(""), SPIF_UPDATEINIFILE);
//     }
//
//     RegCloseKey(regKey);
// }
//
//
// /* Change system color */
// static VOID
// SetDesktopBackColor(HWND hwndDlg, PBACKGROUND_DATA pData)
// {
//     HKEY hKey;
//     INT iElement = COLOR_BACKGROUND;
//     TCHAR clText[16];
//     BYTE red, green, blue;
//
//     if (!SetSysColors(1, &iElement, &g_GlobalData.desktop_color))
//     {
//         /* FIXME: these error texts can need internationalization? */
//         MessageBox(hwndDlg, TEXT("SetSysColor() failed!"),
//             TEXT("Error!"), MB_ICONSTOP );
//     }
//
//     if (RegCreateKeyEx(HKEY_CURRENT_USER, TEXT("Control Panel\\Colors"), 0, NULL,
//                        REG_OPTION_NON_VOLATILE, KEY_SET_VALUE, NULL, &hKey, NULL) != ERROR_SUCCESS)
//     {
//         return;
//     }
//
//     red   = GetRValue(g_GlobalData.desktop_color);
//     green = GetGValue(g_GlobalData.desktop_color);
//     blue  = GetBValue(g_GlobalData.desktop_color);
//
//     /* Format string to be set to registry */
//     StringCbPrintf(clText, sizeof(clText), TEXT("%d %d %d"), red, green, blue);
//     RegSetValueEx(hKey, TEXT("Background"), 0, REG_SZ, (LPBYTE)clText,
//                   (wcslen(clText) + 1) * sizeof(TCHAR));
//
//     RegCloseKey(hKey);
// }
//
// static VOID
// OnCustomButton(HWND hwndDlg, PBACKGROUND_DATA pData)
// {
//     HPROPSHEETPAGE hpsp[1] = {0};
//     PROPSHEETHEADER psh = {sizeof(psh)};
//     PROPSHEETPAGE psp = {sizeof(psp)};
//
//     psh.dwFlags = PSH_NOAPPLYNOW;
//     psh.hwndParent = GetParent(hwndDlg);
//     psh.hInstance = hApplet;
//     psh.pszCaption = MAKEINTRESOURCE(IDS_DESKTOP_ITEMS);
//     psh.phpage = hpsp;
//
//     psp.dwFlags = PSP_DEFAULT;
//     psp.hInstance = hApplet;
//     psp.pszTemplate = MAKEINTRESOURCE(IDD_DESKTOP_GENERAL);
//     psp.pfnDlgProc = DesktopPageProc;
//     psp.lParam = (LPARAM)&pData->desktopData;
//
//     hpsp[0] = CreatePropertySheetPage(&psp);
//     if (!hpsp[0])
//         return;
//
//     psh.nPages++;
//
//     if (PropertySheet(&psh) > 0)
//     {
//         if (SaveDesktopSettings(&pData->desktopData))
//             PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//     }
// }
//
//
// INT_PTR __stdcall
// BackgroundPageProc(HWND hwndDlg,
//                    UINT uMsg,
//                    WPARAM wParam,
//                    LPARAM lParam)
// {
//     PBACKGROUND_DATA pData;
//     struct GdiplusStartupInput gdipStartup;
//
//     pData = (PBACKGROUND_DATA)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             pData = (PBACKGROUND_DATA)HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(BACKGROUND_DATA));
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pData);
//             gdipStartup.GdiplusVersion = 1;
//             gdipStartup.DebugEventCallback = NULL;
//             gdipStartup.SuppressBackgroundThread = FALSE;
//             gdipStartup.SuppressExternalCodecs = FALSE;
//             GdiplusStartup(&pData->gdipToken, &gdipStartup, NULL);
//             InitBackgroundDialog(hwndDlg, pData);
//             InitDesktopSettings(&pData->desktopData);
//             break;
//
//         case WM_COMMAND:
//             {
//                 DWORD controlId = LOWORD(wParam);
//                 DWORD command   = HIWORD(wParam);
//
//                 switch (controlId)
//                 {
//                     case IDC_COLOR_BUTTON:
//                         if (command == BN_CLICKED)
//                             OnColorButton(hwndDlg, pData);
//                         break;
//
//                     case IDC_BROWSE_BUTTON:
//                         if (command == BN_CLICKED)
//                             OnBrowseButton(hwndDlg, pData);
//                         break;
//
//                     case IDC_PLACEMENT_COMBO:
//                         if (command == CBN_SELCHANGE)
//                         {
//                             pData->placementSelection = (int)SendDlgItemMessage(hwndDlg, IDC_PLACEMENT_COMBO, CB_GETCURSEL, 0, 0);
//
//                             InvalidateRect(GetDlgItem(hwndDlg, IDC_BACKGROUND_PREVIEW), NULL, TRUE);
//
//                             PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                         }
//                         break;
//
//                     case IDC_DESKTOP_CUSTOM:
//                         if (command == BN_CLICKED)
//                             OnCustomButton(hwndDlg, pData);
//                         break;
//                 }
//             } break;
//
//         case WM_DRAWITEM:
//             {
//                 LPDRAWITEMSTRUCT drawItem = (LPDRAWITEMSTRUCT)lParam;
//
//                 if (drawItem->CtlID == IDC_BACKGROUND_PREVIEW)
//                     DrawBackgroundPreview(drawItem, pData);
//             }
//             break;
//
//         case WM_NOTIFY:
//             {
//                 LPNMHDR lpnm = (LPNMHDR)lParam;
//
//                 switch(lpnm->code)
//                 {
//                     case PSN_APPLY:
//                         if (pData->bWallpaperChanged)
//                             SetWallpaper(pData);
//                         if (pData->bClrBackgroundChanged)
//                             SetDesktopBackColor(hwndDlg, pData);
//                         if (pData->desktopData.bSettingsChanged)
//                             SetDesktopSettings(&pData->desktopData);
//                         SendMessage(HWND_BROADCAST, WM_SETTINGCHANGE, 0, (LPARAM)_T(""));
//                         return TRUE;
//
//                     case LVN_ITEMCHANGED:
//                         {
//                             LPNMLISTVIEW nm = (LPNMLISTVIEW)lParam;
//
//                             if ((nm->uNewState & LVIS_SELECTED) == 0)
//                                 return FALSE;
//
//                             ListViewItemChanged(hwndDlg, pData, nm->iItem);
//                         }
//                         break;
//
//                     case NM_CUSTOMDRAW:
//                         if (lpnm->idFrom == IDC_COLOR_BUTTON)
//                         {
//                             return SetDlgMsgResult(hwndDlg, 0, ClrBtn_CustomDraw((NMCUSTOMDRAW*)lpnm,
//                                                                                  g_GlobalData.desktop_color));
//                         }
//                         break;
//                 }
//             }
//             break;
//
//         case WM_DESTROY:
//             if (pData->pWallpaperBitmap != NULL)
//                 DibFreeImage(pData->pWallpaperBitmap);
//
//             GdiplusShutdown(pData->gdipToken);
//             HeapFree(GetProcessHeap(), 0, pData);
//             break;
//     }
//
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
